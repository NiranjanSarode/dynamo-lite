use std::collections::{BTreeMap};
use serde::{Deserialize, Serialize};
use rand::prelude::IndexedRandom;
use log::info;
use reactor_actor::{ActorProcess, ActorSend, BehaviourBuilder, RouteTo, RuntimeCtx};
use reactor_actor::codec::BincodeCodec;

use crate::messages::{DynamoClientIn, DynamoClientOut, GeneratorTrigger, ClientToNode, NodeToClient};
use crate::vector_clock::VectorClock;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Cart { items: BTreeMap<String, u32> }

impl Cart {
    fn from_json(s: &str) -> Option<Cart> { serde_json::from_str(s).ok() }
    fn to_json(&self) -> String { serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string()) }
    fn merge_many(vals: &[String]) -> Cart {
        let mut res = Cart::default();
        for v in vals {
            if let Some(c) = Cart::from_json(v) {
                for (sku, qty) in c.items {
                    *res.items.entry(sku).or_insert(0) += qty;
                }
            }
        }
        res
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag="op", rename_all="lowercase")]
pub enum CartStep {
    Add { cart: String, sku: String, qty: u32 },
    Remove { cart: String, sku: String, qty: u32 },
    Get { cart: String },
}

pub struct CartClient {
    client_id: String,
    nodes: Vec<String>,
    steps: Vec<CartStep>,
    next: usize,
    inflight: bool,
    last_metadata: Vec<VectorClock>,
    local_vc: VectorClock,
}

impl CartClient {
    pub fn new(client_id: String, nodes: Vec<String>, steps: Vec<CartStep>) -> Self {
        Self { client_id, nodes, steps, next: 0, inflight: false, last_metadata: vec![], local_vc: VectorClock::new() }
    }
}

impl ActorProcess for CartClient {
    type IMsg = DynamoClientIn;
    type OMsg = DynamoClientOut;

    fn process(&mut self, input: Self::IMsg) -> Vec<Self::OMsg> {
        match input {
            DynamoClientIn::GeneratorTrigger(_) => {
                if self.inflight || self.next >= self.steps.len() { return vec![]; }
                // Always begin with a GET for Add/Remove to implement read-before-write
                match &self.steps[self.next] {
                    CartStep::Get { cart } | CartStep::Add { cart, .. } | CartStep::Remove { cart, .. } => {
                        let req_id = (self.next as u64) * 2 + 1;
                        self.inflight = true;
                        info!("[cart] {} sending GET cart={} req_id={}", self.client_id, cart, req_id);
                        vec![DynamoClientOut::ClientToNode(ClientToNode::ClientGet{ key: format!("cart:{}", cart), client_addr: self.client_id.clone(), request_id: req_id })]
                    }
                }
            }
            DynamoClientIn::NodeToClient(resp) => {
                match resp {
                    NodeToClient::ClientGetRsp{ key, request_id, values, metadata, .. } => {
                        info!("[cart] {} GetOk key={} req_id={} versions={}", self.client_id, key, request_id, metadata.len());
                        // Build merged cart from siblings
                        let mut cart = Cart::merge_many(&values);
                        self.last_metadata = metadata;
                        // If this step is Add/Remove, compute new cart and PUT
                        match &self.steps[self.next] {
                            CartStep::Get { cart: _ } => {
                                // Just a read; advance
                                self.inflight = false;
                                self.next += 1;
                                if self.next < self.steps.len() { return vec![DynamoClientOut::ClientToNode(ClientToNode::ClientGet{ key: format!("cart:{}", match &self.steps[self.next] { CartStep::Get{cart}|CartStep::Add{cart,..}|CartStep::Remove{cart,..} => cart.clone() }), client_addr: self.client_id.clone(), request_id: (self.next as u64)*2 + 1 })]; }
                                return vec![];
                            }
                            CartStep::Add { cart: id, sku, qty } => {
                                *cart.items.entry(sku.clone()).or_insert(0) += *qty;
                                // PUT back
                                let mut base = if !self.last_metadata.is_empty() { VectorClock::converge(self.last_metadata.clone()) } else { self.local_vc.clone() };
                                base.increment(&self.client_id);
                                self.local_vc = base.clone();
                                let val = cart.to_json();
                                let req_id = (self.next as u64) * 2 + 2;
                                info!("[cart] {} PUT cart={} sku={} qty={} req_id={}", self.client_id, id, sku, qty, req_id);
                                return vec![DynamoClientOut::ClientToNode(ClientToNode::ClientPut{ key: format!("cart:{}", id), value: val, metadata: vec![base], client_addr: self.client_id.clone(), request_id: req_id })];
                            }
                            CartStep::Remove { cart: id, sku, qty } => {
                                let entry = cart.items.entry(sku.clone()).or_insert(0);
                                *entry = entry.saturating_sub(*qty);
                                if *entry == 0 { cart.items.remove(sku); }
                                let mut base = if !self.last_metadata.is_empty() { VectorClock::converge(self.last_metadata.clone()) } else { self.local_vc.clone() };
                                base.increment(&self.client_id);
                                self.local_vc = base.clone();
                                let val = cart.to_json();
                                let req_id = (self.next as u64) * 2 + 2;
                                info!("[cart] {} PUT cart={} remove sku={} qty={} req_id={}", self.client_id, id, sku, qty, req_id);
                                return vec![DynamoClientOut::ClientToNode(ClientToNode::ClientPut{ key: format!("cart:{}", id), value: val, metadata: vec![base], client_addr: self.client_id.clone(), request_id: req_id })];
                            }
                        }
                    }
                    NodeToClient::ClientPutRsp{ key, request_id, .. } => {
                        info!("[cart] {} PutOk key={} req_id={}", self.client_id, key, request_id);
                        // Finished one step (which required GET then PUT)
                        self.inflight = false;
                        self.next += 1;
                        // Trigger next cycle (which will start with a GET)
                        if self.next < self.steps.len() {
                            let next_cart = match &self.steps[self.next] { CartStep::Get{cart}|CartStep::Add{cart,..}|CartStep::Remove{cart,..} => cart.clone() };
                            return vec![DynamoClientOut::ClientToNode(ClientToNode::ClientGet{ key: format!("cart:{}", next_cart), client_addr: self.client_id.clone(), request_id: (self.next as u64)*2 + 1 })];
                        }
                        return vec![];
                    }
                }
            }
        }
    }
}

pub struct CartClientSender { nodes: Vec<String> }
impl CartClientSender { pub fn new(nodes: Vec<String>) -> Self { Self{ nodes } } }
impl ActorSend for CartClientSender {
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

pub async fn cart_client_behaviour(
    ctx: RuntimeCtx,
    client_id: String,
    nodes: Vec<String>,
    steps: Vec<CartStep>,
    decoder: reactor_actor::SubDecoderStore<DynamoClientIn>,
) {
    let proc = CartClient::new(client_id.clone(), nodes.clone(), steps);
    // drive with a single trigger; the client chains GET->(PUT?) per step
    BehaviourBuilder::new(proc, BincodeCodec::default())
        .send(CartClientSender::new(nodes))
        .sub_decoders(decoder)
        .ask_receiver_to_adapt()
        .generator_if(true, move || { vec![DynamoClientIn::GeneratorTrigger(GeneratorTrigger)].into_iter() })
        .build()
        .run(ctx)
        .await
        .unwrap();
}
