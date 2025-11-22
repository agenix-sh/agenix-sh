#!/bin/bash
# ICP-1 Test 1: Simple Single-Task Job
#
# Tests basic job submission and execution with a single echo command.
# This validates the minimal viable path: AGX → AGQ → AGW → AGQ

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers/config.sh"

echo "========================================"
echo "ICP-1 Test 1: Simple Single-Task Job"
echo "========================================"

# Check prerequisites
check_binaries || exit 2
check_port || exit 2

# Cleanup any previous test runs
"$HELPERS_DIR/cleanup.sh" 2>/dev/null || true

# Start AGQ
log_info "Step 1: Starting AGQ..."
source "$HELPERS_DIR/start_agq.sh"

# Start AGW
log_info "Step 2: Starting AGW..."
source "$HELPERS_DIR/start_agw.sh" "test-worker-1"

# Wait for initialization
sleep 2

# Submit job via AGX
log_info "Step 3: Submitting simple job..."
JOB_FILE="$FIXTURES_DIR/simple-job.json"

# Extract plan from job fixture (remove job_id, keep plan_id and tasks)
PLAN_FILE="/tmp/icp-1-simple-plan-$$.json"
cat "$JOB_FILE" | jq '{plan_id, plan_description, tasks}' > "$PLAN_FILE"

# Load plan into AGX buffer
PLAN_BUFFER="/tmp/icp-1-plan-buffer-$$.json"
cp "$PLAN_FILE" "$PLAN_BUFFER"

# Submit plan to AGQ using AGX
log_info "Submitting plan to AGQ..."
SUBMIT_OUTPUT=$(AGX_PLAN_PATH="$PLAN_BUFFER" \
    AGQ_ADDR="127.0.0.1:$TEST_PORT" \
    AGQ_SESSION_KEY="$TEST_SESSION_KEY" \
    "$AGX_BIN" PLAN submit 2>&1)

if [ $? -ne 0 ]; then
    log_error "Plan submission failed"
    echo "$SUBMIT_OUTPUT" >&2
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Extract job_id from submission response
JOB_ID=$(echo "$SUBMIT_OUTPUT" | jq -r '.job_id // empty')
if [ -z "$JOB_ID" ]; then
    log_error "No job_id returned from PLAN submit"
    echo "$SUBMIT_OUTPUT" >&2
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

log_info "Job submitted: $JOB_ID"

# Cleanup temp files
rm -f "$PLAN_FILE" "$PLAN_BUFFER"

# Wait for job completion
log_info "Step 4: Waiting for job to complete..."
if ! source "$HELPERS_DIR/wait_for_status.sh" "$JOB_ID" 30; then
    log_error "Job did not complete successfully"
    log_error "AGQ log:"
    tail -20 "$AGQ_LOG" >&2
    log_error "AGW log:"
    tail -20 "$AGW_LOG" >&2
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Verify output
log_info "Step 5: Verifying job output..."

# TODO: Replace with actual AGX query command
# Expected: agx job stdout <job-id> --agq-addr <addr> --session-key <key>
STDOUT=$("$AGX_BIN" job stdout "$JOB_ID" \
    --agq-addr "127.0.0.1:$TEST_PORT" \
    --session-key "$TEST_SESSION_KEY" 2>/dev/null || echo "")

EXPECTED="hello world"

if [[ "$STDOUT" == "$EXPECTED" ]]; then
    log_info "✅ Output matches expected: '$EXPECTED'"
else
    log_error "❌ Output mismatch"
    log_error "Expected: '$EXPECTED'"
    log_error "Got: '$STDOUT'"
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Cleanup
log_info "Step 6: Cleanup..."
"$HELPERS_DIR/cleanup.sh"

echo ""
log_info "========================================"
log_info "✅ Test 1 PASSED"
log_info "========================================"
exit 0
