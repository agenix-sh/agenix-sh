# Integration Checkpoint 1 (ICP-1): Minimal End-to-End Job Execution

**Version:** 1.0
**Status:** In Progress
**Target Date:** 2025-11-19
**Owner:** AGEniX Core Team

---

## Overview

ICP-1 validates the **minimal viable integration** between AGX (Planner), AGQ (Queue), and AGW (Worker) components. This checkpoint ensures a simple job can flow through the entire system from submission to completion.

### Goal

Demonstrate that:
- AGX can generate and submit a job to AGQ
- AGQ can queue the job and make it available to workers
- AGW can fetch, execute, and post results back to AGQ
- The system handles errors gracefully

### Non-Goals (Phase 2+)

- Job memory/shared state (AGQ-016/017/018)
- Action semantics (fan-out multiple jobs)
- Plan storage/versioning
- Interactive REPL interface
- Agentic Units (AU) beyond Unix tools
- Multi-machine distributed execution
- Workflow orchestration

---

## Current State Assessment

### AGX (Planner) - ✅ Ready

**Completed:**
- ✅ Echo/Delta model integration (VibeThinker-1.5B)
- ✅ PLAN submit via RESP protocol
- ✅ Job envelope generation (conforms to job-schema.md v0.2)
- ✅ RESP client with authentication
- ✅ Task→Plan→Job nomenclature alignment

**Not Required for ICP-1:**
- ⏸️ REPL interface (AGX-042) - Can use CLI directly
- ⏸️ Action semantics (AGX-044) - Single job execution only

**AGX is ready for ICP-1 testing.**

### AGQ (Queue) - ⚠️ Partially Ready

**Completed:**
- ✅ RESP listener (TCP server)
- ✅ Session-key authentication
- ✅ Embedded storage backend (redb)
- ✅ ZADD scheduling queue (AGQ-005)
- ✅ HASH job metadata storage (AGQ-006)
- ✅ LPUSH/BRPOP queue operations (AGQ-004)
- ✅ RPOPLPUSH atomic transitions (AGQ-011)

**Blockers for ICP-1:**
- ❌ **AGQ-007**: Worker heartbeat registry - **IN PROGRESS**
- ❌ **AGQ-008**: RESP command router - **CRITICAL BLOCKER**
- ❌ **AGQ-009**: PLAN submission endpoint - **CRITICAL BLOCKER**

**AGQ needs 3 issues resolved before ICP-1.**

### AGW (Worker) - ✅ Ready

**Completed:**
- ✅ RESP client with authentication
- ✅ Job fetching (BRPOP)
- ✅ Multi-task execution engine (AGW-011)
- ✅ Task→Plan→Job nomenclature alignment (AGW-012)
- ✅ Result posting to AGQ (AGW-007)
- ✅ Tool availability metadata (AGW-008)
- ✅ Graceful shutdown (AGW-009)

**Not Required for ICP-1:**
- ⏸️ Job memory access (AGW-020) - Simple jobs don't need shared state

**AGW is ready for ICP-1 testing.**

---

## Critical Path to ICP-1

### Phase 1: Complete AGQ Blockers (Est. 7-10 hours)

#### AGQ-007: Worker Heartbeat Registry (2-3 hours)
**Status:** In Progress (separate session)

**Implementation:**
```rust
// RESP command: SET worker:<id>:alive 1 EX 10
// Workers send heartbeat every 5 seconds
// AGQ expires stale workers after 10 seconds
```

**Acceptance Criteria:**
- [ ] Workers can register with `SET worker:<id>:alive 1 EX 10`
- [ ] AGQ tracks active workers
- [ ] Expired workers automatically removed
- [ ] Worker list queryable via `KEYS worker:*:alive`

#### AGQ-008: RESP Command Router (3-4 hours)
**Status:** Blocked by AGQ-007

**Implementation:**
```rust
// Parse RESP commands and route to handlers:
// - AUTH <session-key>
// - PING
// - SET <key> <value> [EX <seconds>]
// - GET <key>
// - LPUSH <queue> <value>
// - BRPOP <queue> <timeout>
// - ZADD <set> <score> <member>
// - HSET <hash> <field> <value>
// - HGET <hash> <field>
// - RPOPLPUSH <src> <dst>
```

