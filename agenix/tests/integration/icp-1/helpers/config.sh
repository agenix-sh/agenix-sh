#!/bin/bash
# Shared configuration for ICP-1 integration tests

# Default binary paths (override with environment variables)
export AGX_BIN="${AGX_BIN:-/Users/lewis/work/agenix-sh/agx/target/release/agx}"
export AGQ_BIN="${AGQ_BIN:-/Users/lewis/work/agenix-sh/agq/target/release/agq}"
export AGW_BIN="${AGW_BIN:-/Users/lewis/work/agenix-sh/agw/target/release/agw}"

# Test configuration
export TEST_PORT="${TEST_PORT:-6379}"
export TEST_TIMEOUT="${TEST_TIMEOUT:-30}"
# Use fixed session key for testing (64 hex chars = 32 bytes)
export TEST_SESSION_KEY="deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"

# Paths
export TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export FIXTURES_DIR="$TEST_DIR/fixtures"
export HELPERS_DIR="$TEST_DIR/helpers"

# Log files
export AGQ_LOG="/tmp/agq-test-$$.log"
export AGW_LOG="/tmp/agw-test-$$.log"

# Process tracking
export AGQ_PID_FILE="/tmp/agq-test-$$.pid"
export AGW_PID_FILE="/tmp/agw-test-$$.pid"

# Colors for output
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export NC='\033[0m' # No Color

# Utility functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Verify binaries exist
check_binaries() {
    local missing=0

    if [[ ! -x "$AGX_BIN" ]]; then
        log_error "AGX binary not found: $AGX_BIN"
        log_error "Build it with: cd agx && cargo build --release"
        missing=1
    fi

    if [[ ! -x "$AGQ_BIN" ]]; then
        log_error "AGQ binary not found: $AGQ_BIN"
        log_error "Build it with: cd agq && cargo build --release"
        missing=1
    fi

    if [[ ! -x "$AGW_BIN" ]]; then
        log_error "AGW binary not found: $AGW_BIN"
        log_error "Build it with: cd agw && cargo build --release"
        missing=1
    fi

    if [[ $missing -eq 1 ]]; then
        return 2
    fi

    return 0
}

# Check if port is available
check_port() {
    if lsof -Pi :$TEST_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        log_error "Port $TEST_PORT already in use"
        log_error "Kill process with: lsof -ti:$TEST_PORT | xargs kill -9"
        return 2
    fi
    return 0
}
