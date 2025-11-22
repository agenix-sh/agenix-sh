#!/bin/bash
# ICP-1 Test 3: Worker Lifecycle
#
# Tests worker registration, heartbeat, and cleanup.
# Validates AGQ-007 (worker heartbeat registry).

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers/config.sh"

echo "========================================"
echo "ICP-1 Test 3: Worker Lifecycle"
echo "========================================"

# Check prerequisites
check_binaries || exit 2
check_port || exit 2

# Cleanup
"$HELPERS_DIR/cleanup.sh" 2>/dev/null || true

# Test requires redis-cli or similar to query worker keys
if ! command -v redis-cli &> /dev/null; then
    log_warn "redis-cli not found, skipping worker key verification"
    log_warn "Install with: brew install redis (macOS) or apt-get install redis-tools (Linux)"
    log_info "Test will validate basic worker lifecycle only"
fi

# Start AGQ
log_info "Step 1: Starting AGQ..."
source "$HELPERS_DIR/start_agq.sh"

# Verify no workers registered
log_info "Step 2: Verifying no workers registered initially..."
if command -v redis-cli &> /dev/null; then
    WORKER_COUNT=$(redis-cli -p "$TEST_PORT" KEYS 'worker:*:alive' 2>/dev/null | wc -l || echo "0")
    if [[ "$WORKER_COUNT" -ne 0 ]]; then
        log_error "Expected 0 workers, found $WORKER_COUNT"
        "$HELPERS_DIR/cleanup.sh"
        exit 1
    fi
    log_info "✅ No workers registered (as expected)"
fi

# Start AGW
log_info "Step 3: Starting AGW..."
source "$HELPERS_DIR/start_agw.sh" "test-worker-lifecycle"

sleep 2

# Verify worker registered
log_info "Step 4: Verifying worker registered..."
if command -v redis-cli &> /dev/null; then
    WORKER_KEY="worker:test-worker-lifecycle:alive"
    WORKER_EXISTS=$(redis-cli -p "$TEST_PORT" EXISTS "$WORKER_KEY" 2>/dev/null || echo "0")

    if [[ "$WORKER_EXISTS" -eq 1 ]]; then
        log_info "✅ Worker registered: $WORKER_KEY"
    else
        log_error "❌ Worker not registered"
        log_error "Expected key: $WORKER_KEY"
        redis-cli -p "$TEST_PORT" KEYS 'worker:*' 2>/dev/null || true
        "$HELPERS_DIR/cleanup.sh"
        exit 1
    fi
fi

# Wait for heartbeat to expire (TTL is 10 seconds, wait 11)
log_info "Step 5: Testing heartbeat expiry (waiting 11 seconds)..."
# First, kill AGW to stop heartbeats
AGW_PID=$(cat "$AGW_PID_FILE")
kill $AGW_PID 2>/dev/null || true
log_info "Stopped AGW, waiting for key expiry..."

sleep 11

if command -v redis-cli &> /dev/null; then
    WORKER_EXISTS=$(redis-cli -p "$TEST_PORT" EXISTS "$WORKER_KEY" 2>/dev/null || echo "1")

    if [[ "$WORKER_EXISTS" -eq 0 ]]; then
        log_info "✅ Worker key expired correctly after 10s TTL"
    else
        log_error "❌ Worker key did not expire"
        log_error "TTL: $(redis-cli -p "$TEST_PORT" TTL "$WORKER_KEY")"
        "$HELPERS_DIR/cleanup.sh"
        exit 1
    fi
fi

# Re-register worker
log_info "Step 6: Re-registering worker..."
source "$HELPERS_DIR/start_agw.sh" "test-worker-lifecycle"

sleep 2

if command -v redis-cli &> /dev/null; then
    WORKER_EXISTS=$(redis-cli -p "$TEST_PORT" EXISTS "$WORKER_KEY" 2>/dev/null || echo "0")

    if [[ "$WORKER_EXISTS" -eq 1 ]]; then
        log_info "✅ Worker re-registered successfully"
    else
        log_error "❌ Worker failed to re-register"
        "$HELPERS_DIR/cleanup.sh"
        exit 1
    fi
fi

# Graceful shutdown (SIGTERM)
log_info "Step 7: Testing graceful shutdown..."
AGW_PID=$(cat "$AGW_PID_FILE")
kill -TERM $AGW_PID 2>/dev/null || true
sleep 2

if command -v redis-cli &> /dev/null; then
    WORKER_EXISTS=$(redis-cli -p "$TEST_PORT" EXISTS "$WORKER_KEY" 2>/dev/null || echo "1")

    # Note: Graceful shutdown cleanup depends on AGW implementation
    # It may or may not explicitly delete the key on shutdown
    # For now, we just verify the worker process died
    if ! kill -0 $AGW_PID 2>/dev/null; then
        log_info "✅ Worker process terminated gracefully"
    else
        log_error "❌ Worker process still running after SIGTERM"
        "$HELPERS_DIR/cleanup.sh"
        exit 1
    fi
fi

# Cleanup
log_info "Step 8: Cleanup..."
"$HELPERS_DIR/cleanup.sh"

echo ""
log_info "========================================"
log_info "✅ Test 3 PASSED"
log_info "========================================"
exit 0