**Acceptance Criteria:**
- [ ] Routes all implemented RESP commands
- [ ] Authentication required before non-AUTH commands
- [ ] Error responses for unknown commands
- [ ] Error responses for malformed commands
- [ ] Unit tests for all routing paths

#### AGQ-009: PLAN Submission Endpoint (2-3 hours)
**Status:** Blocked by AGQ-008

**Implementation:**
```rust
// Custom RESP command: PLAN.SUBMIT <job-json>
// 1. Validate job envelope against job-schema.md
// 2. Store job metadata in HASH (job:<id>:*)
// 3. Push to queue via LPUSH job:queue <job-id>
// 4. Return job_id to client
```

**Acceptance Criteria:**
- [ ] Accepts job envelope matching job-schema.md v0.2
- [ ] Validates required fields (job_id, plan_id, tasks)
- [ ] Validates task numbering (contiguous, 1-based)
- [ ] Validates input_from_task references
- [ ] Stores job metadata in `job:<id>:*` HASH
- [ ] Enqueues job_id in job:queue LIST
- [ ] Returns job_id on success
- [ ] Returns error on validation failure

### Phase 2: Integration Testing (Est. 1 day)

Create integration tests in `/Users/lewis/work/agenix-sh/agenix/tests/integration/icp-1/`

---

## Test Scenarios

### Test 1: Simple Single-Task Job

**Scenario:** Execute a single Unix command via AGW

**Job Payload:**
```json
{
  "job_id": "test-simple-001",
  "plan_id": "echo-hello",
  "plan_description": "Echo hello world",
  "tasks": [
    {
      "task_number": 1,
      "command": "echo",
      "args": ["hello", "world"],
      "timeout_secs": 5
    }
  ]
}
```

**Test Steps:**
1. Start AGQ: `agq --port 6379 --session-key test-key-123`
2. Start AGW: `agw --worker-id test-worker --agq-addr 127.0.0.1:6379 --session-key test-key-123`
3. Submit job via AGX: `agx plan submit <job.json>`
4. Wait for completion (max 10 seconds)
5. Query status: `agx job status test-simple-001`
6. Retrieve output: `agx job stdout test-simple-001`

**Expected Results:**
- Job status: `completed`
- Stdout: `hello world\n`
- Stderr: empty
- Exit code: 0

**Failure Modes to Test:**
- AGQ not running → AGX connection error
- AGW not running → Job stays in queue
- Invalid command → Job fails with error in stderr
- Timeout exceeded → Job fails with timeout status

---

### Test 2: Multi-Task Pipeline

**Scenario:** Execute tasks with stdin/stdout piping

**Job Payload:**
```json
{
  "job_id": "test-pipeline-001",
  "plan_id": "sort-uniq",
  "plan_description": "Sort lines and remove duplicates",
  "tasks": [
    {
      "task_number": 1,
      "command": "sort",
      "args": ["-r"],
      "timeout_secs": 30
    },
    {
      "task_number": 2,
      "command": "uniq",
      "args": [],
      "input_from_task": 1,
      "timeout_secs": 30
    }
  ]
}
```

**Input Data (stdin to task 1):**
```
apple
banana
apple
cherry
banana
```

**Test Steps:**
1. Submit job with input data
2. AGW fetches and executes
3. Task 1: Sorts in reverse → `cherry\nbanana\napple\napple\nbanana`
4. Task 2: Gets sorted output as stdin, runs uniq → `cherry\nbanana\napple`

**Expected Results:**
- Job status: `completed`
- Stdout (final): `cherry\nbanana\napple\n`
- Both tasks succeeded

**Validation:**
- Task execution order maintained
- Data piping between tasks works
- Final output matches expected

---

### Test 3: Worker Heartbeat and Lifecycle

**Scenario:** Validate worker registration, heartbeat, and cleanup

**Test Steps:**
1. Start AGQ
2. Query workers (should be empty): `KEYS worker:*:alive` → `[]`
3. Start AGW
4. Verify worker registered: `KEYS worker:*:alive` → `["worker:test-worker:alive"]`
5. Wait 11 seconds (heartbeat expires after 10s)
6. Verify worker expired: `KEYS worker:*:alive` → `[]`
7. Restart AGW
8. Verify re-registered: `KEYS worker:*:alive` → `["worker:test-worker:alive"]`
9. Graceful shutdown AGW (SIGTERM)
10. Verify worker cleaned up: `KEYS worker:*:alive` → `[]`

