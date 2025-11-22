#!/bin/bash
SESSION_KEY="0000000000000000000000000000000000000000000000000000000000000000"
AUTH_CMD=$(printf "*2\r\n\$4\r\nAUTH\r\n\$64\r\n%s\r\n" "$SESSION_KEY")
PLAN_JSON='{"plan_id":"test-plan-001","plan_description":"Test Plan","tasks":[{"task_number":1,"command":"echo","args":["Hello World"],"timeout_secs":10}]}'
PLAN_CMD=$(printf "*2\r\n\$11\r\nPLAN.SUBMIT\r\n\$%d\r\n%s\r\n" ${#PLAN_JSON} "$PLAN_JSON")

echo -ne "${AUTH_CMD}${PLAN_CMD}" > payload.bin
ls -l payload.bin
cat -v payload.bin
