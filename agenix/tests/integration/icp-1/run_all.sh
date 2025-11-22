#!/bin/bash
# Run all ICP-1 integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers/config.sh"

echo "========================================"
echo "ICP-1 Integration Test Suite"
echo "========================================"
echo ""

# Cleanup any stale processes
log_info "Cleaning up any stale test processes..."
"$HELPERS_DIR/cleanup.sh" 2>/dev/null || true
sleep 1

# Track results
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Run each test
run_test() {
    local test_name="$1"
    local test_script="$2"

    echo ""
    echo "========================================"
    log_info "Running: $test_name"
    echo "========================================"

    TESTS_RUN=$((TESTS_RUN + 1))

    if bash "$test_script"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
        log_info "✅ $test_name PASSED"
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("$test_name")
        log_error "❌ $test_name FAILED"
    fi

    # Cleanup between tests
    "$HELPERS_DIR/cleanup.sh" 2>/dev/null || true
    sleep 2
}

# Test suite
run_test "Test 1: Simple Job" "$SCRIPT_DIR/test_1_simple_job.sh"
run_test "Test 2: Pipeline" "$SCRIPT_DIR/test_2_pipeline.sh"
run_test "Test 3: Worker Lifecycle" "$SCRIPT_DIR/test_3_worker_lifecycle.sh"

# TODO: Add Test 4 (error handling) and Test 5 (status query) when ready
# run_test "Test 4: Error Handling" "$SCRIPT_DIR/test_4_error_handling.sh"
# run_test "Test 5: Status Query" "$SCRIPT_DIR/test_5_status_query.sh"

# Summary
echo ""
echo "========================================"
echo "ICP-1 Test Suite Results"
echo "========================================"
echo "Tests Run:    $TESTS_RUN"
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $TESTS_FAILED"
echo ""

if [[ $TESTS_FAILED -gt 0 ]]; then
    log_error "Failed tests:"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    echo ""
    log_error "❌ TEST SUITE FAILED"
    exit 1
else
    log_info "✅ ALL TESTS PASSED"
    exit 0
fi