**Expected Results:**
- Workers heartbeat every 5 seconds
- Stale workers expire after 10 seconds
- Graceful shutdown removes worker immediately

---

### Test 4: Error Handling

**Scenario A: Invalid Command**
```json
{
  "job_id": "test-error-001",
  "tasks": [
    {"task_number": 1, "command": "nonexistent-tool", "args": []}
  ]
}
```
- Expected: Job fails, stderr contains "command not found"

**Scenario B: Timeout Exceeded**
```json
{
  "job_id": "test-timeout-001",
  "tasks": [
    {"task_number": 1, "command": "sleep", "args": ["100"], "timeout_secs": 2}
  ]
}
```
- Expected: Job fails after 2 seconds, status includes "timeout"

**Scenario C: Task Dependency Error**
```json
{
  "job_id": "test-dependency-001",
  "tasks": [
    {"task_number": 1, "command": "false", "args": []},
    {"task_number": 2, "command": "echo", "args": ["should not run"], "input_from_task": 1}
  ]
}
```
- Expected: Task 1 fails, task 2 never executes, job marked failed

**Scenario D: Worker Dies Mid-Job**
1. Submit long-running job (e.g., `sleep 30`)
2. Kill AGW with SIGKILL (not graceful)
3. Job should remain in queue or be marked orphaned
4. Start new AGW
5. New worker should detect and handle orphaned job

---

### Test 5: Job Status Query

**Scenario:** Query job at different lifecycle stages

**Test Steps:**
1. Submit job
2. Query immediately → Status: `pending` or `running`
3. Wait for completion
4. Query again → Status: `completed`
5. Query stdout → Contains expected output
6. Query stderr → Empty (for successful job)
7. Query non-existent job → Error: "job not found"

**RESP Commands Used:**
```redis
HGET job:test-001:metadata status        # pending/running/completed/failed
HGET job:test-001:metadata stdout        # Command output
HGET job:test-001:metadata stderr        # Error output
HGET job:test-001:metadata exit_code     # Process exit code
HGET job:test-001:metadata started_at    # Timestamp
HGET job:test-001:metadata completed_at  # Timestamp
```

---

## Data Flow Validation

### Flow 1: Successful Job Execution

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. AGX: Generate Plan                                           │
│    - User: "sort lines in reverse"                              │
│    - Echo model: Generate high-level plan                       │
│    - Delta model: Validate and add tool details                 │
│    - Output: Job envelope JSON                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. AGX → AGQ: Submit Job                                        │
│    RESP: PLAN.SUBMIT <job-json>                                 │
│    Response: +OK job_id                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. AGQ: Store and Queue                                         │
│    - Validate job envelope                                      │
│    - HSET job:<id>:metadata (status=pending, ...)               │
│    - LPUSH job:queue <job-id>                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. AGW: Fetch Job                                               │
│    - Worker heartbeat: SET worker:<id>:alive 1 EX 10            │
│    - Blocking fetch: BRPOP job:queue 5                          │
│    - Receives: job-id                                           │
│    - Fetch full job: HGET job:<id>:metadata json                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. AGW: Execute Tasks                                           │
│    - Mark running: HSET job:<id>:metadata status running        │
│    - For each task:                                             │
│      1. Spawn process (command + args)                          │
│      2. Pipe stdin (from input_from_task if specified)          │
│      3. Capture stdout/stderr                                   │
│      4. Apply timeout                                           │
│    - If all succeed → status=completed                          │
│    - If any fail → status=failed                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. AGW → AGQ: Post Results                                      │
│    HSET job:<id>:metadata status completed                      │
│    HSET job:<id>:metadata stdout <output>                       │
│    HSET job:<id>:metadata stderr <errors>                       │
│    HSET job:<id>:metadata exit_code 0                           │
│    HSET job:<id>:metadata completed_at <timestamp>              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 7. AGX: Query Results                                           │
│    HGET job:<id>:metadata status     → "completed"              │
│    HGET job:<id>:metadata stdout     → "hello world\n"          │
│    Display to user                                              │
└─────────────────────────────────────────────────────────────────┘
```

### Flow 2: Failed Job (Timeout)

```
AGX → AGQ (submit) → AGW (fetch) → AGW (execute task 1)
                                   ↓
                         Task exceeds timeout_secs
                                   ↓
                         Kill process (SIGKILL)
                                   ↓
                  HSET job:<id>:metadata status failed
                  HSET job:<id>:metadata stderr "Task 1 exceeded timeout"
                                   ↓
                         AGX queries → status=failed
