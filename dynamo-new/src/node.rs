use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use log::{info, debug, warn};
use reactor_actor::{ActorProcess, ActorSend, BehaviourBuilder, RouteTo, RuntimeCtx};
use reactor_actor::codec::BincodeCodec;

use crate::consistent_hash::ConsistentHash;
use crate::messages::{DynamoNodeIn, DynamoNodeOut, NodeToNode, NodeToClient, ClientToNode};
use crate::vector_clock::VectorClock;
use crate::versioned_value::{VersionedValue, VersionedValues};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ReqKind { Put, Get }

struct Deadline { to: String, kind: ReqKind, seq: u64, at: Instant, key: String }

pub struct DynamoNode {
    node_id: String,
    nodes: Vec<String>,
    ring: ConsistentHash,
    n: usize,
    w: usize,
    r: usize,
    t: usize,
    seq: u64,
    store: HashMap<String, VersionedValues>,
    // pending
    pending_put_rsp: HashMap<u64, HashSet<String>>, // seq -> acks
    pending_put_msg: HashMap<u64, (String, String, u64)>, // seq -> (client_addr, key, client_req_id)
    pending_put_data: HashMap<u64, (String, String, VectorClock)>, // seq -> (key, value, clock)
    pending_get_rsp: HashMap<u64, Vec<(String, VersionedValues)>>, // seq -> list of (from, values)
    pending_get_msg: HashMap<u64, (String, String, u64)>, // seq -> (client_addr, key, client_req_id)
    pending_req: HashMap<(ReqKind, u64), HashSet<String>>, // (kind, seq) -> sent nodes
    failed: HashSet<String>,
    handoffs: HashMap<String, HashSet<String>>, // failed_node -> keys to handoff
    deadlines: Vec<Deadline>,
    timeout_ms: u64,
    last_ping: Instant,
    ping_interval_ms: u64,
    // anti-entropy sweep
    sync_cursor: usize,
    sync_batch: usize,
}

impl DynamoNode {
    pub fn new(node_id: String, nodes: Vec<String>, n: usize, w: usize, r: usize, t: usize) -> Self {
        let ring = ConsistentHash::new(&nodes, t);
        Self {
            node_id, nodes, ring, n, w, r, t,
            seq: 0,
            store: HashMap::new(),
            pending_put_rsp: HashMap::new(),
            pending_put_msg: HashMap::new(),
            pending_get_rsp: HashMap::new(),
            pending_get_msg: HashMap::new(),
            pending_req: HashMap::new(),
            pending_put_data: HashMap::new(),
            failed: HashSet::new(),
            handoffs: HashMap::new(),
            deadlines: vec![],
            timeout_ms: 800,
            last_ping: Instant::now(),
            ping_interval_ms: 1000,
            sync_cursor: 0,
            sync_batch: 2,
        }
    }

    fn next_seq(&mut self) -> u64 { self.seq += 1; self.seq }

