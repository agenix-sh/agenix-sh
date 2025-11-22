#!/bin/bash
# ICP-1 Test 2: Multi-Task Pipeline
#
# Tests task chaining with stdin/stdout piping between tasks.
# Pipeline: sort -r | uniq

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers/config.sh"

echo "========================================"
echo "ICP-1 Test 2: Multi-Task Pipeline"
echo "========================================"

# Check prerequisites
check_binaries || exit 2
check_port || exit 2

# Cleanup
"$HELPERS_DIR/cleanup.sh" 2>/dev/null || true

# Start components
log_info "Step 1: Starting AGQ..."
source "$HELPERS_DIR/start_agq.sh"

log_info "Step 2: Starting AGW..."
source "$HELPERS_DIR/start_agw.sh" "test-worker-2"

sleep 2

# Submit pipeline job with input data
log_info "Step 3: Submitting pipeline job..."
JOB_FILE="$FIXTURES_DIR/pipeline-job.json"
INPUT_FILE="$FIXTURES_DIR/sample-input.txt"

JOB_ID=$(cat "$JOB_FILE" | jq -r '.job_id')
log_info "Submitting job: $JOB_ID with input data"

# TODO: Update with actual AGX submission command that accepts stdin
# Expected: cat input.txt | agx job submit --file job.json ...
if ! cat "$INPUT_FILE" | "$AGX_BIN" job submit \
    --file "$JOB_FILE" \
    --agq-addr "127.0.0.1:$TEST_PORT" \
    --session-key "$TEST_SESSION_KEY" \
    >/dev/null 2>&1; then
    log_error "Job submission failed"
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Wait for completion
log_info "Step 4: Waiting for pipeline to complete..."
if ! source "$HELPERS_DIR/wait_for_status.sh" "$JOB_ID" 30; then
    log_error "Pipeline job failed"
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Verify output
log_info "Step 5: Verifying pipeline output..."

STDOUT=$("$AGX_BIN" job stdout "$JOB_ID" \
    --agq-addr "127.0.0.1:$TEST_PORT" \
    --session-key "$TEST_SESSION_KEY" 2>/dev/null || echo "")

# Expected: Input sorted in reverse, then unique
# Input: apple, banana, apple, cherry, banana, apple, date, cherry
# After sort -r: date, cherry, cherry, banana, banana, apple, apple, apple
# After uniq: date, cherry, banana, apple
EXPECTED="date
cherry
banana
apple"

if [[ "$STDOUT" == "$EXPECTED" ]]; then
    log_info "✅ Pipeline output correct"
else
    log_error "❌ Pipeline output incorrect"
    log_error "Expected:"
    echo "$EXPECTED" >&2
    log_error "Got:"
    echo "$STDOUT" >&2
    "$HELPERS_DIR/cleanup.sh"
    exit 1
fi

# Cleanup
log_info "Step 6: Cleanup..."
"$HELPERS_DIR/cleanup.sh"

echo ""
log_info "========================================"
log_info "✅ Test 2 PASSED"
log_info "========================================"
exit 0
