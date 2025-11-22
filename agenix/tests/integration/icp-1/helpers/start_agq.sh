#!/bin/bash
# Start AGQ in background for testing

set -e
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

log_info "Starting AGQ on port $TEST_PORT..."

# Start AGQ
"$AGQ_BIN" \
    --bind "127.0.0.1:$TEST_PORT" \
    --session-key "$TEST_SESSION_KEY" \
    > "$AGQ_LOG" 2>&1 &

AGQ_PID=$!
echo $AGQ_PID > "$AGQ_PID_FILE"

# Wait for AGQ to be ready (max 5 seconds)
log_info "Waiting for AGQ to start (PID: $AGQ_PID)..."
for i in {1..50}; do
    if ! kill -0 $AGQ_PID 2>/dev/null; then
        log_error "AGQ process died"
        cat "$AGQ_LOG" >&2
        exit 2
    fi

    if nc -z localhost $TEST_PORT 2>/dev/null; then
        log_info "AGQ ready on port $TEST_PORT"
        return 0
    fi

    sleep 0.1
done

log_error "AGQ failed to start within 5 seconds"
cat "$AGQ_LOG" >&2
kill $AGQ_PID 2>/dev/null || true
exit 2