    fn sweep_timeouts(&mut self) -> Vec<DynamoNodeOut> {
        let now = Instant::now();
        let mut out = vec![];
        let mut expired: Vec<usize> = vec![];
        for (i, d) in self.deadlines.iter().enumerate() {
            if now >= d.at {
                expired.push(i);
            }
        }
        // remove in reverse to keep indices valid
        expired.sort(); expired.reverse();
        for idx in expired {
            let d = self.deadlines.remove(idx);
            if self.failed.insert(d.to.clone()) {
                warn!("node {} timeout to {} on {:?} seq {}", self.node_id, d.to, d.kind, d.seq);
            }
            // retry to an additional node not yet used
            let (pref, _avoided) = self.ring.find_nodes(&d.key, self.n, &self.failed.iter().cloned().collect::<Vec<_>>() );
            let key = d.key.clone();
            let sent = self.pending_req.entry((d.kind, d.seq)).or_default().clone();
            for node in pref {
                if !sent.contains(&node) {
                    match d.kind {
                        ReqKind::Put => {
                            if let Some((k, v, clk)) = self.pending_put_data.get(&d.seq).cloned() {
                                info!("[put-timeout-retry] coord={} key={} retry_to={} seq={}", self.node_id, k, node, d.seq);
                                out.push(DynamoNodeOut::NodeToNode(NodeToNode::PutReq{
                                    from: self.node_id.clone(), to: node.clone(), key: k.clone(), value: v.clone(), clock: clk.clone(), msg_id: d.seq, handoff: None
                                }));
                                self.pending_req.entry((ReqKind::Put, d.seq)).or_default().insert(node.clone());
                                self.deadlines.push(Deadline{ to: node, kind: ReqKind::Put, seq: d.seq, at: Instant::now() + Duration::from_millis(self.timeout_ms), key: key.clone() });
                            }
                        }
                        ReqKind::Get => {
                            out.push(DynamoNodeOut::NodeToNode(NodeToNode::GetReq{ from: self.node_id.clone(), to: node.clone(), key: key.clone(), msg_id: d.seq }));
                            self.pending_req.entry((ReqKind::Get, d.seq)).or_default().insert(node.clone());
                            self.deadlines.push(Deadline{ to: node.clone(), kind: ReqKind::Get, seq: d.seq, at: Instant::now() + Duration::from_millis(self.timeout_ms), key: key.clone() });
                        }
                    }
                    break;
                }
            }
        }
        out
    }

    fn on_client_put(&mut self, key: String, value: String, mut meta: Vec<VectorClock>, client_addr: String, request_id: u64) -> Vec<DynamoNodeOut> {
        let (pref, avoided) = self.ring.find_nodes(&key, self.n, &self.failed.iter().cloned().collect::<Vec<_>>() );
        if !pref.contains(&self.node_id) {
            let coord = pref.get(0).cloned().unwrap_or_else(|| self.node_id.clone());
            info!("[forward-put] at={} forwarding key={} to coordinator={} pref={:?}", self.node_id, key, coord, pref);
            return vec![DynamoNodeOut::NodeToNode(NodeToNode::ForwardClientPut{ coordinator: coord, key, value, metadata: meta, client_addr, request_id })];
        }
        // coordinator path
        let seq = self.next_seq();
        self.pending_put_rsp.insert(seq, HashSet::new());
        self.pending_put_msg.insert(seq, (client_addr.clone(), key.clone(), request_id));
        self.pending_req.entry((ReqKind::Put, seq)).or_default();

        // build clock: converge metadata if provided, then update this node with seq
        let mut clock = if meta.is_empty() { VectorClock::new() } else { VectorClock::converge(meta.drain(..)) };
        clock.update(&self.node_id, seq);
        self.pending_put_data.insert(seq, (key.clone(), value.clone(), clock.clone()));

        let avoided_top_n: Vec<String> = avoided.into_iter().take(self.n).collect();
        let non_extra = self.n.saturating_sub(avoided_top_n.len());
        let mut out = vec![];
        info!("[coord-put] coord={} key={} seq={} pref={:?} avoided_top_n={:?} clock={:?}", self.node_id, key, seq, pref, avoided_top_n, clock.clock);
        for (i, node) in pref.into_iter().enumerate() {
            let handoff = if i >= non_extra && !avoided_top_n.is_empty() { Some(avoided_top_n.clone()) } else { None };
            out.push(DynamoNodeOut::NodeToNode(NodeToNode::PutReq{
                from: self.node_id.clone(), to: node.clone(), key: key.clone(), value: value.clone(), clock: clock.clone(), msg_id: seq, handoff
            }));
            self.pending_req.get_mut(&(ReqKind::Put, seq)).unwrap().insert(node.clone());
            self.deadlines.push(Deadline{ to: node, kind: ReqKind::Put, seq, at: Instant::now() + Duration::from_millis(self.timeout_ms), key: key.clone() });
        }
        out
    }

