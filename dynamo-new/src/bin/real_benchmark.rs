/// Real Dynamo Benchmark - Direct Logic Execution
///
/// This benchmark uses ACTUAL Dynamo business logic:
/// - Real ConsistentHash for key routing
/// - Real VectorClock operations
/// - Real VersionedValue storage and conflict resolution
/// - Real quorum logic
/// - Real HashMap storage operations
///
/// All operations execute real Dynamo code, not simulations.
/// Latencies are measured from actual computation time.

use std::collections::HashMap;
use std::time::Instant;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;

use dynamo_new::vector_clock::VectorClock;
use dynamo_new::versioned_value::{VersionedValue, VersionedValues};
use dynamo_new::consistent_hash::ConsistentHash;

/// Minimal DynamoStore that uses REAL Dynamo logic
struct RealDynamoStore {
    store: HashMap<String, VersionedValues>,
    clock: VectorClock,
    node_id: String,
}

impl RealDynamoStore {
    fn new(node_id: String) -> Self {
        Self {
            store: HashMap::new(),
            clock: VectorClock::new(),
            node_id,
        }
    }

    /// Real PUT operation using actual Dynamo logic
    fn put(&mut self, key: String, value: String) {
        // Real vector clock increment
        self.clock.increment(&self.node_id);

        // Real versioned value creation
        let versioned = VersionedValue {
            value,
            clock: self.clock.clone(),
        };

        // Real storage with conflict detection
        match self.store.get_mut(&key) {
            Some(values) => {
                // Real concurrent version handling
                values.add_version(versioned);
            }
            None => {
                let mut values = VersionedValues::new();
                values.add_version(versioned);
                self.store.insert(key, values);
            }
        }
    }

    /// Real GET operation using actual Dynamo logic
    fn get(&mut self, key: &str) -> Option<VersionedValues> {
        // Real storage retrieval
        self.store.get(key).cloned()
    }

    /// Real merge operation for read repair
    fn merge(&mut self, key: String, values: VersionedValues) {
        // Real conflict resolution logic
        match self.store.get_mut(&key) {
            Some(local_values) => {
                // Real version comparison and merge
                for v in values.versions {
                    local_values.add_version(v);
                }
            }
            None => {
                self.store.insert(key, values);
            }
        }
    }
}

/// Real quorum coordinator using actual Dynamo logic
struct RealQuorumCoordinator {
    nodes: Vec<RealDynamoStore>,
    ring: ConsistentHash,
    n: usize,
    w: usize,
    r: usize,
}

impl RealQuorumCoordinator {
    fn new(node_names: Vec<String>, n: usize, w: usize, r: usize, t: usize) -> Self {
        let nodes = node_names
            .iter()
            .map(|name| RealDynamoStore::new(name.clone()))
            .collect();

        let ring = ConsistentHash::new(&node_names, t);

        Self { nodes, ring, n, w, r }
    }

    /// Real PUT with quorum write
    fn quorum_put(&mut self, key: String, value: String) {
        // Real consistent hash routing
        let (preference_list, _) = self.ring.find_nodes(&key, self.n, &[]);

        // Real quorum write to W nodes
        for i in 0..self.w.min(preference_list.len()) {
            if let Some(node_idx) = self.find_node_index(&preference_list[i]) {
                // Real PUT operation on actual node
                self.nodes[node_idx].put(key.clone(), value.clone());
            }
        }
    }

    /// Real GET with quorum read and read repair
    fn quorum_get(&mut self, key: &str) -> Option<String> {
        // Real consistent hash routing
        let (preference_list, _) = self.ring.find_nodes(key, self.n, &[]);

        let mut responses = Vec::new();

        // Real quorum read from R nodes
        for i in 0..self.r.min(preference_list.len()) {
            if let Some(node_idx) = self.find_node_index(&preference_list[i]) {
                // Real GET operation on actual node
                if let Some(values) = self.nodes[node_idx].get(key) {
                    responses.push((node_idx, values));
                }
            }
        }

        if responses.is_empty() {
            return None;
        }

        // Real version reconciliation
        let mut all_values = VersionedValues::new();
        for (_, values) in &responses {
            for v in &values.versions {
                all_values.add_version(v.clone());
            }
        }

        // Real read repair - push latest version to nodes
        for i in 0..self.w.min(preference_list.len()) {
            if let Some(node_idx) = self.find_node_index(&preference_list[i]) {
                self.nodes[node_idx].merge(key.to_string(), all_values.clone());
            }
        }

        // Return most recent value (real causality resolution)
        all_values.versions.first().map(|v| v.value.clone())
    }

    fn find_node_index(&self, node_name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.node_id == node_name)
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments: num_ops [n] [w] [r]
    let num_ops = if args.len() > 1 {
        args[1].parse().unwrap_or(1000)
    } else {
        1000
    };

    let n = if args.len() > 2 {
        args[2].parse().unwrap_or(3)
    } else {
        3
    };

    let w = if args.len() > 3 {
        args[3].parse().unwrap_or(2)
    } else {
        2
    };

    let r = if args.len() > 4 {
        args[4].parse().unwrap_or(2)
    } else {
        2
    };

    println!("========================================");
    println!("Real Dynamo Benchmark");
    println!("========================================");
    println!();
    println!("Operations: {}", num_ops);
    println!("Configuration: N={}, W={}, R={}", n, w, r);
    println!("Using REAL Dynamo logic:");
    println!("  ✓ Real ConsistentHash routing");
    println!("  ✓ Real VectorClock operations");
    println!("  ✓ Real VersionedValue storage");
    println!("  ✓ Real quorum coordination");
    println!("  ✓ Real read repair");
    println!();

    // Setup real Dynamo cluster with dynamic node count
    let num_nodes = n.max(5); // At least 5 nodes for reasonable distribution
    let node_names: Vec<String> = (0..num_nodes)
        .map(|i| format!("node{}", (b'A' + i as u8) as char))
        .collect();

    let mut coordinator = RealQuorumCoordinator::new(node_names, n, w, r, 10);

    // Open output file
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("latency.log")
        .expect("Failed to create latency.log");

    // Run REAL operations with REAL timing
    for i in 0..num_ops {
        let key = format!("key{}", i % 100);
        let op_type = if i % 2 == 0 { "PUT" } else { "GET" };

        let start = Instant::now();

        if op_type == "PUT" {
            let value = format!("value_{}", i);
            // REAL PUT: executes actual consistent hashing, vector clock increment,
            // versioned storage, and quorum write
            coordinator.quorum_put(key.clone(), value);
        } else {
            // REAL GET: executes actual consistent hashing, quorum read,
            // version reconciliation, and read repair
            let _ = coordinator.quorum_get(&key);
        }

        // Measure REAL computation time
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        let log_line = format!("{} key={} latency={:.2}ms\n", op_type, key, latency_ms);
        file.write_all(log_line.as_bytes())
            .expect("Failed to write to latency.log");

        if (i + 1) % 200 == 0 {
            println!("  Progress: {}/{} operations", i + 1, num_ops);
        }
    }

    println!();
    println!("✓ Benchmark complete!");
    println!("  Output: latency.log");
    println!();
    println!("All operations executed REAL Dynamo code:");
    println!("  ✓ {} ConsistentHash lookups", num_ops);
    println!("  ✓ {} VectorClock operations", num_ops / 2);
    println!("  ✓ {} VersionedValue storage ops", num_ops / 2);
    println!("  ✓ {} Quorum reads with read repair", num_ops / 2);
}
