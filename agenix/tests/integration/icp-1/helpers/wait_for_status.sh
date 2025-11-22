#!/bin/bash
# Poll job status until it reaches a terminal state (completed/failed)

set -e
source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

JOB_ID="$1"
MAX_WAIT="${2:-$TEST_TIMEOUT}"

if [[ -z "$JOB_ID" ]]; then
    log_error "Usage: wait_for_status.sh <job-id> [max-wait-seconds]"
    exit 1
fi

log_info "Waiting for job $JOB_ID to complete (max ${MAX_WAIT}s)..."

for i in $(seq 1 $MAX_WAIT); do
    # Query job status via AGX (this will depend on final AGX CLI interface)
    # For now, we'll use a placeholder command structure
    # This will need to be updated once AGX job status query is finalized

    # Placeholder: Assume AGX has a 'job status' command
    STATUS=$("$AGX_BIN" job status "$JOB_ID" \
        --agq-addr "127.0.0.1:$TEST_PORT" \
        --session-key "$TEST_SESSION_KEY" \
        2>/dev/null | jq -r '.status' || echo "unknown")

    case "$STATUS" in
        completed)
            log_info "Job $JOB_ID completed successfully"
            return 0
            ;;
        failed)
            log_error "Job $JOB_ID failed"
            return 1
            ;;
        pending|running)
            # Still in progress, continue waiting
            sleep 1
            ;;
        unknown)
            log_warn "Could not query job status (attempt $i/$MAX_WAIT)"
            sleep 1
            ;;
        *)
            log_warn "Unexpected status: $STATUS (attempt $i/$MAX_WAIT)"
            sleep 1
            ;;
    esac
done

log_error "Job $JOB_ID did not complete within ${MAX_WAIT}s"
return 3
