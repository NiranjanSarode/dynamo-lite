use bincode::{Decode, Encode};
use reactor_macros::{DefaultPrio, Msg as DeriveMsg, msg_converter};
use crate::vector_clock::VectorClock;
use crate::versioned_value::VersionedValues;

#[derive(Debug, Clone, Encode, Decode, DefaultPrio, DeriveMsg)]
pub struct GeneratorTrigger;

// Client -> Node
#[derive(Debug, Clone, Encode, Decode, DefaultPrio, DeriveMsg)]
pub enum ClientToNode {
    ClientPut { key: String, value: String, metadata: Vec<VectorClock>, client_addr: String, request_id: u64 },
    ClientGet { key: String, client_addr: String, request_id: u64 },
}

// Node -> Client
#[derive(Debug, Clone, Encode, Decode, DefaultPrio, DeriveMsg)]
pub enum NodeToClient {
    ClientPutRsp { key: String, request_id: u64, client_addr: String },
    ClientGetRsp { key: String, request_id: u64, values: Vec<String>, metadata: Vec<VectorClock>, client_addr: String },
}

// Node <-> Node
#[derive(Debug, Clone, Encode, Decode, DefaultPrio, DeriveMsg)]
pub enum NodeToNode {
    ForwardClientPut { coordinator: String, key: String, value: String, metadata: Vec<VectorClock>, client_addr: String, request_id: u64 },
    ForwardClientGet { coordinator: String, key: String, client_addr: String, request_id: u64 },

    PutReq { from: String, to: String, key: String, value: String, clock: VectorClock, msg_id: u64, handoff: Option<Vec<String>> },
    PutRsp { from: String, to: String, msg_id: u64 },
    GetReq { from: String, to: String, key: String, msg_id: u64 },
    GetRsp { from: String, to: String, key: String, values: VersionedValues, msg_id: u64 },

    // background anti-entropy: push local view of a key to a replica for merge
    SyncKey { from: String, to: String, key: String, values: VersionedValues },

    PingReq { from: String, to: String },
    PingRsp { from: String, to: String },
}

msg_converter! {
    Unions: [
        DynamoNodeIn = ClientToNode, NodeToNode;
        DynamoNodeOut = NodeToNode, NodeToClient;
        DynamoClientIn = NodeToClient, GeneratorTrigger;
        DynamoClientOut = ClientToNode;
    ];
    Adapters: [
        DynamoNodeIn from DynamoClientOut via ClientToNode;
        DynamoNodeIn from DynamoNodeOut via NodeToNode;
        DynamoClientIn from DynamoNodeOut via NodeToClient;
    ];
}