    fn on_forward_client_put(&mut self, coordinator: String, key: String, value: String, metadata: Vec<VectorClock>, client_addr: String, request_id: u64) -> Vec<DynamoNodeOut> {
        if coordinator != self.node_id { return vec![DynamoNodeOut::NodeToNode(NodeToNode::ForwardClientPut{ coordinator, key, value, metadata, client_addr, request_id })]; }
        self.on_client_put(key, value, metadata, client_addr, request_id)
    }

    fn on_put_req(&mut self, from: String, key: String, value: String, clock: VectorClock, handoff: Option<Vec<String>>, msg_id: u64) -> Vec<DynamoNodeOut> {
        let entry = self.store.entry(key.clone()).or_insert_with(VersionedValues::new);
        entry.add_version(VersionedValue::new(value, clock));
        debug!("[store-put] node={} key={} versions_now={}", self.node_id, key, entry.versions.len());
    if let Some(ref h) = handoff { for failed in h.iter() { self.failed.insert(failed.clone()); self.handoffs.entry(failed.clone()).or_default().insert(key.clone()); } }
    if let Some(h) = &handoff { info!("[hinted-handoff] node={} storing hints for failed={:?} key={}", self.node_id, h, key); }
        vec![DynamoNodeOut::NodeToNode(NodeToNode::PutRsp{ from: self.node_id.clone(), to: from, msg_id })]
    }

    fn on_put_rsp(&mut self, from: String, msg_id: u64) -> Vec<DynamoNodeOut> {
        if let Some(acks) = self.pending_put_rsp.get_mut(&msg_id) {
            acks.insert(from);
            info!("[put-ack] coord={} seq={} acks={}/{}", self.node_id, msg_id, acks.len(), self.w);
            if acks.len() >= self.w {
                if let Some((client, key, client_req_id)) = self.pending_put_msg.remove(&msg_id) {
                    self.pending_put_rsp.remove(&msg_id);
                    self.pending_req.remove(&(ReqKind::Put, msg_id));
                    self.pending_put_data.remove(&msg_id);
                    // clear deadlines for this seq
                    self.deadlines.retain(|d| !(d.seq==msg_id && matches!(d.kind, ReqKind::Put)));
                    return vec![DynamoNodeOut::NodeToClient(NodeToClient::ClientPutRsp{ key, request_id: client_req_id, client_addr: client })];
                }
            }
        }
        vec![]
    }

    fn on_client_get(&mut self, key: String, client_addr: String, request_id: u64) -> Vec<DynamoNodeOut> {
        let (pref, _avoided) = self.ring.find_nodes(&key, self.n, &self.failed.iter().cloned().collect::<Vec<_>>() );
        if !pref.contains(&self.node_id) {
            let coord = pref.get(0).cloned().unwrap_or_else(|| self.node_id.clone());
            info!("[forward-get] at={} forwarding key={} to coordinator={} pref={:?}", self.node_id, key, coord, pref);
            return vec![DynamoNodeOut::NodeToNode(NodeToNode::ForwardClientGet{ coordinator: coord, key, client_addr, request_id })];
        }
        let seq = self.next_seq();
    self.pending_get_msg.insert(seq, (client_addr.clone(), key.clone(), request_id));
        self.pending_get_rsp.insert(seq, vec![]);
        self.pending_req.entry((ReqKind::Get, seq)).or_default();
        let mut out = vec![];
        info!("[coord-get] coord={} key={} seq={} pref={:?}", self.node_id, key, seq, pref);
        for node in pref.into_iter() {
            out.push(DynamoNodeOut::NodeToNode(NodeToNode::GetReq{ from: self.node_id.clone(), to: node.clone(), key: key.clone(), msg_id: seq }));
            self.pending_req.get_mut(&(ReqKind::Get, seq)).unwrap().insert(node.clone());
            self.deadlines.push(Deadline{ to: node.clone(), kind: ReqKind::Get, seq, at: Instant::now() + Duration::from_millis(self.timeout_ms), key: key.clone() });
        }
        out
    }

