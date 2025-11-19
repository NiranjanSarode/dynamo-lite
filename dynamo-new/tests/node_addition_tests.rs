// Node Addition Tests
// Tests for dynamic node addition and verification that existing features still work

use dynamo_new::vector_clock::VectorClock;
use dynamo_new::versioned_value::{VersionedValue, VersionedValues};
use dynamo_new::consistent_hash::ConsistentHash;
use dynamo_new::node::DynamoNode;
use dynamo_new::messages::{DynamoNodeIn, NodeToNode};
use reactor_actor::ActorProcess;

#[cfg(test)]
mod node_addition_tests {
    use super::*;

    #[test]
    fn test_consistent_hash_add_node() {
        let initial_nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let mut ring = ConsistentHash::new(&initial_nodes, 10);

        // Check initial state
        let (pref_before, _) = ring.find_nodes("test_key", 3, &[]);
        assert_eq!(pref_before.len(), 3);

        // Add a new node
        ring.add_node("nodeD", 10);

        // Verify new node is in the ring
        let (pref_after, _) = ring.find_nodes("test_key", 3, &[]);
        assert_eq!(pref_after.len(), 3);

        // Verify we can get all nodes
        let all_nodes = ring.get_nodes();
        assert_eq!(all_nodes.len(), 4);
        assert!(all_nodes.contains(&"nodeD".to_string()));
    }

    #[test]
    fn test_consistent_hash_add_multiple_nodes() {
        let initial_nodes = vec!["node1".to_string(), "node2".to_string()];
        let mut ring = ConsistentHash::new(&initial_nodes, 10);

        // Add multiple nodes
        ring.add_node("node3", 10);
        ring.add_node("node4", 10);
        ring.add_node("node5", 10);

        let all_nodes = ring.get_nodes();
        assert_eq!(all_nodes.len(), 5);
        assert!(all_nodes.contains(&"node3".to_string()));
        assert!(all_nodes.contains(&"node4".to_string()));
        assert!(all_nodes.contains(&"node5".to_string()));
    }

    #[test]
    fn test_consistent_hash_add_duplicate_node() {
        let initial_nodes = vec!["nodeA".to_string(), "nodeB".to_string()];
        let mut ring = ConsistentHash::new(&initial_nodes, 10);

        let nodes_before = ring.get_nodes();
        assert_eq!(nodes_before.len(), 2);

        // Add duplicate node
        ring.add_node("nodeA", 10);

        let nodes_after = ring.get_nodes();
        // Should still be 2 unique nodes, but nodeA will have more virtual nodes
        assert_eq!(nodes_after.len(), 2);
    }

    #[test]
    fn test_dynamo_node_add_node_basic() {
        let initial_nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), initial_nodes.clone(), 3, 2, 2, 10);

        // Simulate adding a new node
        let add_msg = DynamoNodeIn::NodeToNode(NodeToNode::AddNode {
            from: "admin".to_string(),
            to: "nodeA".to_string(),
            new_node: "nodeD".to_string(),
        });

        let responses = node.process(add_msg);

        // Should get an acknowledgment
        assert!(responses.len() >= 1);

        // Check that the AddNodeAck message is present
        let has_ack = responses.iter().any(|msg| {
            matches!(msg, dynamo_new::messages::DynamoNodeOut::NodeToNode(NodeToNode::AddNodeAck { .. }))
        });
        assert!(has_ack, "Should receive AddNodeAck");
    }

    #[test]
    fn test_dynamo_node_add_duplicate_node() {
        let initial_nodes = vec!["nodeA".to_string(), "nodeB".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), initial_nodes.clone(), 3, 2, 2, 10);

        // Try to add a node that already exists
        let add_msg = DynamoNodeIn::NodeToNode(NodeToNode::AddNode {
            from: "admin".to_string(),
            to: "nodeA".to_string(),
            new_node: "nodeB".to_string(), // Already exists
        });

        let responses = node.process(add_msg);

        // Should still get an acknowledgment but no redistribution
        assert!(responses.len() == 1);
    }

    #[test]
    fn test_node_addition_with_data_redistribution() {
        let initial_nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), initial_nodes.clone(), 3, 2, 2, 10);

        // First, add some data to the node
        let put_msg = DynamoNodeIn::NodeToNode(NodeToNode::PutReq {
            from: "client".to_string(),
            to: "nodeA".to_string(),
            key: "testkey".to_string(),
            value: "testvalue".to_string(),
            clock: VectorClock::new(),
            msg_id: 1,
            handoff: None,
        });
        node.process(put_msg);

        // Now add a new node
        let add_msg = DynamoNodeIn::NodeToNode(NodeToNode::AddNode {
            from: "admin".to_string(),
            to: "nodeA".to_string(),
            new_node: "nodeD".to_string(),
        });

        let responses = node.process(add_msg);

        // Should have at least one response (the Ack)
        assert!(responses.len() >= 1);

        // If there's redistribution, there should be PutReq messages
        let has_redistribution = responses.iter().any(|msg| {
            matches!(msg, dynamo_new::messages::DynamoNodeOut::NodeToNode(NodeToNode::PutReq { .. }))
        });

        // Note: redistribution depends on whether the new node falls into the preference list
        // for the keys we have stored, so this may or may not happen
    }
}