```

### Flow 3: Worker Dies (Orphaned Job)

```
AGX → AGQ (submit) → AGW (fetch) → AGW (executing)
                                        ↓
                               Worker crashes (SIGKILL)
                                        ↓
                            Job status remains "running"
                            Worker heartbeat expires
                                        ↓
                   AGQ detects stale job (no heartbeat for 10s)
                                        ↓
                   Option A: Re-queue job (AGQ-012, not in ICP-1)
                   Option B: Mark as "orphaned" (manual intervention)
```

**ICP-1 Acceptance:** Jobs can become orphaned. Retry logic is Phase 2 (AGQ-012).

---

## Integration Test Implementation

### Test Harness Structure

```
agenix/
└── tests/
    └── integration/
        └── icp-1/
            ├── README.md                 # Test overview
            ├── test_simple_job.sh        # Test 1: Single task
            ├── test_pipeline.sh          # Test 2: Multi-task
            ├── test_worker_lifecycle.sh  # Test 3: Heartbeat
            ├── test_error_handling.sh    # Test 4: Failures
            ├── test_status_query.sh      # Test 5: Query API
            ├── fixtures/
            │   ├── simple-job.json
            │   ├── pipeline-job.json
            │   └── sample-input.txt
            └── helpers/
                ├── start_agq.sh
                ├── start_agw.sh
                └── cleanup.sh
```

### Example Test Script

```bash
#!/bin/bash
# tests/integration/icp-1/test_simple_job.sh

set -e
source ./helpers/start_agq.sh
source ./helpers/start_agw.sh

echo "=== ICP-1 Test 1: Simple Single-Task Job ==="

# Start components
SESSION_KEY="test-key-$(openssl rand -hex 16)"
echo "Starting AGQ..."
AGQ_PID=$(start_agq --port 6379 --session-key "$SESSION_KEY")

echo "Starting AGW..."
AGW_PID=$(start_agw --worker-id test-worker --session-key "$SESSION_KEY")

# Wait for initialization
sleep 2

# Submit job
echo "Submitting job..."
JOB_ID=$(agx plan submit --file fixtures/simple-job.json --session-key "$SESSION_KEY")
echo "Job ID: $JOB_ID"

# Poll for completion (max 30 seconds)
for i in {1..30}; do
  STATUS=$(agx job status "$JOB_ID" --session-key "$SESSION_KEY" | jq -r .status)
  echo "Status: $STATUS"

  if [[ "$STATUS" == "completed" ]]; then
    echo "✅ Job completed successfully"
    break
  elif [[ "$STATUS" == "failed" ]]; then
    echo "❌ Job failed"
    exit 1
  fi

  sleep 1
done

# Verify output
STDOUT=$(agx job stdout "$JOB_ID" --session-key "$SESSION_KEY")
EXPECTED="hello world"

if [[ "$STDOUT" == "$EXPECTED" ]]; then
  echo "✅ Output matches expected"
else
  echo "❌ Output mismatch"
  echo "Expected: $EXPECTED"
  echo "Got: $STDOUT"
  exit 1
fi

