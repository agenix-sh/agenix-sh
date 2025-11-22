#!/bin/bash
set -e

# Kill any running instances forcefully
echo "Killing existing instances..."
pkill -9 -f "target/debug/agq" || true
pkill -9 -f "target/debug/agw" || true
pkill -9 -f "target/debug/agw" || true
sleep 2

# Clean DB
echo "Cleaning DB..."
rm -f ~/.agq/data.redb

# Build
echo "Building agq..."
cargo build --manifest-path agq/Cargo.toml
echo "Building agw..."
cargo build --manifest-path agw/Cargo.toml

# Fixed Session Key (32 bytes hex)
SESSION_KEY="0000000000000000000000000000000000000000000000000000000000000000"

# Start AGQ
echo "Starting AGQ..."
RUST_LOG=debug ./agq/target/debug/agq --session-key "$SESSION_KEY" > agq.log 2>&1 &
AGQ_PID=$!

# Wait for AGQ to be ready
echo "Waiting for AGQ to start..."
for i in {1..10}; do
    if nc -z localhost 6379; then
        echo "AGQ is up!"
        break
    fi
    sleep 1
done

# Start AGW
echo "Starting AGW..."
export AGQ_SESSION_KEY="$SESSION_KEY"
RUST_LOG=debug ./agw/target/debug/agw > agw.log 2>&1 &
AGW_PID=$!
sleep 2

# Generate Plan Payload
# Note: We write directly to file to avoid $(...) stripping trailing newlines
printf "*2\r\n\$4\r\nAUTH\r\n\$64\r\n%s\r\n" "$SESSION_KEY" > plan_payload.bin
PLAN_JSON='{"plan_id":"test-plan-001","plan_description":"Test Plan","tasks":[{"task_number":1,"command":"echo","args":["Hello World"],"timeout_secs":10}]}'
printf "*2\r\n\$11\r\nPLAN.SUBMIT\r\n\$%d\r\n%s\r\n" ${#PLAN_JSON} "$PLAN_JSON" >> plan_payload.bin

python3 submit_job.py plan_payload.bin
sleep 1

# Submit an Action
echo "Submitting Action..."
ACTION_JSON='{"action_id":"test-action-001","plan_id":"test-plan-001","inputs":[{"file":"/tmp/test"}]}'

# Generate Action Payload
printf "*2\r\n\$4\r\nAUTH\r\n\$64\r\n%s\r\n" "$SESSION_KEY" > action_payload.bin
printf "*2\r\n\$13\r\nACTION.SUBMIT\r\n\$%d\r\n%s\r\n" ${#ACTION_JSON} "$ACTION_JSON" >> action_payload.bin

python3 submit_job.py action_payload.bin
sleep 5

# Check logs (we should see output from AGW)
echo "Checking for AGW activity..."
# Since we backgrounded them, their stdout should be in this shell.

kill $AGQ_PID
kill $AGW_PID
