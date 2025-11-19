# ============================================================================
# Dynamo-New Makefile
#
# Dynamo-New is a decentralized key-value store implementation based on
# Amazon's Dynamo paper. It provides:
# - Vector clock-based causality tracking
# - Multi-version conflict detection and resolution
# - Quorum-based replication (N=3, R=2, W=2)
# - Read repair for eventual consistency
# - Hinted handoff for availability during failures
# ============================================================================

.PHONY: help build clean test node job benchmark graphs reactor-install

# Local reactor directory for controllers
LOCAL_REACTOR_DIR ?= ../reactor-master/target/debug
REACTOR_NCTRL := $(shell command -v reactor_nctrl 2>/dev/null || echo $(LOCAL_REACTOR_DIR)/reactor_nctrl)
REACTOR_JCTRL := $(shell command -v reactor_jctrl 2>/dev/null || echo $(LOCAL_REACTOR_DIR)/reactor_jctrl)

help:
	@echo "Dynamo-New Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build            - Build the Dynamo-New library"
	@echo "  clean            - Remove all build artifacts"
	@echo "  test             - Run standard Dynamo unit tests"
	@echo "  reactor-install  - Build Reactor framework (required for node/job)"
	@echo "  node             - Start node controller on port 3000"
	@echo "  job              - Run basic test job (5 nodes + 1 client)"
	@echo "  benchmark        - Run REAL Dynamo benchmarks and generate graphs"
	@echo "  graphs           - Generate graphs from existing benchmark logs"
	@echo ""
	@echo "Quick Start:"
	@echo "  1. make reactor-install  # Build Reactor (one-time setup)"
	@echo "  2. make build            # Build Dynamo library"
	@echo "  3. make benchmark        # Run benchmarks with REAL Dynamo logic"
	@echo ""
	@echo "Reactor Usage (optional):"
	@echo "  1. make node          # Start node controller (terminal 1)"
	@echo "  2. make job           # Run test job (terminal 2)"
	@echo ""
	@echo "Benchmarking:"
	@echo "  make benchmark        # Runs 100% REAL Dynamo data collection"
	@echo "     • 1000 ops with actual ConsistentHash, VectorClock, Quorum logic"
	@echo "     • 4800 ops across 24 configurations (N=3,5,10,20)"
	@echo "     • Generates 10 PNG graphs"
	@echo "     • Zero simulated delays - only real computation time"
	@echo ""
	@echo "  make graphs           # Generate graphs from existing logs"
	@echo ""
	@echo "Architecture:"
	@echo "  - Causally consistent key-value store"
	@echo "  - Vector clocks for conflict detection"
	@echo "  - Quorum replication: N=3, R=2, W=2"
	@echo "  - Anti-entropy synchronization"
	@echo ""

# Build the library in debug mode
# Compiles the Dynamo-New implementation with all dependencies
build:
	@echo "Building Dynamo-New (debug mode)..."
	cargo build
	@echo "✓ Build complete: target/debug/libdynamo_new.*"

# Remove all build artifacts and clean the project
# Deletes the target/ directory and all compiled binaries
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f latency.log latency_configs.log *.png
	@echo "✓ Clean complete"

# Build Reactor framework (one-time setup)
# Required for using make node and make job targets
reactor-install:
	@echo "=========================================="
	@echo "Building Reactor Framework"
	@echo "=========================================="
	@echo ""
	@if [ ! -d "../reactor-master" ]; then \
		echo "Error: reactor-master directory not found"; \
		echo "Expected location: ../reactor-master"; \
		exit 1; \
	fi
	@echo "Building reactor_nctrl and reactor_jctrl..."
	cd ../reactor-master && cargo build
	@echo ""
	@echo "✓ Reactor built successfully!"
	@echo "  reactor_nctrl: ../reactor-master/target/debug/reactor_nctrl"
	@echo "  reactor_jctrl: ../reactor-master/target/debug/reactor_jctrl"
	@echo ""
	@echo "You can now use:"
	@echo "  make node  # Start node controller"
	@echo "  make job   # Run Dynamo job"