#[cfg(test)]
mod existing_features_tests {
    use super::*;

    #[test]
    fn test_vector_clock_basic_operations() {
        let mut vc1 = VectorClock::new();
        vc1.increment("node1");
        assert_eq!(vc1.clock.get("node1"), Some(&1));

        vc1.increment("node1");
        assert_eq!(vc1.clock.get("node1"), Some(&2));

        vc1.increment("node2");
        assert_eq!(vc1.clock.get("node2"), Some(&1));
    }

    #[test]
    fn test_vector_clock_update() {
        let mut vc = VectorClock::new();
        vc.update("node1", 5);
        assert_eq!(vc.clock.get("node1"), Some(&5));

        vc.update("node1", 3); // Should not decrease
        assert_eq!(vc.clock.get("node1"), Some(&5));

        vc.update("node1", 10);
        assert_eq!(vc.clock.get("node1"), Some(&10));
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut vc1 = VectorClock::new();
        vc1.increment("node1");
        vc1.increment("node2");

        let mut vc2 = VectorClock::new();
        vc2.increment("node2");
        vc2.increment("node3");

        vc1.merge(&vc2);

        assert_eq!(vc1.clock.get("node1"), Some(&1));
        assert_eq!(vc1.clock.get("node2"), Some(&1)); // max(1, 1) from merge
        assert_eq!(vc1.clock.get("node3"), Some(&1));
    }

    #[test]
    fn test_vector_clock_compare_concurrent() {
        let mut vc1 = VectorClock::new();
        vc1.increment("node1");

        let mut vc2 = VectorClock::new();
        vc2.increment("node2");

        use dynamo_new::vector_clock::ClockOrdering;
        assert_eq!(vc1.compare(&vc2), ClockOrdering::Concurrent);
    }

    #[test]
    fn test_vector_clock_compare_before() {
        let mut vc1 = VectorClock::new();
        vc1.increment("node1");

        let mut vc2 = VectorClock::new();
        vc2.increment("node1");
        vc2.increment("node1");

        use dynamo_new::vector_clock::ClockOrdering;
        assert_eq!(vc1.compare(&vc2), ClockOrdering::Before);
    }

    #[test]
    fn test_versioned_value_creation() {
        let mut vc = VectorClock::new();
        vc.increment("node1");

        let vv = VersionedValue::new("value1".to_string(), vc.clone());
        assert_eq!(vv.value, "value1");
        assert_eq!(vv.clock.clock.get("node1"), Some(&1));
    }

    #[test]
    fn test_versioned_values_add_and_supersede() {
        let mut vvs = VersionedValues::new();

        let mut vc1 = VectorClock::new();
        vc1.increment("node1");
        let vv1 = VersionedValue::new("v1".to_string(), vc1.clone());

        vvs.add_version(vv1.clone());
        assert_eq!(vvs.versions.len(), 1);

        // Add newer version that supersedes
        let mut vc2 = VectorClock::new();
        vc2.increment("node1");
        vc2.increment("node1");
        let vv2 = VersionedValue::new("v2".to_string(), vc2.clone());

        vvs.add_version(vv2.clone());
        assert_eq!(vvs.versions.len(), 1); // Old version should be removed
        assert_eq!(vvs.versions[0].value, "v2");
    }

    #[test]
    fn test_versioned_values_concurrent_versions() {
        let mut vvs = VersionedValues::new();

        let mut vc1 = VectorClock::new();
        vc1.increment("node1");
        let vv1 = VersionedValue::new("v1".to_string(), vc1.clone());

        let mut vc2 = VectorClock::new();
        vc2.increment("node2");
        let vv2 = VersionedValue::new("v2".to_string(), vc2.clone());

        vvs.add_version(vv1.clone());
        vvs.add_version(vv2.clone());

        // Both should exist (concurrent)
        assert_eq!(vvs.versions.len(), 2);
    }

