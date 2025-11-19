use std::time::Instant;
use std::fs::OpenOptions;
use std::io::Write;
use log::info;
use reactor_actor::{ActorProcess, ActorSend, BehaviourBuilder, RouteTo, RuntimeCtx};
use reactor_actor::codec::BincodeCodec;

use crate::messages::{DynamoClientIn, DynamoClientOut, GeneratorTrigger, ClientToNode, NodeToClient};
use crate::vector_clock::VectorClock;

/// Benchmark client that measures GET/PUT latencies
/// Writes directly to latency.log file
pub struct BenchClient {
    client_id: String,
    nodes: Vec<String>,
    num_ops: usize,
    current_op: usize,
    inflight_key: Option<String>,
    inflight_start: Option<Instant>,
    inflight_type: Option<String>, // "GET" or "PUT"
    local_vc: VectorClock,
}

impl BenchClient {
    pub fn new(client_id: String, nodes: Vec<String>, num_ops: usize) -> Self {
        Self {
            client_id,
            nodes,
            num_ops,
            current_op: 0,
            inflight_key: None,
            inflight_start: None,
            inflight_type: None,
            local_vc: VectorClock::new(),
        }
    }

    fn next_operation(&mut self) -> Option<ClientToNode> {
        if self.current_op >= self.num_ops {
            return None;
        }

        let key = format!("key{}", self.current_op % 100);
        let op_type = if self.current_op % 2 == 0 { "PUT" } else { "GET" };

        self.inflight_key = Some(key.clone());
        self.inflight_start = Some(Instant::now());
        self.inflight_type = Some(op_type.to_string());
        self.current_op += 1;

        if op_type == "PUT" {
            let value = format!("value_{}", self.current_op);
            self.local_vc.increment(&self.client_id);
            Some(ClientToNode::ClientPut {
                key,
                value,
                request_id: self.current_op as u64,
                metadata: vec![self.local_vc.clone()],
                client_addr: self.client_id.clone(),
            })
        } else {
            Some(ClientToNode::ClientGet {
                key,
                request_id: self.current_op as u64,
                client_addr: self.client_id.clone(),
            })
        }
    }

    fn record_latency(&mut self) {
        if let (Some(key), Some(start), Some(op_type)) =
            (&self.inflight_key, self.inflight_start, &self.inflight_type)
        {
            let latency = start.elapsed();
            let latency_ms = latency.as_secs_f64() * 1000.0;

            // Write directly to latency.log file for reliable capture
            let log_line = format!("{} key={} latency={:.2}ms\n", op_type, key, latency_ms);

            // Also log via info! for console output
            info!("{} key={} latency={:.2}ms", op_type, key, latency_ms);

            // Write to file
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("latency.log")
            {
                let _ = file.write_all(log_line.as_bytes());
            }

            self.inflight_key = None;
            self.inflight_start = None;
            self.inflight_type = None;
        }
    }
}

impl ActorProcess for BenchClient {
    type IMsg = DynamoClientIn;
    type OMsg = DynamoClientOut;

    fn process(&mut self, input: Self::IMsg) -> Vec<Self::OMsg> {
        match input {
            DynamoClientIn::GeneratorTrigger(_) => {
                // Start first operation
                if let Some(req) = self.next_operation() {
                    vec![DynamoClientOut::ClientToNode(req)]
                } else {
                    vec![]
                }
            }
            DynamoClientIn::NodeToClient(resp) => {
                // Record latency for completed operation
                self.record_latency();

                // Process response
                match resp {
                    NodeToClient::ClientPutRsp { .. } => {}
                    NodeToClient::ClientGetRsp { metadata, .. } => {
                        // Update local clock with returned metadata
                        if !metadata.is_empty() {
                            self.local_vc = VectorClock::converge(metadata);
                        }
                    }
                }

                // Issue next operation
                if let Some(req) = self.next_operation() {
                    vec![DynamoClientOut::ClientToNode(req)]
                } else {
                    info!("[bench_client] Completed {} operations", self.num_ops);
                    vec![]
                }
            }
        }
    }
}

pub struct BenchClientSender {
    nodes: Vec<String>,
    current: usize,
}

impl BenchClientSender {
    pub fn new(nodes: Vec<String>) -> Self {
        Self { nodes, current: 0 }
    }
}

impl ActorSend for BenchClientSender {
    type OMsg = DynamoClientOut;

    async fn before_send<'a>(&'a mut self, out: &Self::OMsg) -> RouteTo<'a> {
        match out {
            DynamoClientOut::ClientToNode(_) => {
                // Round-robin load balancing
                let node = &self.nodes[self.current % self.nodes.len()];
                self.current += 1;
                RouteTo::from(node.as_str())
            }
        }
    }
}

pub async fn bench_client_behaviour(
    ctx: RuntimeCtx,
    client_id: String,
    nodes: Vec<String>,
    num_ops: usize,
    decoder: reactor_actor::SubDecoderStore<DynamoClientIn>,
) {
    info!("[bench_client] Starting benchmark: {} operations", num_ops);

    let proc = BenchClient::new(client_id, nodes.clone(), num_ops);
    BehaviourBuilder::new(proc, BincodeCodec::default())
        .send(BenchClientSender::new(nodes))
        .sub_decoders(decoder)
        .ask_receiver_to_adapt()
        .generator_if(true, || vec![DynamoClientIn::GeneratorTrigger(GeneratorTrigger)].into_iter())
        .build()
        .run(ctx)
        .await
        .unwrap();
}
