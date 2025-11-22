#!/bin/bash
# Start AGW in background for testing

set -e
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

WORKER_ID="${1:-test-worker-$$}"

log_info "Starting AGW (worker: $WORKER_ID)..."

# Start AGW
"$AGW_BIN" \
    --worker-id "$WORKER_ID" \
    --agq-address "127.0.0.1:$TEST_PORT" \
    --session-key "$TEST_SESSION_KEY" \
    > "$AGW_LOG" 2>&1 &

AGW_PID=$!
echo $AGW_PID > "$AGW_PID_FILE"

# Wait for AGW to connect (check logs for "connected" or similar)
log_info "Waiting for AGW to connect (PID: $AGW_PID)..."
for i in {1..30}; do
    if ! kill -0 $AGW_PID 2>/dev/null; then
        log_error "AGW process died"
        cat "$AGW_LOG" >&2
        exit 2
    fi

    # Check if worker registered (this will depend on AGW implementation)
    # For now, just wait 2 seconds for initialization
    if [[ $i -ge 20 ]]; then
        log_info "AGW initialized (PID: $AGW_PID)"
        return 0
    fi

    sleep 0.1
done

log_error "AGW failed to start within 3 seconds"
cat "$AGW_LOG" >&2
kill $AGW_PID 2>/dev/null || true
exit 2
