use rand::prelude::IndexedRandom;
use log::info;
use reactor_actor::{ActorProcess, ActorSend, BehaviourBuilder, RouteTo, RuntimeCtx};
use reactor_actor::codec::BincodeCodec;

use crate::messages::{DynamoClientIn, DynamoClientOut, GeneratorTrigger, ClientToNode, NodeToClient};
use crate::vector_clock::VectorClock;

pub struct DynamoClient {
    client_id: String,
    nodes: Vec<String>,
    reqs: Vec<ClientToNode>,
    next: usize,
    last_metadata: Vec<VectorClock>,
    local_vc: VectorClock,
    inflight: bool,
}

impl DynamoClient {
    pub fn new(client_id: String, nodes: Vec<String>, reqs: Vec<ClientToNode>) -> Self {
        Self { client_id, nodes, reqs, next: 0, last_metadata: vec![], local_vc: VectorClock::new(), inflight: false }
    }
}

impl ActorProcess for DynamoClient {
    type IMsg = DynamoClientIn;
    type OMsg = DynamoClientOut;

    fn process(&mut self, input: Self::IMsg) -> Vec<Self::OMsg> {
        match input {
            DynamoClientIn::GeneratorTrigger(_) => {
                // Send only if nothing is inflight; otherwise drop trigger
                if self.inflight || self.next >= self.reqs.len() { return vec![]; }
                let mut req = self.reqs[self.next].clone();
                if let ClientToNode::ClientPut{ ref mut metadata, .. } = req {
                    let mut base = if !self.last_metadata.is_empty() { VectorClock::converge(self.last_metadata.clone()) } else { self.local_vc.clone() };
                    base.increment(&self.client_id);
                    *metadata = vec![base.clone()];
                    self.local_vc = base;
                }
                self.inflight = true;
                info!("[client] sending {:?}", &req);
                vec![DynamoClientOut::ClientToNode(req)]
            }
            DynamoClientIn::NodeToClient(resp) => {
                match resp {
                    NodeToClient::ClientPutRsp{ key, request_id, .. } => {
                        info!("[client] PutOk key={} req_id={}", key, request_id);
                    }
                    NodeToClient::ClientGetRsp{ key, request_id, values, metadata, .. } => {
                        // Zip values with their clocks for clearer debugging
                        let pairs: Vec<String> = values.iter().zip(metadata.iter()).map(|(v, vc)| format!("({},{:?})", v, vc.clock)).collect();
                        info!("[client] GetOk key={} req_id={} versions={} entries={:?}", key, request_id, metadata.len(), pairs);
                        self.last_metadata = metadata;
                    }
                }
                // Chain next request after a response
                if self.next + 1 <= self.reqs.len() {
                    self.next += 1;
                }
                self.inflight = false;
                // Emit next request immediately if any remain
                if self.next < self.reqs.len() {
                    let mut req = self.reqs[self.next].clone();
                    if let ClientToNode::ClientPut{ ref mut metadata, .. } = req {
                        let mut base = if !self.last_metadata.is_empty() { VectorClock::converge(self.last_metadata.clone()) } else { self.local_vc.clone() };
                        base.increment(&self.client_id);
                        *metadata = vec![base.clone()];
                        self.local_vc = base;
                    }
                    self.inflight = true;
                    info!("[client] sending {:?}", &req);
                    vec![DynamoClientOut::ClientToNode(req)]
                } else {
                    vec![]
                }
            }
        }
    }
}

pub struct DynamoClientSender { nodes: Vec<String> }
impl DynamoClientSender { pub fn new(nodes: Vec<String>) -> Self { Self{ nodes } } }
impl ActorSend for DynamoClientSender {
    type OMsg = DynamoClientOut;
    async fn before_send<'a>(&'a mut self, out: &Self::OMsg) -> RouteTo<'a> {
        match out {
            DynamoClientOut::ClientToNode(ClientToNode::ClientPut{ .. }) | DynamoClientOut::ClientToNode(ClientToNode::ClientGet{ .. }) => {
                let mut rng = rand::rng();
                if let Some(node) = self.nodes.choose(&mut rng) { RouteTo::from(node.as_str()) } else { RouteTo::Reply }
            }
        }
    }
}

pub async fn client_behaviour(
    ctx: RuntimeCtx,
    client_id: String,
    nodes: Vec<String>,
    reqs: Vec<ClientToNode>,
    decoder: reactor_actor::SubDecoderStore<DynamoClientIn>,
) {
    let reqs_len = 1usize; // single trigger to kick off serialized flow
    let proc = DynamoClient::new(client_id.clone(), nodes.clone(), reqs);
    BehaviourBuilder::new(proc, BincodeCodec::default())
        .send(DynamoClientSender::new(nodes))
        .sub_decoders(decoder)
        .ask_receiver_to_adapt()
        .generator_if(true, move || { vec![DynamoClientIn::GeneratorTrigger(GeneratorTrigger); reqs_len].into_iter() })
        .build()
        .run(ctx)
        .await
        .unwrap();
}
