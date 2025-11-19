# Dynamo-New

A decentralized, highly available key-value store implementation based on Amazon's Dynamo paper (SOSP 2007), built using the Reactor actor framework.

## Table of Contents
- [Architecture & Design Decisions](#architecture--design-decisions)
- [Quick Start](#quick-start)
- [Using Reactor Dashboard](#using-reactor-dashboard)
- [Configuration](#configuration)
- [Testing](#testing)

---

## Architecture & Design Decisions

### Core Design Principles

**1. Eventual Consistency over Strong Consistency**
- Prioritizes availability and partition tolerance (AP in CAP theorem)
- Accepts temporary inconsistencies for better performance and fault tolerance
- Conflicts are detected and can be resolved automatically or by application logic

**2. Decentralized Architecture**
- No single point of failure - every node is equal
- Any node can serve as coordinator for any request
- Self-organizing system using consistent hashing

**3. Always Writable**
- Writes succeed even during network partitions or node failures
- Uses sloppy quorum and hinted handoff to maintain availability

### Key Components

#### Vector Clocks
**Why:** Captures causality between events in a distributed system without synchronized clocks.

```rust
// Track which node made which update
VectorClock { "nodeA": 2, "nodeB": 1, "nodeC": 3 }
```

**Design Decision:**
- Allows detection of concurrent vs. sequential writes
- Enables automatic conflict detection
- Small overhead: grows with number of nodes that update a key

**Trade-offs:**
- Size grows with number of writers (mitigated by periodic pruning)
- Cannot detect all conflicts (siblings may occur)

#### Multi-Version Storage
**Why:** Preserve conflicting concurrent writes instead of losing data.

```rust
pub struct VersionedValues {
    pub versions: Vec<VersionedValue>,  // Multiple concurrent versions
}
```

**Design Decision:**
- Store all concurrent versions (siblings)
- Let application resolve conflicts semantically
- Example: Shopping cart merges all items from concurrent adds

**Trade-offs:**
- Increased storage for conflicting writes
- Application must handle conflict resolution
- Benefit: Never lose writes during partitions

#### Consistent Hashing with Virtual Nodes
**Why:** Evenly distribute data and handle node additions/removals gracefully.

```
Virtual nodes per physical node: T=10
Ring: 0 → 2^32-1 (hash space)
```

**Design Decision:**
- Each node owns T virtual nodes on ring
- Key hashed to ring position, stored on next N nodes clockwise
- When node fails, load redistributes to remaining nodes

**Trade-offs:**
- More virtual nodes = better load balance, more memory
- T=10 provides good balance for small clusters (5-10 nodes)
- T=150 recommended for production (100+ nodes)

#### Quorum Replication (N, R, W)
**Why:** Tunable consistency-availability-latency trade-off.

```
N = 3  # Replicate to 3 nodes
R = 2  # Read from 2 nodes
W = 2  # Write to 2 nodes
```

**Design Decision:**
- W + R > N guarantees read-your-writes consistency
- Default (N=3, R=2, W=2) balances consistency and availability

**Trade-off Examples:**

| Config | Consistency | Latency | Availability |
|--------|-------------|---------|--------------|
| R=1, W=1 | Weak | Low | High |
| R=2, W=2 | Medium | Medium | Medium |
| R=3, W=3 | Strong | High | Low |

**When to use:**
- High read throughput: R=1, W=N (eventual consistency)
- High write throughput: R=N, W=1 (read-heavy consistency)
- Balanced: R=2, W=2, N=3 (default)

#### Read Repair
**Why:** Passively bring stale replicas up to date during normal operations.

```rust
// On read, if replicas have different versions:
// 1. Return latest to client
// 2. Asynchronously push latest to stale replicas
```

**Design Decision:**
- Happens during read path (no extra traffic)
- Reduces inconsistency window without active anti-entropy

**Trade-offs:**
- Only repairs accessed keys (cold data may diverge)
- Adds minimal latency to read operations

#### Hinted Handoff
**Why:** Accept writes when primary replicas are unavailable.

```
Normal:    Write to nodeA, nodeB, nodeC
Node B down: Write to nodeA, nodeC, nodeD (hint for B)
```

**Design Decision:**
- Write succeeds even with N-W failures
- Temporary replica (nodeD) holds data until nodeB recovers
- Maintains write availability during transient failures

**Trade-offs:**
- Increases eventual consistency window
- Requires storage for hints
- Benefit: Never reject writes due to temporary failures

#### Anti-Entropy Synchronization
**Why:** Repair inconsistencies not caught by read repair.

```
Every 1 second: Compare versions with random peers
Batch size: 2 keys per sync
```

**Design Decision:**
- Gossip-based background process
- Eventually all replicas converge
- Catches divergence from permanent failures

**Trade-offs:**
- Background network overhead
- Tunable: frequency vs. consistency speed
- Complements read repair for full convergence

### Actor Model (Reactor Framework)
**Why:** Natural fit for distributed systems.

```
Each Dynamo node = Actor
- Receives messages (get/put requests)
- Maintains local state (storage)
- Sends messages (coordinate with replicas)
```

**Design Decision:**
- Message-passing matches network communication
- No shared state = easier reasoning about concurrency
- Reactor provides scheduling, fault tolerance, deployment

**Trade-offs:**
- Learning curve for actor programming
- Message overhead vs. direct function calls
- Benefit: Clean separation, testability, scalability

---

## Quick Start

### Prerequisites
- Rust toolchain (1.70+)
- Reactor controllers (built automatically from `../reactor-master`)

### Three-Step Workflow

**Terminal 1: Build and start node controller**
```bash
cd dynamo-new
make build
make node
```

Output:
```
==========================================
Starting Reactor Node Controller
==========================================

Port: 3000
Library Path: ./target/debug

The node controller will:
  • Load Dynamo-New library
  • Listen for job placement requests
  • Spawn actor instances (nodes & clients)
```

**Keep this terminal running!**

**Terminal 2: Run test job**
```bash
cd dynamo-new
make job
```

Output:
```
==========================================
Running Basic Test Job
==========================================

Configuration (basic_test.toml):
  • 5 Dynamo nodes (nodeA-nodeE)
  • 1 Client (client1)
  • Quorum: N=3, R=2, W=2
  • Virtual nodes per node: T=10
```

### Makefile Commands

```bash
make help   # Show all available commands
make build  # Compile the Dynamo-New library
make clean  # Remove build artifacts
make test   # Run unit tests (vector clocks, versioned values, etc.)
make node   # Start node controller on port 3000
make job    # Deploy and run basic test job
```

---

## Using Reactor Dashboard

The Reactor framework includes a web-based dashboard for visualizing and debugging distributed actor systems.

### Starting the Dashboard

**Terminal 3: Launch dashboard**
```bash
cd ../reactor-master
cargo run --bin reactor_dashboard
```

Or if installed globally:
```bash
reactor_dashboard
```

Default: http://localhost:8080

### Dashboard Features

**1. Topology View**
- Visual graph of all nodes and actors
- See which actors are deployed on which physical nodes
- Real-time updates as actors spawn/terminate

**2. Message Flow**
- Live visualization of messages between actors
- Color-coded by message type (get/put/sync/etc.)
- Latency information per message

**3. Actor Inspector**
- Click any actor to see:
  - Current state (stored keys/values)
  - Recent messages sent/received
  - Vector clocks for stored versions
  - Pending hinted handoffs

**4. Performance Metrics**
- Request latency percentiles (p50, p99)
- Throughput (ops/sec)
- Message queue depths
- Network utilization per node

**5. Chaos Engineering**
- Inject failures:
  - Drop messages (simulate packet loss)
  - Delay messages (simulate network latency)
  - Partition network (split brain scenarios)
  - Kill actors (node failures)
- Test read repair and hinted handoff in real-time

### Example Dashboard Workflow

**Scenario: Visualize Quorum Write**

1. Start dashboard, node controller, and job
2. In dashboard, select "Message Flow" view
3. Trigger a write from client (happens automatically in basic_test)
4. Observe:
   ```
   client1 → nodeC (coordinator)
   nodeC → nodeA (replica 1)
   nodeC → nodeD (replica 2)
   nodeC → nodeE (replica 3)
   nodeA → nodeC (ack W=1)
   nodeD → nodeC (ack W=2) ← Quorum reached!
   nodeC → client1 (success)
   nodeE → nodeC (ack W=3 after quorum)
   ```

**Scenario: Test Network Partition**

1. In dashboard, select nodes nodeA and nodeB
2. Click "Create Partition" (isolate from nodeC, D, E)
3. Trigger writes from client
4. Observe:
   - Writes to keys hashing to A/B succeed (hinted handoff to C/D/E)
   - Writes to keys hashing to C/D/E succeed normally
   - After healing partition, observe anti-entropy sync

**Scenario: Inspect Version Conflicts**

1. Use chaos tools to partition network
2. Trigger concurrent writes to same key from both partitions
3. Heal partition
4. In Actor Inspector, click coordinator node
5. See multiple versions (siblings) for conflicted key
6. Observe vector clocks showing concurrent updates

### Dashboard Configuration

Edit `reactor-master/dashboard_config.toml`:

```toml
[dashboard]
host = "0.0.0.0"
port = 8080
refresh_rate_ms = 100  # UI update frequency

[monitoring]
enable_metrics = true
history_size = 1000    # Messages to keep in buffer
```

### Useful Dashboard Tips

**Debugging slow requests:**
1. Message Flow → Filter by actor
2. Check message latencies
3. Look for queuing delays (actor overloaded)

**Verifying quorum behavior:**
1. Actor Inspector → Select coordinator
2. View "Pending Requests" state
3. See W/R thresholds and acknowledgments

**Testing failure scenarios:**
1. Kill 1 node → Verify W+R-N nodes can still serve requests
2. Kill 2 nodes → Should still work with N=3, W=2, R=2
3. Kill 3 nodes → Writes fail (cannot reach W=2)

---

## Configuration

### Quorum Parameters (`basic_test.toml`)

```toml
[[placement.dynamo_node]]
actor_name = "nodeA"
node_id = "nodeA"
nodes = ["nodeA","nodeB","nodeC","nodeD","nodeE"]  # All nodes in cluster
N = 3   # Replication factor
W = 2   # Write quorum
R = 2   # Read quorum
T = 10  # Virtual nodes per physical node
```

**Tuning Guidelines:**

- **High availability:** N=3, W=1, R=1 (tolerates 2 failures)
- **Balanced (default):** N=3, W=2, R=2 (tolerates 1 failure, read-your-writes)
- **Strong consistency:** N=3, W=3, R=1 (all writes durable, no stale reads)
- **Read-heavy:** N=5, W=4, R=2 (fast reads, slow writes)

**Virtual nodes (T):**
- Small clusters (< 10 nodes): T=10
- Medium clusters (10-50 nodes): T=50
- Large clusters (50+ nodes): T=150
- Trade-off: Load balance vs. memory overhead

### Adding More Nodes

```toml
# Add nodeF, nodeG...
[[placement.dynamo_node]]
actor_name = "nodeF"
node_id = "nodeF"
nodes = ["nodeA","nodeB","nodeC","nodeD","nodeE","nodeF","nodeG"]
N = 3
W = 2
R = 2
T = 10
```

Update all existing node configurations to include new nodes in `nodes = [...]` array.

---

## Testing

### Unit Tests
```bash
make test
```

Covers:
- **Vector clock operations:** increment, merge, compare, converge
- **Versioned value storage:** add, supersede, detect conflicts
- **Causal consistency:** happens-before, concurrent detection
- **Conflict resolution:** sibling preservation, last-write-wins

### Integration Tests (Currently Commented Out)

Located in `tests/integration_tests.rs` and `tests/advanced_tests.rs`.

**To enable:**
1. Uncomment test files
2. Run `cargo test`

**Coverage (38 tests):**
- Quorum simulation (W=2, R=2, sloppy quorum)
- Read repair scenarios
- Shopping cart conflicts (merge siblings)
- Network partition healing
- Multi-datacenter causality
- Edge cases (1000-node vector clocks, 50 concurrent siblings)

### Manual Testing with Dashboard

**Test 1: Basic Put/Get**
```
1. Start node + job + dashboard
2. Watch client1 put "key1" → "value1"
3. Observe coordinator selection via consistent hash
4. See fanout to N=3 replicas
5. Verify W=2 acknowledgments
6. Client gets "key1" → returns "value1"
```

**Test 2: Concurrent Writes**
```
1. Use dashboard to partition network
2. Write "key1" → "A" from partition 1
3. Write "key1" → "B" from partition 2
4. Heal partition
5. Get "key1" → observe siblings [A, B]
6. Check vector clocks show concurrency
```

**Test 3: Read Repair**
```
1. Write "key2" → "v1" (reaches nodeA, nodeB, not nodeC)
2. Kill nodeC temporarily
3. Write "key2" → "v2" (reaches nodeA, nodeB)
4. Restart nodeC (still has v1)
5. Read "key2" from coordinator that includes nodeC
6. Verify coordinator repairs nodeC with v2
```

---

## Project Structure

```
dynamo-new/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── vector_clock.rs     # Causality tracking
│   ├── versioned_value.rs  # Multi-version storage
│   ├── consistent_hash.rs  # Virtual node ring
│   ├── node.rs             # Dynamo storage node actor
│   ├── client.rs           # Test client actor
│   ├── cart_client.rs      # Shopping cart demo client
│   └── messages.rs         # Actor message types
├── tests/
│   ├── integration_tests.rs  # 23 integration tests (commented)
│   └── advanced_tests.rs     # 15 advanced tests (commented)
├── basic_test.toml         # Deployment configuration
├── Makefile                # Build and run commands
└── README.md               # This file
```

---

## Design Trade-offs Summary

| Aspect | Choice | Alternative | Reasoning |
|--------|--------|-------------|-----------|
| Consistency | Eventual | Strong | High availability, partition tolerance |
| Conflicts | Multi-version | Last-write-wins | Preserve all writes, app resolves |
| Replication | Quorum | Chain/Tree | Tunable consistency, low latency |
| Membership | Consistent hash | Master-based | Decentralized, elastic scaling |
| Clocks | Vector clocks | Timestamps | Causality without clock sync |
| Repair | Read repair + Anti-entropy | Active repair | Balances overhead and convergence |
| Framework | Actor model | Thread pool | Natural fit for distributed messages |

---

## References

- [Dynamo: Amazon's Highly Available Key-value Store (SOSP 2007)](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf)
- [Reactor Actor Framework](https://github.com/satyamjay-iitd/reactor/)
- [Vector Clocks in Distributed Systems](https://en.wikipedia.org/wiki/Vector_clock)
- [Consistent Hashing](https://en.wikipedia.org/wiki/Consistent_hashing)

---

## License

MIT
