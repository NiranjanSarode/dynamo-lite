pub mod messages;
pub mod vector_clock;
pub mod versioned_value;
pub mod consistent_hash;
pub mod node;
mod client;
mod cart_client;
mod bench_client;

pub use reactor_actor::setup_shared_logger_ref;

use reactor_actor::RuntimeCtx;
use reactor_actor::actor;
use std::collections::HashMap;

use messages::{DynamoNodeIn, DynamoClientIn, ClientToNode, NodeToNode, NodeToClient, DynamoClientOut, DynamoNodeOut};
use cart_client::CartStep;
use reactor_macros::msg_converter;

msg_converter! {
    Decoders: [
        // Receivers should be able to adapt from sender OUT union types
        // Node accepts messages from clients and other nodes
        dynamo_node_decoder can decode DynamoClientOut, DynamoNodeOut to DynamoNodeIn;
        // Client accepts messages from nodes
        dynamo_client_decoder can decode DynamoNodeOut to DynamoClientIn;
    ];
}

lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

#[actor]
fn dynamo_node(ctx: RuntimeCtx, mut payload: HashMap<String, serde_json::Value>) {
    let node_id = payload.remove("node_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| ctx.addr.to_string());
    let nodes: Vec<String> = payload.remove("nodes").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    let n = payload.remove("N").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
    let w = payload.remove("W").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
    let r = payload.remove("R").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
    let t = payload.remove("T").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    RUNTIME.spawn(node::node_behaviour(ctx, node_id, nodes, n, w, r, t, dynamo_node_decoder));
}

#[actor]
fn dynamo_client(ctx: RuntimeCtx, mut payload: HashMap<String, serde_json::Value>) {
    let client_id = payload.remove("client_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| ctx.addr.to_string());
    let nodes: Vec<String> = payload.remove("nodes").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    // Optional script: array of { op: "put"|"get", key: String, value?: String }
    let script_val = payload.remove("script");
    let mut reqs: Vec<ClientToNode> = vec![];
    if let Some(serde_json::Value::Array(arr)) = script_val {
        log::info!("[client-init] {} using provided script steps={} ", client_id, arr.len());
        let mut rid: u64 = 1;
        for item in arr {
            if let Some(obj) = item.as_object() {
                let op = obj.get("op").and_then(|v| v.as_str()).unwrap_or("");
                let key = obj.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                match op {
                    "put" => {
                        let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        reqs.push(ClientToNode::ClientPut{ key, value, metadata: vec![], client_addr: client_id.clone(), request_id: rid });
                        rid += 1;
                    }
                    "get" => {
                        reqs.push(ClientToNode::ClientGet{ key, client_addr: client_id.clone(), request_id: rid });
                        rid += 1;
                    }
                    _ => {}
                }
            }
        }
        // brief preview of first few steps for debugging
        let preview: Vec<String> = reqs.iter().take(3).map(|r| match r {
            ClientToNode::ClientPut{ key, value, .. } => format!("put {}:{}", key, value),
            ClientToNode::ClientGet{ key, .. } => format!("get {}", key),
        }).collect();
        log::info!("[client-init] {} script preview: {:?}", client_id, preview);
    } else {
        // default simple script
        log::warn!("[client-init] {} no script provided; using default demo script", client_id);
        let mut rid: u64 = 1;
        reqs.push(ClientToNode::ClientPut { key: "user:1".into(), value: "Alice".into(), metadata: vec![], client_addr: client_id.clone(), request_id: rid });
        rid += 1;
        reqs.push(ClientToNode::ClientGet { key: "user:1".into(), client_addr: client_id.clone(), request_id: rid });
        rid += 1;
        reqs.push(ClientToNode::ClientPut { key: "user:1".into(), value: "Alice_Updated".into(), metadata: vec![], client_addr: client_id.clone(), request_id: rid });
        rid += 1;
        reqs.push(ClientToNode::ClientGet { key: "user:1".into(), client_addr: client_id.clone(), request_id: rid });
    }
    RUNTIME.spawn(client::client_behaviour(ctx, client_id, nodes, reqs, dynamo_client_decoder));
}

#[actor]
fn dynamo_cart_client(ctx: RuntimeCtx, mut payload: HashMap<String, serde_json::Value>) {
    let client_id = payload.remove("client_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| ctx.addr.to_string());
    let nodes: Vec<String> = payload.remove("nodes").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    // Strictly typed script steps
    let steps: Vec<CartStep> = payload.remove("script").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    log::info!("[cart-init] {} steps={} first={:?}", client_id, steps.len(), steps.get(0));
    RUNTIME.spawn(cart_client::cart_client_behaviour(ctx, client_id, nodes, steps, dynamo_client_decoder));
}

#[actor]
fn dynamo_bench_client(ctx: RuntimeCtx, mut payload: HashMap<String, serde_json::Value>) {
    let client_id = payload.remove("client_id").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| ctx.addr.to_string());
    let nodes: Vec<String> = payload.remove("nodes").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    let num_ops = payload.remove("num_ops").and_then(|v| v.as_u64()).unwrap_or(1000) as usize;
    log::info!("[bench-init] {} running {} operations", client_id, num_ops);
    RUNTIME.spawn(bench_client::bench_client_behaviour(ctx, client_id, nodes, num_ops, dynamo_client_decoder));
}
