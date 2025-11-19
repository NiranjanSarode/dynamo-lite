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

.PHONY: help build clean test node job

# Local reactor directory for controllers
LOCAL_REACTOR_DIR ?= ../reactor-master/target/debug
REACTOR_NCTRL := $(shell command -v reactor_nctrl 2>/dev/null || echo $(LOCAL_REACTOR_DIR)/reactor_nctrl)
REACTOR_JCTRL := $(shell command -v reactor_jctrl 2>/dev/null || echo $(LOCAL_REACTOR_DIR)/reactor_jctrl)

help:
	@echo "Dynamo-New Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build  - Build the Dynamo-New library"
	@echo "  clean  - Remove all build artifacts"
	@echo "  test   - Run standard Dynamo unit tests"
	@echo "  node   - Start node controller on port 3000"
	@echo "  job    - Run basic test job (5 nodes + 1 client)"
	@echo ""
	@echo "Usage:"
	@echo "  1. make build         # Build the library"
	@echo "  2. make node          # Start node controller (terminal 1)"
	@echo "  3. make job           # Run test job (terminal 2)"
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
	@echo "✓ Clean complete"

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
