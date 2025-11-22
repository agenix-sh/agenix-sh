# Worker Registration Protocol

**Version:** 1.0
**Status:** Specification (Phase 2 - Planned)
**Last Updated:** 2025-11-17

This document specifies the worker registration and lifecycle management protocol for AGW workers connecting to AGQ.

---

## Table of Contents

1. [Overview](#overview)
2. [Registration Flow](#registration-flow)
3. [Heartbeat Protocol](#heartbeat-protocol)
4. [Job Lifecycle](#job-lifecycle)
5. [Error Handling](#error-handling)
6. [Worker States](#worker-states)
7. [Security Considerations](#security-considerations)

---

## 1. Overview

### Purpose

The worker registration protocol enables:
- AGW workers to announce their presence to AGQ
- Capability-based job routing
- Worker health monitoring via heartbeats
- Graceful worker shutdown and failover

### Actors

- **AGW (Worker)**: Stateless execution engine that pulls and executes jobs
- **AGQ (Queue Manager)**: Centralized queue and worker coordinator

---

## 2. Registration Flow

### 2.1 Initial Connection

```
┌─────┐                                  ┌─────┐
│ AGW │                                  │ AGQ │
└──┬──┘                                  └──┬──┘
   │                                        │
   │  1. TCP Connect (127.0.0.1:6380)      │
   ├───────────────────────────────────────>│
   │                                        │
   │  2. AUTH <session_key>                 │
   ├───────────────────────────────────────>│
   │  +OK                                   │
   │<───────────────────────────────────────┤
   │                                        │
   │  3. WORKER.REGISTER <json>             │
   ├───────────────────────────────────────>│
   │  +OK worker_id=... heartbeat_interval=30│
   │<───────────────────────────────────────┤
   │                                        │
   │  Now ready to pull jobs                │
   │                                        │
```

### 2.2 Registration Command

**Command**: `WORKER.REGISTER <worker_json>`

**Worker JSON Structure**:
```json
{
  "worker_id": "worker-macbook-001",
  "hostname": "macbook-pro.local",
  "platform": "darwin-arm64",
  "agw_version": "0.1.0",
  "capabilities": {
    "tools": ["sort", "uniq", "grep", "cut", "jq"],
    "agentic_units": ["agx-ocr"]
  },
  "max_concurrent_jobs": 4,
  "tags": {
    "environment": "local",
    "tier": "development"
  }
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `worker_id` | string | Yes | Unique worker identifier (e.g., `worker-<hostname>-<pid>`) |
| `hostname` | string | Yes | Machine hostname for debugging |
| `platform` | string | No | OS/arch (e.g., `darwin-arm64`, `linux-x86_64`) |
| `agw_version` | string | Yes | AGW binary version (semantic versioning) |
| `capabilities.tools` | array | Yes | List of Unix tools available (from `$PATH`) |
| `capabilities.agentic_units` | array | No | List of installed AUs (default: empty) |
| `max_concurrent_jobs` | integer | No | Maximum parallel jobs (default: 1) |
| `tags` | object | No | Arbitrary key-value metadata |

**Response**:
- Success: `+OK worker_id=worker-macbook-001 heartbeat_interval=30`
- Error: `-ERR Worker ID already registered`
- Error: `-ERR Invalid capabilities format`

**Worker ID Requirements**:
- Must be unique across all active workers
- Recommended format: `worker-<hostname>-<pid>` or UUID
- Alphanumeric + hyphens only
- Max 64 characters

### 2.3 AGQ Storage

On successful registration, AGQ stores:

**Worker Metadata**:
```
Key: worker:<worker_id>:metadata
Value: JSON (worker registration data)
TTL: None (persists until unregister)
```

**Worker Alive Timestamp**:
```
Key: worker:<worker_id>:alive
Value: Unix timestamp (seconds)
TTL: 90 seconds (3x heartbeat interval)
```

**Capabilities Index**:
```
Key: capability:<tool_name>:workers
Value: Set of worker IDs
```

Example: `capability:agx-ocr:workers = {worker-001, worker-003}`

---

## 3. Heartbeat Protocol

### 3.1 Purpose

Heartbeats serve to:
- Detect worker failures (no heartbeat = dead worker)
- Monitor worker health and load
- Provide real-time statistics

### 3.2 Heartbeat Interval

**Default**: 30 seconds
**Timeout**: 90 seconds (3x interval)

If no heartbeat received within timeout, worker is marked **dead** and any active jobs are re-queued.

### 3.3 Heartbeat Command

**Command**: `WORKER.HEARTBEAT <worker_id> [stats_json]`

**Stats JSON Structure** (optional):
```json
{
  "active_jobs": 2,
  "completed_jobs_total": 150,
  "failed_jobs_total": 3,
  "uptime_seconds": 3600,
  "cpu_usage_percent": 45.2,
  "memory_mb": 512,
  "disk_available_gb": 120
}
```

**Response**:
- Success: `+OK`
- Error: `-ERR Worker not registered: <worker_id>`

**AGW Implementation**:
```rust
// Spawn background task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;

        let stats = collect_worker_stats().await;
        match client.heartbeat(worker_id, Some(stats)).await {
            Ok(_) => debug!("Heartbeat sent"),
            Err(e) => error!("Heartbeat failed: {}", e),
        }
    }
});
```

### 3.4 Heartbeat Storage

**Worker Alive Timestamp**:
```
Key: worker:<worker_id>:alive
Value: Unix timestamp
TTL: 90 seconds (auto-expires)
```

Each heartbeat refreshes the TTL. If TTL expires, worker is considered dead.

**Worker Stats** (optional):
```
Key: worker:<worker_id>:stats
Value: JSON stats
TTL: 120 seconds
```

---

## 4. Job Lifecycle

### 4.1 Job Pulling

Workers pull jobs using blocking pop:

**Command**: `BRPOP queue:ready <timeout>`

**Behavior**:
- Blocks until job available or timeout
- Returns job JSON with `job_id`, `plan`, `inputs`
- Worker immediately claims job (job removed from queue)

**Example**:
```rust
loop {
    match client.brpop("queue:ready", 5).await {
        Ok(Some(job_json)) => {
            let job: Job = serde_json::from_str(&job_json)?;
            execute_job(job).await?;
        }
        Ok(None) => {
            // Timeout, loop and try again
        }
        Err(e) => {
            error!("BRPOP failed: {}", e);
        }
    }
}
```

### 4.2 Job Execution Updates

**Start Notification**:
```bash
JOB.UPDATE job-abc123 '{"status":"running","worker_id":"worker-001","started_at":"2025-11-17T10:00:00Z"}'
```

**Progress Updates** (optional):
```bash
JOB.UPDATE job-abc123 '{"current_task":2,"progress_percent":40}'
```

**Completion Notification**:
```bash
JOB.UPDATE job-abc123 '{"status":"completed","completed_at":"2025-11-17T10:00:15Z","task_results":[...]}'
```

**Failure Notification**:
```bash
JOB.UPDATE job-abc123 '{"status":"failed","failed_at":"2025-11-17T10:00:10Z","error":"Task 2 timed out","task_results":[...]}'
```

### 4.3 Job Ownership

**Claim Tracking**:
```
Key: job:<job_id>:worker
Value: <worker_id>
TTL: Job completion timeout (default: 1 hour)
```

Once a worker pulls a job via `BRPOP`, AGQ records the claim. Only that worker can submit updates.

---

## 5. Error Handling

### 5.1 Connection Loss

**Worker Behavior**:
- Detect connection loss (TCP timeout, write error)
- Attempt reconnection with exponential backoff
- Re-authenticate and re-register on reconnect
- Resume job pulling

**AGQ Behavior**:
- Mark worker offline (TTL expiry on `worker:<id>:alive`)
- Re-queue any jobs claimed by dead worker (after timeout)

**Reconnection Backoff**:
```
Attempt 1: Wait 1 second
Attempt 2: Wait 2 seconds
Attempt 3: Wait 4 seconds
...
Max wait: 60 seconds
```

### 5.2 Job Timeout

If worker fails to complete job within timeout (default: 1 hour):
- AGQ marks job as `failed`
- Job is re-queued for retry (if retries remain)
- Worker is flagged for investigation (potential hang)

### 5.3 Worker Crash

**Detection**:
- No heartbeat for 90 seconds
- `worker:<id>:alive` key expires

**Recovery**:
- AGQ re-queues any jobs claimed by crashed worker
- Worker metadata remains for debugging (manual cleanup)

**AGW Restart**:
- New `worker_id` (e.g., new PID)
- Re-register as new worker
- Old worker remains in "dead" state

---

## 6. Worker States

### State Diagram

```
     ┌──────────────┐
     │              │
     │  UNREGISTERED│
     │              │
     └──────┬───────┘
            │
            │ WORKER.REGISTER
            │
            ▼
     ┌──────────────┐      Heartbeat timeout
     │              │      (90 seconds)
     │    ACTIVE    ├─────────────────────────┐
     │              │                         │
     └──────┬───────┘                         │
            │                                 │
            │ WORKER.UNREGISTER               │
            │ (graceful shutdown)             │
            │                                 ▼
            │                          ┌──────────────┐
            │                          │              │
            │                          │     DEAD     │
            │                          │              │
            │                          └──────────────┘
            │
            ▼
     ┌──────────────┐
     │              │
     │ UNREGISTERED │
     │              │
     └──────────────┘
```

### State Descriptions

| State | Description | Can Pull Jobs? | Heartbeat Required? |
|-------|-------------|----------------|---------------------|
| `UNREGISTERED` | Worker not yet registered or cleanly shut down | No | No |
| `ACTIVE` | Worker registered, heartbeating, available | Yes | Yes (every 30s) |
| `DEAD` | Worker failed, crashed, or timed out | No | No |

---

## 7. Security Considerations

### 7.1 Session Key Isolation

Each worker has its own session key:
- Generate unique 32-byte random key per worker
- Distribute via secure channel (config file, environment variable)
- Never log session keys

**AGQ Configuration**:
```toml
[workers]
"worker-001" = "session_key_64_hex_chars_for_worker_001..."
"worker-002" = "session_key_64_hex_chars_for_worker_002..."
```

### 7.2 Worker ID Validation

Prevent injection attacks:
```rust
fn validate_worker_id(id: &str) -> Result<()> {
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("Invalid worker ID characters");
    }
    if id.len() > 64 {
        bail!("Worker ID too long");
    }
    Ok(())
}
```

### 7.3 Capability Verification

**Phase 2**: Trust worker-reported capabilities
**Phase 3**: Verify capabilities via challenge-response

```bash
# AGQ challenges worker
> WORKER.VERIFY agx-ocr '{"command":"agx-ocr","args":["--version"]}'

# Worker executes and responds
< +OK output="agx-ocr 0.1.0"
```

### 7.4 Job Ownership Enforcement

Workers can only update jobs they claimed:
```rust
fn validate_job_update(job_id: &str, worker_id: &str) -> Result<()> {
    let claimed_by = db.get(format!("job:{}:worker", job_id))?;
    if claimed_by != worker_id {
        bail!("Worker {} cannot update job claimed by {}", worker_id, claimed_by);
    }
    Ok(())
}
```

---

## 8. Graceful Shutdown

### 8.1 Worker Shutdown Sequence

```rust
async fn graceful_shutdown(worker_id: &str, client: &mut RespClient) -> Result<()> {
    // 1. Stop pulling new jobs
    stop_job_puller().await;

    // 2. Wait for active jobs to complete
    wait_for_active_jobs(Duration::from_secs(300)).await?;

    // 3. Unregister from AGQ
    client.unregister(worker_id).await?;

    // 4. Close connection
    client.close().await?;

    Ok(())
}
```

### 8.2 Forced Shutdown

If graceful shutdown times out:
- Log warning with active job IDs
- Unregister anyway
- Exit (AGQ will re-queue jobs)

---

## 9. Monitoring and Observability

### 9.1 Worker Metrics

Workers should expose:
- Total jobs completed
- Total jobs failed
- Average job duration
- Current CPU/memory usage
- Uptime

**Via Heartbeat Stats**:
```json
{
  "completed_jobs_total": 150,
  "failed_jobs_total": 3,
  "avg_job_duration_ms": 2500,
  "cpu_usage_percent": 45.2,
  "uptime_seconds": 3600
}
```

### 9.2 AGQ Queries

**List Active Workers**:
```bash
> KEYS worker:*:alive
*3
worker:worker-001:alive
worker:worker-002:alive
worker:worker-003:alive
```

**Get Worker Stats**:
```bash
> GET worker:worker-001:stats
{"active_jobs":2,"completed_jobs_total":150,...}
```

---

## 10. Future Enhancements

### Phase 3

**Dynamic Capability Discovery**:
- Workers report `--describe` outputs for all AUs
- AGQ builds searchable capability index
- Semantic matching for job routing

**Worker Pools**:
- Group workers by tags (e.g., `gpu: true`)
- Route jobs to specific pools

**Prioritization**:
- Workers subscribe to multiple queues
- High-priority queue checked first

**Autoscaling**:
- AGQ monitors queue depth
- Signals to spawn new workers (via external orchestrator)

---

## Related Documentation

- [RESP Protocol](./resp-protocol.md) - Base protocol
- [AGQ Endpoints](./agq-endpoints.md) - Full command reference
- [Job Schema](../architecture/job-schema.md) - Job structure
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Security model

---

**Maintained by:** AGX Core Team
**Review cycle:** Per phase delivery
**Questions?** See [agq-endpoints.md](./agq-endpoints.md) for command details