# Cleanup
kill $AGW_PID $AGQ_PID
echo "=== Test 1 PASSED ==="
```

---

## Acceptance Criteria

### ✅ ICP-1 is COMPLETE when:

#### Core Functionality
- [ ] **Job Submission**: AGX successfully submits job to AGQ
- [ ] **Job Validation**: AGQ validates job envelope against job-schema.md
- [ ] **Job Queuing**: AGQ stores job metadata and enqueues job_id
- [ ] **Worker Registration**: AGW registers via heartbeat
- [ ] **Job Fetch**: AGW fetches job from queue (BRPOP)
- [ ] **Task Execution**: AGW executes all tasks in sequence
- [ ] **Data Piping**: stdout from task N becomes stdin for task N+1
- [ ] **Result Storage**: AGW posts stdout/stderr/status to AGQ
- [ ] **Status Query**: AGX queries job status successfully
- [ ] **Output Retrieval**: AGX retrieves stdout/stderr from AGQ

#### Error Handling
- [ ] **Invalid Command**: Jobs with non-existent commands fail gracefully
- [ ] **Timeout**: Tasks exceeding timeout_secs are killed and marked failed
- [ ] **Failed Task**: Job stops on first task failure
- [ ] **Validation Error**: Malformed jobs rejected at submission
- [ ] **Connection Error**: AGX handles AGQ unreachable gracefully
- [ ] **Worker Death**: Orphaned jobs remain queryable (retry not required)

#### Worker Lifecycle
- [ ] **Heartbeat**: Workers send heartbeat every 5 seconds
- [ ] **Expiry**: Stale workers expire after 10 seconds
- [ ] **Discovery**: Active workers queryable via `KEYS worker:*:alive`
- [ ] **Graceful Shutdown**: Workers cleanup on SIGTERM
- [ ] **Crash Handling**: Workers can crash without corrupting AGQ state

#### Integration Tests
- [ ] **Test 1**: Simple single-task job passes
- [ ] **Test 2**: Multi-task pipeline passes
- [ ] **Test 3**: Worker lifecycle test passes
- [ ] **Test 4**: Error handling tests pass
- [ ] **Test 5**: Status query tests pass
- [ ] **All tests**: Can run in CI/CD without manual intervention

#### Documentation
- [ ] **ICP-1 document**: This document complete and reviewed
- [ ] **Test plan**: All test scenarios documented
- [ ] **Known limitations**: Documented below
- [ ] **Troubleshooting guide**: Common issues documented

---

## Known Limitations (Deferred to Phase 2)

### Not Implemented in ICP-1

1. **Job Memory** (AGQ-016/017/018)
   - No shared state between tasks
   - No MapReduce-style counters
   - Tasks communicate via stdin/stdout only

2. **Action Semantics** (AGX-044)
   - No fan-out (one Plan → many Jobs)
   - Each job submission is independent

3. **Plan Storage** (AGQ-014)
   - Plans not stored in AGQ
   - Each job contains full plan definition
   - No plan versioning or reuse

4. **Retry Logic** (AGQ-012)
   - Failed jobs not automatically retried
   - Orphaned jobs require manual intervention

5. **REPL Interface** (AGX-042)
   - Command-line only
   - No interactive plan building

6. **Distributed Workers**
   - Workers assumed on same machine or network
   - No load balancing or affinity

7. **Agentic Units (AU)**
   - Only Unix commands supported
   - No custom AU integration

8. **Workflow Orchestration**
   - No conditional logic
   - No loops or branches
   - Sequential execution only

---

## DGX Deployment Validation

### Environment Setup

**Hardware:** NVIDIA DGX Station
**OS:** Ubuntu 22.04
**Rust:** 1.82+

**Build Commands:**
```bash
# Build all components in release mode
cd /path/to/agx && cargo build --release
cd /path/to/agq && cargo build --release
cd /path/to/agw && cargo build --release
```

### Deployment Test

**Step 1: Start AGQ**
```bash
./target/release/agq \
  --port 6379 \
  --session-key $(openssl rand -hex 32) \
  --log-level info
```

**Step 2: Start AGW**
```bash
./target/release/agw \
  --worker-id dgx-worker-1 \
  --agq-addr 127.0.0.1:6379 \
  --session-key <same-as-agq> \
  --log-level info
