# AGQ API Endpoints

**Version:** 1.0
**Status:** Specification (Phase 2 - Planned)
**Last Updated:** 2025-11-17

This document specifies AGQ-specific RESP commands for plan submission, job management, and worker coordination.

---

## Table of Contents

1. [Overview](#overview)
2. [Plan Management](#plan-management)
3. [Job Management](#job-management)
4. [Action Management](#action-management)
5. [Worker Management](#worker-management)
6. [Queue Operations](#queue-operations)
7. [Observability](#observability)

---

## 1. Overview

AGQ extends the base RESP protocol with domain-specific commands for managing Plans, Jobs, Actions, and Workers.

### Command Namespaces

| Namespace | Purpose | Examples |
|-----------|---------|----------|
| `PLAN.*` | Plan storage and retrieval | `PLAN.SUBMIT`, `PLAN.GET` |
| `JOB.*` | Job lifecycle management | `JOB.STATUS`, `JOB.UPDATE`, `JOB.LIST` |
| `ACTION.*` | Batch job creation | `ACTION.SUBMIT`, `ACTION.STATUS` |
| `WORKER.*` | Worker registration/heartbeat | `WORKER.REGISTER`, `WORKER.HEARTBEAT` |
| `QUEUE.*` | Queue inspection | `QUEUE.STATS`, `QUEUE.PEEK` |

### Authentication

All commands require prior `AUTH` authentication. See [resp-protocol.md](./resp-protocol.md#authentication).

---

## 2. Plan Management

### PLAN.SUBMIT

**Status**: Phase 2 (Planned)

**Syntax**: `PLAN.SUBMIT <plan_json>`

**Description**: Store a reusable Plan definition in AGQ

**Parameters**:
- `plan_json` (bulk string): JSON-encoded Plan per [job-schema.md](../architecture/job-schema.md)

**Plan Structure**:
```json
{
  "plan_id": "uuid-5678",
  "plan_description": "Sort and deduplicate data",
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

**Response**:
- Success: `+OK plan_id=<uuid>`
- Error: `-ERR Invalid plan schema: <validation_error>`
- Error: `-ERR Plan already exists: <plan_id>`

**Validation**:
- JSON schema compliance
- `plan_id` is unique
- Tasks array non-empty
- `task_number` 1-based, contiguous
- `input_from_task` references valid previous tasks
- Maximum task count (default: 100)
- Command names are non-empty strings

**Storage**:
- Key: `plan:<plan_id>`
- Value: JSON Plan definition
- Indexed by: `plan_id`

**Example**:
```bash
redis-cli -p 6380
> AUTH my-session-key-64-hex-chars
+OK
> PLAN.SUBMIT '{"plan_id":"uuid-5678",...}'
+OK plan_id=uuid-5678
```

---

### PLAN.GET

**Status**: Phase 2 (Planned)

**Syntax**: `PLAN.GET <plan_id>`

**Description**: Retrieve a stored Plan definition

**Parameters**:
- `plan_id` (bulk string): Unique plan identifier

**Response**:
- Success: `$<length>\r\n<plan_json>\r\n`
- Not found: `$-1\r\n` (nil)

**Example**:
```resp
Client: *2\r\n$8\r\nPLAN.GET\r\n$9\r\nuuid-5678\r\n
Server: $123\r\n{"plan_id":"uuid-5678","tasks":[...]}\r\n
```

---

### PLAN.LIST

**Status**: Phase 3 (Future)

**Syntax**: `PLAN.LIST [pattern] [LIMIT <count>] [OFFSET <start>]`

**Description**: List all stored Plans with optional filtering

**Parameters**:
- `pattern` (optional): Glob pattern for `plan_id` matching (e.g., `ocr-*`)
- `LIMIT` (optional): Maximum results (default: 100)
- `OFFSET` (optional): Skip first N results (default: 0)

**Response**:
- Array of plan IDs: `*<count>\r\n$<len1>\r\n<plan_id1>\r\n...`
- Empty: `*0\r\n`

---

### PLAN.DELETE

**Status**: Phase 3 (Future)

**Syntax**: `PLAN.DELETE <plan_id>`

**Description**: Remove a Plan definition (only if no active Jobs reference it)

**Response**:
- Success: `:1\r\n` (deleted)
- Not found: `:0\r\n`
- Error: `-ERR Cannot delete plan with active jobs`

---

## 3. Job Management

### JOB.STATUS

**Status**: Phase 2 (Planned)

**Syntax**: `JOB.STATUS <job_id>`

**Description**: Query detailed Job execution status and results

**Parameters**:
- `job_id` (bulk string): Unique job identifier

**Response**:
- Success: `$<length>\r\n<job_status_json>\r\n`
- Not found: `$-1\r\n` (nil)

**Job Status Structure**:
```json
{
  "job_id": "job-abc123",
  "action_id": "action-001",
  "plan_id": "uuid-5678",
  "status": "completed",
  "created_at": "2025-11-17T10:00:00Z",
  "started_at": "2025-11-17T10:00:05Z",
  "completed_at": "2025-11-17T10:00:15Z",
  "worker_id": "worker-local-001",
  "task_results": [
    {
      "task_number": 1,
      "command": "sort",
      "exit_code": 0,
      "stdout": "line1\nline2\n",
      "stderr": "",
      "duration_ms": 120
    },
    {
      "task_number": 2,
      "command": "uniq",
      "exit_code": 0,
      "stdout": "line1\nline2\n",
      "stderr": "",
      "duration_ms": 85
    }
  ]
}
```

**Status Values**:
- `pending`: Job created, waiting in queue
- `running`: Job pulled by worker, executing
- `completed`: All tasks succeeded
- `failed`: One or more tasks failed
- `dead`: Failed after max retries

**Example**:
```bash
> JOB.STATUS job-abc123
{"job_id":"job-abc123","status":"completed",...}
```

---

### JOB.UPDATE

**Status**: Phase 2 (Planned, AGW internal use)

**Syntax**: `JOB.UPDATE <job_id> <update_json>`

**Description**: Worker reports Job progress/completion (AGW → AGQ only)

**Parameters**:
- `job_id` (bulk string): Job identifier
- `update_json` (bulk string): Status update payload

**Update Structure**:
```json
{
  "status": "running",
  "worker_id": "worker-local-001",
  "started_at": "2025-11-17T10:00:05Z",
  "current_task": 2
}
```

**Or on completion**:
```json
{
  "status": "completed",
  "completed_at": "2025-11-17T10:00:15Z",
  "task_results": [...]
}
```

**Response**:
- Success: `+OK`
- Error: `-ERR Job not found: <job_id>`
- Error: `-ERR Invalid status transition: <from> -> <to>`

**Access Control**: Only the worker assigned to the job can update it

---

### JOB.LIST

**Status**: Phase 2 (Planned)

**Syntax**: `JOB.LIST <action_id> [status]`

**Description**: List all Jobs for a given Action, optionally filtered by status

**Parameters**:
- `action_id` (bulk string): Action identifier
- `status` (optional): Filter by status (`pending`, `running`, `completed`, `failed`)

**Response**:
- Array of job IDs: `*<count>\r\n$<len1>\r\n<job_id1>\r\n...`
- Empty: `*0\r\n`

**Example**:
```bash
> JOB.LIST action-001
*3
$11
job-abc123
$11
job-def456
$11
job-ghi789

> JOB.LIST action-001 failed
*1
$11
job-def456
```

---

### JOB.RETRY

**Status**: Phase 3 (Future)

**Syntax**: `JOB.RETRY <job_id>`

**Description**: Re-enqueue a failed Job for retry

**Response**:
- Success: `+OK job re-queued`
- Error: `-ERR Job not in failed status`
- Error: `-ERR Max retries exceeded`

---

## 4. Action Management

### ACTION.SUBMIT

**Status**: Phase 2 (Planned)

**Syntax**: `ACTION.SUBMIT <action_json>`

**Description**: Create multiple Jobs from one Plan with different inputs (batch processing)

**Parameters**:
- `action_json` (bulk string): JSON with `plan_id` and array of `inputs`

**Action Structure**:
```json
{
  "action_id": "action-001",
  "plan_id": "uuid-5678",
  "inputs": [
    {"file": "data1.txt", "output": "result1.txt"},
    {"file": "data2.txt", "output": "result2.txt"},
    {"file": "data3.txt", "output": "result3.txt"}
  ]
}
```

**Behavior**:
1. Validates `plan_id` exists
2. Creates N Jobs (one per input element)
3. Each Job gets unique `job_id`, same `plan_id` and `action_id`
4. All Jobs enqueued to `queue:ready` (immediate) or `queue:scheduled` (future)
5. Returns Action ID for tracking

**Response**:
- Success: `+OK action_id=<uuid> jobs_created=<count>`
- Error: `-ERR Plan not found: <plan_id>`
- Error: `-ERR Invalid action schema: <details>`
- Error: `-ERR Too many inputs: max <limit>`

**Example**:
```bash
> ACTION.SUBMIT '{"action_id":"action-001","plan_id":"uuid-5678","inputs":[...]}'
+OK action_id=action-001 jobs_created=3
```

---

### ACTION.STATUS

**Status**: Phase 2 (Planned)

**Syntax**: `ACTION.STATUS <action_id>`

**Description**: Get aggregate status of all Jobs in an Action

**Response**:
```json
{
  "action_id": "action-001",
  "plan_id": "uuid-5678",
  "total_jobs": 100,
  "pending": 10,
  "running": 30,
  "completed": 55,
  "failed": 5,
  "dead": 0,
  "created_at": "2025-11-17T10:00:00Z",
  "completed_jobs_at": "2025-11-17T10:05:00Z"
}
```

**Example**:
```bash
> ACTION.STATUS action-001
{"action_id":"action-001","total_jobs":100,"completed":55,...}
```

---

## 5. Worker Management

### WORKER.REGISTER

**Status**: Phase 2 (Planned)

**Syntax**: `WORKER.REGISTER <worker_json>`

**Description**: Register a worker with AGQ, declaring capabilities

**Parameters**:
- `worker_json` (bulk string): Worker registration payload

**Worker Registration Structure**:
```json
{
  "worker_id": "worker-local-001",
  "hostname": "macbook-pro.local",
  "capabilities": ["sort", "uniq", "grep", "agx-ocr"],
  "max_concurrent_jobs": 4,
  "agw_version": "0.1.0"
}
```

**Response**:
- Success: `+OK worker_id=<id> heartbeat_interval=30`
- Error: `-ERR Worker ID already registered`
- Error: `-ERR Invalid capabilities: <details>`

**Behavior**:
- Creates worker record: `worker:<worker_id>:metadata`
- Sets alive timestamp: `worker:<worker_id>:alive` (expires in 2x heartbeat interval)
- Stores capabilities for future job matching

**See Also**: [worker-registration.md](./worker-registration.md)

---

### WORKER.HEARTBEAT

**Status**: Phase 2 (Planned)

**Syntax**: `WORKER.HEARTBEAT <worker_id> [stats_json]`

**Description**: Worker sends periodic heartbeat to indicate it's alive

**Parameters**:
- `worker_id` (bulk string): Registered worker identifier
- `stats_json` (optional): Current worker statistics

**Stats Structure** (optional):
```json
{
  "active_jobs": 2,
  "completed_jobs_since_start": 150,
  "uptime_seconds": 3600,
  "cpu_usage_percent": 45.2,
  "memory_mb": 512
}
```

**Response**:
- Success: `+OK`
- Error: `-ERR Worker not registered: <worker_id>`

**Behavior**:
- Updates `worker:<worker_id>:alive` timestamp
- Optionally updates `worker:<worker_id>:stats`

**Heartbeat Interval**: Every 30 seconds (configurable)

**Timeout**: Worker marked dead if no heartbeat for 90 seconds (3x interval)

---

### WORKER.UNREGISTER

**Status**: Phase 2 (Planned)

**Syntax**: `WORKER.UNREGISTER <worker_id>`

**Description**: Gracefully unregister a worker (shutdown)

**Response**:
- Success: `+OK`
- Error: `-ERR Worker not registered`

**Behavior**:
- Removes worker metadata
- Any active jobs assigned to worker are re-queued
- Worker must finish existing jobs before unregistering (graceful shutdown)

---

## 6. Queue Operations

### QUEUE.STATS

**Status**: Phase 2 (Planned)

**Syntax**: `QUEUE.STATS [queue_name]`

**Description**: Get statistics for a queue (or all queues)

**Parameters**:
- `queue_name` (optional): Specific queue (default: all)

**Response**:
```json
{
  "queue:ready": {
    "length": 42,
    "oldest_job_age_seconds": 15,
    "newest_job_age_seconds": 0
  },
  "queue:scheduled": {
    "length": 100,
    "next_job_due_in_seconds": 30
  },
  "workers": {
    "total": 10,
    "active": 8,
    "idle": 2
  }
}
```

---

### QUEUE.PEEK

**Status**: Phase 3 (Future)

**Syntax**: `QUEUE.PEEK <queue_name> [count]`

**Description**: View jobs in queue without removing them

**Parameters**:
- `queue_name` (bulk string): Queue to inspect
- `count` (optional integer): Number of jobs to peek (default: 10)

**Response**:
- Array of job IDs: `*<count>\r\n$<len1>\r\n<job_id1>\r\n...`

---

## 7. Observability

### JOBS.SEARCH

**Status**: Phase 3 (Future)

**Syntax**: `JOBS.SEARCH [status] [plan_id] [LIMIT <n>]`

**Description**: Search jobs by status, plan_id, time range

**Parameters**:
- `status` (optional): Filter by status
- `plan_id` (optional): Filter by plan
- `LIMIT` (optional): Max results

**Response**: Array of matching job IDs

---

### METRICS

**Status**: Phase 3 (Future)

**Syntax**: `METRICS [format]`

**Description**: Export metrics in Prometheus format

**Parameters**:
- `format` (optional): `prometheus` | `json` (default: prometheus)

**Response**:
```
# TYPE agq_jobs_total counter
agq_jobs_total{status="completed"} 1523
agq_jobs_total{status="failed"} 42

# TYPE agq_workers_active gauge
agq_workers_active 8

# TYPE agq_queue_length gauge
agq_queue_length{queue="ready"} 42
```

---

## Implementation Status

| Command | Phase | Status | Priority |
|---------|-------|--------|----------|
| `PLAN.SUBMIT` | 2 | Planned | High |
| `PLAN.GET` | 2 | Planned | High |
| `ACTION.SUBMIT` | 2 | Planned | High |
| `JOB.STATUS` | 2 | Planned | High |
| `JOB.UPDATE` | 2 | Planned | High |
| `JOB.LIST` | 2 | Planned | Medium |
| `ACTION.STATUS` | 2 | Planned | Medium |
| `WORKER.REGISTER` | 2 | Planned | High |
| `WORKER.HEARTBEAT` | 2 | Planned | High |
| `WORKER.UNREGISTER` | 2 | Planned | Medium |
| `QUEUE.STATS` | 2 | Planned | Medium |
| `PLAN.LIST` | 3 | Future | Low |
| `PLAN.DELETE` | 3 | Future | Low |
| `JOB.RETRY` | 3 | Future | Medium |
| `QUEUE.PEEK` | 3 | Future | Low |
| `JOBS.SEARCH` | 3 | Future | Low |
| `METRICS` | 3 | Future | Medium |

---

## Related Documentation

- [RESP Protocol](./resp-protocol.md) - Base protocol specification
- [Worker Registration](./worker-registration.md) - Worker registration flow
- [Job Schema](../architecture/job-schema.md) - Plan/Job structure
- [Execution Layers](../architecture/execution-layers.md) - Nomenclature

---

**Maintained by:** AGX Core Team
**Review cycle:** Per phase delivery
**Questions?** See [resp-protocol.md](./resp-protocol.md) for base protocol