    fn on_forward_client_get(&mut self, coordinator: String, key: String, client_addr: String, request_id: u64) -> Vec<DynamoNodeOut> {
        if coordinator != self.node_id { return vec![DynamoNodeOut::NodeToNode(NodeToNode::ForwardClientGet{ coordinator, key, client_addr, request_id })]; }
        self.on_client_get(key, client_addr, request_id)
    }

    fn on_get_req(&mut self, from: String, key: String, msg_id: u64) -> Vec<DynamoNodeOut> {
        let values = self.store.get(&key).cloned().unwrap_or_default();
        debug!("[serve-get] node={} key={} versions={} to={}", self.node_id, key, values.versions.len(), from);
        vec![DynamoNodeOut::NodeToNode(NodeToNode::GetRsp{ from: self.node_id.clone(), to: from, key, values, msg_id })]
    }

    fn on_get_rsp(&mut self, from: String, key: String, values: VersionedValues, msg_id: u64) -> Vec<DynamoNodeOut> {
        if let Some(vs) = self.pending_get_rsp.get_mut(&msg_id) {
            vs.push((from, values));
            debug!("[get-rsp] coord={} seq={} collected={}", self.node_id, msg_id, vs.len());
            if vs.len() >= self.r {
                if let Some((client, _k, client_req_id)) = self.pending_get_msg.remove(&msg_id) {
                    let mut merged = VersionedValues::new();
                    for (_n, v) in vs.iter() { merged.merge(v); }
                    // read-repair: for each replica that responded, if missing any merged versions, send repairs
                    let mut repairs: Vec<DynamoNodeOut> = vec![];
                    let mut repair_count = 0usize;
                    for (replica, vset) in vs.iter() {
                        for vv in merged.versions.iter() {
                            if !vset.contains(vv) {
                                repairs.push(DynamoNodeOut::NodeToNode(NodeToNode::PutReq{
                                    from: self.node_id.clone(), to: replica.clone(), key: key.clone(), value: vv.value.clone(), clock: vv.clock.clone(), msg_id: 0, handoff: None
                                }));
                                repair_count += 1;
                            }
                        }
                    }
                    let vals: Vec<String> = merged.versions.iter().map(|v| v.value.clone()).collect();
                    let meta: Vec<VectorClock> = merged.versions.iter().map(|v| v.clock.clone()).collect();
                    self.pending_get_rsp.remove(&msg_id);
                    self.pending_req.remove(&(ReqKind::Get, msg_id));
                    self.deadlines.retain(|d| !(d.seq==msg_id && matches!(d.kind, ReqKind::Get)));
                    info!("[coord-get-rsp] coord={} key={} seq={} merged_versions={} repairs_sent={}", self.node_id, key, msg_id, vals.len(), repair_count);
                    let mut out = vec![DynamoNodeOut::NodeToClient(NodeToClient::ClientGetRsp{ key: key.clone(), request_id: client_req_id, values: vals, metadata: meta, client_addr: client })];
                    out.extend(repairs);
                    return out;
                }
            }
        }
        vec![]
    }