```

**Step 3: Submit Job from AGX**
```bash
echo "Sort and count unique lines" | \
./target/release/agx plan new | \
./target/release/agx plan submit --agq-addr 127.0.0.1:6379 --session-key <same>
```

**Step 4: Monitor Logs**
- AGQ: Should show job received, queued
- AGW: Should show job fetched, executed, results posted
- AGX: Should show job completed

**Step 5: Query Results**
```bash
./target/release/agx job status <job-id>
./target/release/agx job stdout <job-id>
```

### DGX-Specific Validations

- [ ] All binaries run on DGX without errors
- [ ] CUDA libraries not required (CPU-only for Phase 1)
- [ ] Network communication works (localhost)
- [ ] File I/O works (temp directories)
- [ ] Multi-core execution (workers can run in parallel)

---

## Troubleshooting Guide

### Issue: AGX cannot connect to AGQ

**Symptoms:**
```
Error: Connection refused (os error 111)
```

**Diagnosis:**
1. Check AGQ is running: `ps aux | grep agq`
2. Check port: `netstat -tuln | grep 6379`
3. Check firewall: `sudo ufw status`

**Solution:**
- Ensure AGQ started successfully
- Verify correct port in AGX command
- Check network connectivity

---

### Issue: AGW not fetching jobs

**Symptoms:**
- Jobs stay in "pending" status
- AGW logs show no activity

**Diagnosis:**
1. Check worker heartbeat: `redis-cli KEYS worker:*:alive`
2. Check queue contents: `redis-cli LLEN job:queue`
3. Check authentication: Look for auth errors in AGW logs

**Solution:**
- Verify session key matches between AGQ and AGW
- Restart AGW with correct session key
- Check AGQ logs for rejected authentication

---

### Issue: Job marked "running" but never completes

**Symptoms:**
- Job status stuck in "running"
- AGW logs show execution started but no completion

**Diagnosis:**
1. Check if AGW process still alive: `ps aux | grep agw`
2. Check job execution logs: `agx job stderr <job-id>`
3. Check for timeout issues

**Solution:**
- If AGW crashed: Restart AGW (job is orphaned, retry in Phase 2)
- If task hanging: Check timeout_secs value
- If command blocking: Verify command doesn't require interactive input

---

### Issue: Tasks not piping correctly

**Symptoms:**
- Multi-task job fails
- Stderr shows "unexpected input"

**Diagnosis:**
1. Check job envelope: Verify `input_from_task` references correct task
2. Check task numbering: Must be 1-based, contiguous
3. Check task 1 output: `agx job stdout <job-id>` (if only task 1 ran)

**Solution:**
- Fix job envelope: Correct `input_from_task` references
- Ensure tasks are numbered 1, 2, 3, ... (no gaps)
- Test each command independently first

---

## Next Steps After ICP-1

Once ICP-1 acceptance criteria are met, proceed to **Integration Checkpoint 2 (ICP-2)**.

### Potential ICP-2 Scope

1. **Job Memory** (AGQ-016/017/018)
   - Shared state between tasks
   - MapReduce-style counters

2. **Action Semantics** (AGX-044)
   - Fan-out multiple jobs from one plan
   - Parallel execution

3. **Plan Storage** (AGQ-014)
   - Store plans in AGQ
   - Plan versioning and reuse

4. **Retry Logic** (AGQ-012)
   - Automatic retry on failure
   - Orphaned job recovery

5. **REPL Interface** (AGX-042)
   - Interactive plan building
   - Iterative refinement

6. **First Agentic Unit**
   - Integrate agx-ocr or simple AU
   - Test AU execution model

7. **Distributed Workers**
   - Multiple AGW instances
   - Load balancing

---

## References

- **Execution Layers:** `/Users/lewis/work/agenix-sh/agenix/docs/architecture/execution-layers.md`
- **Job Schema:** `/Users/lewis/work/agenix-sh/agenix/docs/architecture/job-schema.md`
- **RESP Protocol:** `/Users/lewis/work/agenix-sh/agenix/docs/api/resp-protocol.md`
- **AGX Issues:** https://github.com/agenix-sh/agx/issues
- **AGQ Issues:** https://github.com/agenix-sh/agq/issues
- **AGW Issues:** https://github.com/agenix-sh/agw/issues

---

## Revision History

| Version | Date       | Author      | Changes                          |
|---------|------------|-------------|----------------------------------|
| 1.0     | 2025-11-17 | AGEniX Team | Initial ICP-1 document           |

---

**Status:** Ready for AGQ blocker resolution (AGQ-007, AGQ-008, AGQ-009)
**Next Review:** After AGQ-009 completion
**Target ICP-1 Completion:** 2025-11-19