# Run standard Dynamo unit tests
# Tests vector clocks, versioned values, causal consistency, and conflict resolution
# Note: Integration tests with distributed nodes are currently commented out
test:
	@echo "=========================================="
	@echo "Running Dynamo-New Unit Tests"
	@echo "=========================================="
	@echo ""
	@echo "Test Coverage:"
	@echo "  • Vector clock operations (increment, merge, compare)"
	@echo "  • Versioned value storage and conflict detection"
	@echo "  • Causal consistency properties"
	@echo "  • Conflict resolution with concurrent writes"
	@echo ""
	@echo "Note: Distributed integration tests are commented out."
	@echo "      To enable, uncomment tests/integration_tests.rs"
	@echo ""
	cargo test
	@echo ""
	@echo "✓ Tests complete"

# Start the Reactor node controller
# Runs on port 3000 and loads libraries from target/debug
# This must be running before executing jobs
node:
	@echo "=========================================="
	@echo "Starting Reactor Node Controller"
	@echo "=========================================="
	@echo ""
	@echo "Port: 3000"
	@echo "Library Path: ./target/debug"
	@echo ""
	@echo "The node controller will:"
	@echo "  • Load Dynamo-New library"
	@echo "  • Listen for job placement requests"
	@echo "  • Spawn actor instances (nodes & clients)"
	@echo ""
	"$(REACTOR_NCTRL)" --port 3000 ./target/debug

# Run the basic test job
# Spawns 5 Dynamo nodes + 1 client on the node controller
# Uses quorum configuration: N=3, R=2, W=2
job:
	@echo "=========================================="
	@echo "Running Basic Test Job"
	@echo "=========================================="
	@echo ""
	@echo "Configuration (basic_test.toml):"
	@echo "  • 5 Dynamo nodes (nodeA-nodeE)"
	@echo "  • 1 Client (client1)"
	@echo "  • Quorum: N=3, R=2, W=2"
	@echo "  • Virtual nodes per node: T=10"
	@echo ""
	@echo "The job will:"
	@echo "  • Place actors on node controller"
	@echo "  • Execute test workload"
	@echo "  • Show coordination and replication logs"
	@echo ""
	"$(REACTOR_JCTRL)" ./basic_test.toml

# Run latency benchmark with 100% REAL Dynamo data collection
# Uses real_benchmark Rust binary executing actual Dynamo logic:
#   - Real ConsistentHash routing
#   - Real VectorClock operations
#   - Real VersionedValue storage and conflict resolution
#   - Real quorum coordination (W=2, R=2)
#   - Real read repair logic
# NO simulated delays - only real computation time is measured
benchmark:
	@echo "=========================================="
	@echo "Dynamo Real Data Collection"
	@echo "=========================================="
	@echo ""
	@echo "Building real_benchmark binary..."
	@cargo build --bin real_benchmark 2>/dev/null || cargo build --bin real_benchmark
	@echo ""
	python3 run_benchmark.py
	@echo ""
	@echo "3. Generating All Graphs"
	@echo ""
	@$(MAKE) graphs
	@echo ""
	@echo "✓ Benchmark complete!"
	@echo "  Basic latency (4 graphs):"
	@echo "    • latency_read_histogram.png"
	@echo "    • latency_write_histogram.png"
	@echo "    • latency_percentiles.png"
	@echo "    • latency_cdf.png"
	@echo "  Config comparison (4 graphs):"
	@echo "    • latency_config_get_boxplot.png"
	@echo "    • latency_config_put_boxplot.png"
	@echo "    • latency_config_median.png"
	@echo "    • latency_config_p99.png"
	@echo "  N=20 analysis (2 graphs):"
	@echo "    • latency_n20_varying_r.png"
	@echo "    • latency_n20_varying_w.png"

# Generate all latency graphs from benchmark logs
# Parses both latency.log and latency_configs.log
# Creates visualizations for basic latencies, config comparison, and N=20 analysis
# Outputs: 10 PNG files (4 basic + 4 config + 2 N=20)
graphs:
	@echo "=========================================="
	@echo "Generating All Latency Graphs"
	@echo "=========================================="
	@echo ""
	@if [ ! -f latency.log ] && [ ! -f latency_configs.log ]; then \
		echo "Error: No benchmark data found"; \
		echo "Run 'make benchmark' first to generate latency data"; \
		echo ""; \
		exit 1; \
	fi
	python3 generate_all_graphs.py
	@echo ""
	@echo "✓ All graphs generated!"

