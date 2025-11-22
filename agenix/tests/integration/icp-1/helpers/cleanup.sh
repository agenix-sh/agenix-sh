#!/bin/bash
# Cleanup all test processes and temporary files

source "$(dirname "${BASH_SOURCE[0]}")/config.sh"

log_info "Cleaning up test processes..."

# Kill AGW
if [[ -f "$AGW_PID_FILE" ]]; then
    AGW_PID=$(cat "$AGW_PID_FILE")
    if kill -0 $AGW_PID 2>/dev/null; then
        log_info "Stopping AGW (PID: $AGW_PID)"
        kill $AGW_PID 2>/dev/null || true
        sleep 1
        kill -9 $AGW_PID 2>/dev/null || true
    fi
    rm -f "$AGW_PID_FILE"
fi

# Kill AGQ
if [[ -f "$AGQ_PID_FILE" ]]; then
    AGQ_PID=$(cat "$AGQ_PID_FILE")
    if kill -0 $AGQ_PID 2>/dev/null; then
        log_info "Stopping AGQ (PID: $AGQ_PID)"
        kill $AGQ_PID 2>/dev/null || true
        sleep 1
        kill -9 $AGQ_PID 2>/dev/null || true
    fi
    rm -f "$AGQ_PID_FILE"
fi

# Clean up any stray processes (careful with this)
pkill -f "agq.*--port $TEST_PORT" 2>/dev/null || true
pkill -f "agw.*test-worker" 2>/dev/null || true

# Remove log files (optional, comment out to preserve for debugging)
# rm -f "$AGQ_LOG" "$AGW_LOG"

log_info "Cleanup complete"