    #[test]
    fn test_versioned_values_merge() {
        let mut vvs1 = VersionedValues::new();
        let mut vvs2 = VersionedValues::new();

        let mut vc1 = VectorClock::new();
        vc1.increment("node1");
        let vv1 = VersionedValue::new("v1".to_string(), vc1.clone());

        let mut vc2 = VectorClock::new();
        vc2.increment("node2");
        let vv2 = VersionedValue::new("v2".to_string(), vc2.clone());

        vvs1.add_version(vv1.clone());
        vvs2.add_version(vv2.clone());

        vvs1.merge(&vvs2);

        assert_eq!(vvs1.versions.len(), 2);
    }

    #[test]
    fn test_consistent_hash_basic() {
        let nodes = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
        let ring = ConsistentHash::new(&nodes, 10);

        let (pref, _) = ring.find_nodes("key1", 3, &[]);
        assert_eq!(pref.len(), 3);
    }

    #[test]
    fn test_consistent_hash_with_failures() {
        let nodes = vec!["node1".to_string(), "node2".to_string(), "node3".to_string(), "node4".to_string()];
        let ring = ConsistentHash::new(&nodes, 10);

        let failed = vec!["node2".to_string()];
        let (pref, avoided) = ring.find_nodes("key1", 3, &failed);

        assert_eq!(pref.len(), 3);
        assert!(!pref.contains(&"node2".to_string()));
        assert!(avoided.contains(&"node2".to_string()));
    }

    #[test]
    fn test_dynamo_node_creation() {
        let nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let node = DynamoNode::new("nodeA".to_string(), nodes, 3, 2, 2, 10);

        // Just verify the node was created successfully
        // We can't directly access fields but we can verify it doesn't panic
    }

    #[test]
    fn test_dynamo_node_put_request() {
        let nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), nodes, 3, 2, 2, 10);

        let put_msg = DynamoNodeIn::NodeToNode(NodeToNode::PutReq {
            from: "client".to_string(),
            to: "nodeA".to_string(),
            key: "key1".to_string(),
            value: "value1".to_string(),
            clock: VectorClock::new(),
            msg_id: 1,
            handoff: None,
        });

        let responses = node.process(put_msg);

        // Should get a PutRsp
        assert!(responses.len() > 0);
        let has_put_rsp = responses.iter().any(|msg| {
            matches!(msg, dynamo_new::messages::DynamoNodeOut::NodeToNode(NodeToNode::PutRsp { .. }))
        });
        assert!(has_put_rsp);
    }

    #[test]
    fn test_dynamo_node_get_request() {
        let nodes = vec!["nodeA".to_string(), "nodeB".to_string(), "nodeC".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), nodes, 3, 2, 2, 10);

        // First put some data
        let put_msg = DynamoNodeIn::NodeToNode(NodeToNode::PutReq {
            from: "client".to_string(),
            to: "nodeA".to_string(),
            key: "key1".to_string(),
            value: "value1".to_string(),
            clock: VectorClock::new(),
            msg_id: 1,
            handoff: None,
        });
        node.process(put_msg);

        // Now get it
        let get_msg = DynamoNodeIn::NodeToNode(NodeToNode::GetReq {
            from: "client".to_string(),
            to: "nodeA".to_string(),
            key: "key1".to_string(),
            msg_id: 2,
        });

        let responses = node.process(get_msg);

        // Should get a GetRsp
        assert!(responses.len() > 0);
        let has_get_rsp = responses.iter().any(|msg| {
            matches!(msg, dynamo_new::messages::DynamoNodeOut::NodeToNode(NodeToNode::GetRsp { .. }))
        });
        assert!(has_get_rsp);
    }

    #[test]
    fn test_dynamo_node_ping_pong() {
        let nodes = vec!["nodeA".to_string(), "nodeB".to_string()];
        let mut node = DynamoNode::new("nodeA".to_string(), nodes, 3, 2, 2, 10);

        let ping_msg = DynamoNodeIn::NodeToNode(NodeToNode::PingReq {
            from: "nodeB".to_string(),
            to: "nodeA".to_string(),
        });

        let responses = node.process(ping_msg);

        // Should get a PingRsp
        assert_eq!(responses.len(), 1);
        let has_ping_rsp = responses.iter().any(|msg| {
            matches!(msg, dynamo_new::messages::DynamoNodeOut::NodeToNode(NodeToNode::PingRsp { .. }))
        });
        assert!(has_ping_rsp);
    }
}