    fn on_add_node(&mut self, from: String, new_node: String) -> Vec<DynamoNodeOut> {
        // Check if node already exists
        if self.nodes.contains(&new_node) {
            warn!("[add-node] node={} ignoring duplicate add_node request for {}", self.node_id, new_node);
            return vec![DynamoNodeOut::NodeToNode(NodeToNode::AddNodeAck{ from: self.node_id.clone(), to: from, new_node })];
        }

        info!("[add-node] node={} adding new_node={} to cluster (current_size={})", self.node_id, new_node, self.nodes.len());

        // Add to nodes list
        self.nodes.push(new_node.clone());

        // Update consistent hash ring
        self.ring.add_node(&new_node, self.t);

        // Remove from failed set if present
        self.failed.remove(&new_node);

        // Redistribute data: for each key, check if new node should now own it
        let mut redistribution_msgs = vec![];
        let keys_to_check: Vec<String> = self.store.keys().cloned().collect();

        for key in keys_to_check {
            let (pref, _) = self.ring.find_nodes(&key, self.n, &[]);

            // If new node is now in preference list, send it the data
            if pref.contains(&new_node) {
                if let Some(vs) = self.store.get(&key) {
                    for v in &vs.versions {
                        redistribution_msgs.push(DynamoNodeOut::NodeToNode(NodeToNode::PutReq{
                            from: self.node_id.clone(),
                            to: new_node.clone(),
                            key: key.clone(),
                            value: v.value.clone(),
                            clock: v.clock.clone(),
                            msg_id: 0,
                            handoff: None
                        }));
                    }
                }
            }
        }

        info!("[add-node] node={} redistributing {} keys to new_node={}",
              self.node_id, redistribution_msgs.len(), new_node);

        let mut out = vec![DynamoNodeOut::NodeToNode(NodeToNode::AddNodeAck{
            from: self.node_id.clone(),
            to: from,
            new_node
        })];
        out.extend(redistribution_msgs);
        out
    }
}

impl ActorProcess for DynamoNode {
    type IMsg = DynamoNodeIn;
    type OMsg = DynamoNodeOut;

    fn process(&mut self, input: Self::IMsg) -> Vec<Self::OMsg> {
        // sweep deadlines each event
        let mut out = self.sweep_timeouts();
        // periodic pings to failed nodes for liveness probing
        let now = Instant::now();
        if now.duration_since(self.last_ping).as_millis() as u64 >= self.ping_interval_ms {
            self.last_ping = now;
            for n in self.failed.clone().into_iter() {
                out.push(DynamoNodeOut::NodeToNode(NodeToNode::PingReq{ from: self.node_id.clone(), to: n }));
            }
            // background anti-entropy: sync a couple of keys each tick to their replicas
            if !self.store.is_empty() {
                let keys: Vec<String> = self.store.keys().cloned().collect();
                let total = keys.len();
                for i in 0..self.sync_batch.min(total) {
                    let idx = (self.sync_cursor + i) % total;
                    let k = &keys[idx];
                    if let Some(vs) = self.store.get(k).cloned() {
                        let (pref, _avoided) = self.ring.find_nodes(k, self.n, &self.failed.iter().cloned().collect::<Vec<_>>() );
                        for node in pref.into_iter() {
                            if node != self.node_id {
                                out.push(DynamoNodeOut::NodeToNode(NodeToNode::SyncKey{ from: self.node_id.clone(), to: node, key: k.clone(), values: vs.clone() }));
                            }
                        }
                    }
                }
                self.sync_cursor = (self.sync_cursor + self.sync_batch).min(total);
                if self.sync_cursor >= total { self.sync_cursor = 0; }
            }
        }
        let more = match input {
            DynamoNodeIn::ClientToNode(c) => match c {
                ClientToNode::ClientPut{ key, value, metadata, client_addr, request_id } => self.on_client_put(key, value, metadata, client_addr, request_id),
                ClientToNode::ClientGet{ key, client_addr, request_id } => self.on_client_get(key, client_addr, request_id),
            },
            DynamoNodeIn::NodeToNode(n2n) => match n2n {
                NodeToNode::ForwardClientPut{ coordinator, key, value, metadata, client_addr, request_id } => self.on_forward_client_put(coordinator, key, value, metadata, client_addr, request_id),
                NodeToNode::ForwardClientGet{ coordinator, key, client_addr, request_id } => self.on_forward_client_get(coordinator, key, client_addr, request_id),
                NodeToNode::PutReq{ from, to:_, key, value, clock, msg_id, handoff } => self.on_put_req(from, key, value, clock, handoff, msg_id),
                NodeToNode::PutRsp{ from, to:_, msg_id } => self.on_put_rsp(from, msg_id),
                NodeToNode::GetReq{ from, to:_, key, msg_id } => self.on_get_req(from, key, msg_id),
                NodeToNode::GetRsp{ from, to:_, key, values, msg_id } => self.on_get_rsp(from, key, values, msg_id),
                NodeToNode::SyncKey{ from:_, to:_, key, values } => {
                    let entry = self.store.entry(key.clone()).or_insert_with(VersionedValues::new);
                    let before = entry.versions.len();
                    entry.merge(&values);
                    let after = entry.versions.len();
                    if after != before { info!("[anti-entropy-merge] node={} key={} versions {}->{}", self.node_id, key, before, after); }
                    vec![]
                },
                NodeToNode::PingReq{ from, to:_ } => vec![DynamoNodeOut::NodeToNode(NodeToNode::PingRsp{ from: self.node_id.clone(), to: from })],
                NodeToNode::PingRsp{ from, to:_ } => {
                    // recovered
                    self.failed.remove(&from);
                    if let Some(keys) = self.handoffs.remove(&from) {
                        info!("[recovery] node={} recovered={}; replaying_handoffs keys={:?}", self.node_id, from, keys);
                        let mut msgs = vec![];
                        for k in keys { if let Some(vs) = self.store.get(&k) {
                            for v in &vs.versions {
                                msgs.push(DynamoNodeOut::NodeToNode(NodeToNode::PutReq{ from: self.node_id.clone(), to: from.clone(), key: k.clone(), value: v.value.clone(), clock: v.clock.clone(), msg_id: 0, handoff: None }));
                            }
                        }}
                        msgs
                    } else { vec![] }
                },
                NodeToNode::AddNode{ from, to:_, new_node } => self.on_add_node(from, new_node),
                NodeToNode::AddNodeAck{ from, to:_, new_node } => {
                    info!("[add-node-ack] node={} received ack from {} for new_node={}", self.node_id, from, new_node);
                    vec![]
                },
            },
        };
        out.extend(more);
        out
    }
}

pub struct DynamoNodeSender {
    // no state now
}
impl DynamoNodeSender { pub fn new() -> Self { Self{} } }

impl ActorSend for DynamoNodeSender {
    type OMsg = DynamoNodeOut;
    async fn before_send<'a>(&'a mut self, o: &Self::OMsg) -> RouteTo<'a> {
        match o {
            DynamoNodeOut::NodeToNode(n2n) => {
                match n2n {
                    NodeToNode::ForwardClientPut{ coordinator, .. } => RouteTo::from(coordinator.clone()),
                    NodeToNode::ForwardClientGet{ coordinator, .. } => RouteTo::from(coordinator.clone()),
                    NodeToNode::PutReq{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::PutRsp{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::GetReq{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::GetRsp{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::SyncKey{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::PingReq{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::PingRsp{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::AddNode{ to, .. } => RouteTo::from(to.clone()),
                    NodeToNode::AddNodeAck{ to, .. } => RouteTo::from(to.clone()),
                }
            }
            DynamoNodeOut::NodeToClient(c) => {
                match c {
                    NodeToClient::ClientPutRsp{ client_addr, .. } => RouteTo::from(client_addr.clone()),
                    NodeToClient::ClientGetRsp{ client_addr, .. } => RouteTo::from(client_addr.clone()),
                }
            }
        }
    }
}

pub async fn node_behaviour(
    ctx: RuntimeCtx,
    node_id: String,
    nodes: Vec<String>,
    n: usize,
    w: usize,
    r: usize,
    t: usize,
    decoder: reactor_actor::SubDecoderStore<DynamoNodeIn>,
) {
    let proc = DynamoNode::new(node_id.clone(), nodes.clone(), n, w, r, t);
    BehaviourBuilder::new(proc, BincodeCodec::default())
        .send(DynamoNodeSender::new())
        .sub_decoders(decoder)
        .ask_receiver_to_adapt()
        .build()
        .run(ctx)
        .await
        .unwrap();
}
