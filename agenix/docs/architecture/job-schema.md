# Job Envelope Schema

**Version:** 0.2
**Status:** Canonical Specification
**Updated:** 2025-11-18 (JSON schema available in specs/job.schema.json)
**JSON Schema:** `../../specs/job.schema.json`

This document defines the Job payload structure used across the AGEniX ecosystem. A Job contains the complete Plan as a single unit, ensuring all Tasks execute on one AGW worker with local data access.

**Note:** This describes the **Job envelope** (Execution Layer 3). For Plan templates (Layer 2), see the distinction in `specs/README.md`.

---

## Nomenclature Alignment

This schema has been updated to align with the [canonical execution layers](./execution-layers.md):
- ~~"steps"~~ → **"tasks"**
- ~~"step_number"~~ → **"task_number"**
- ~~"input_from_step"~~ → **"input_from_task"**

---

## Job Envelope Structure

```json
{
  "job_id": "uuid-1234",
  "plan_id": "uuid-5678",
  "plan_description": "Summarize logs and count errors",
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

---

## Field Descriptions

### Top-Level Fields

- **`job_id`** (string, required): Unique identifier for this execution instance. A new `job_id` is generated for every Job submission.

- **`plan_id`** (string, required): Stable identifier for the logical Plan. The same `plan_id` is reused when executing the same Plan with different inputs across multiple Jobs.

- **`plan_description`** (string, optional): Human-readable description of the Plan's intent and reasoning. Useful for operations, search, and debugging.

- **`tasks`** (array, required): Ordered, non-empty list of Tasks to execute sequentially.

### Task Fields

Each Task in the `tasks` array has:

- **`task_number`** (u32, required): 1-based, contiguous task identifier. Must start at 1 and increment by 1 for each subsequent task.

- **`command`** (string, required): Tool or command identifier (e.g., `sort`, `uniq`, `agx-ocr`, `jq`).

- **`args`** (array of strings, optional): Arguments passed to the command. Defaults to empty array if not specified.

- **`timeout_secs`** (u32, optional): Per-task timeout in seconds. If the task exceeds this duration, it is terminated. Default is system-defined (typically 300s).

- **`input_from_task`** (u32, optional): Reference to a previous task number. The stdout from the referenced task becomes the stdin for this task. Must reference a task that appears earlier in the sequence (no forward or self-references allowed).

---

## Validation Rules

### Client-Side Validation (AGX)

Before submitting a Job, AGX must validate:

1. **Non-empty tasks**: The `tasks` array must contain at least one task.

2. **Contiguous task numbering**: Task numbers must start at 1 and increment by 1 (no gaps, no duplicates).

3. **Task limit**: The number of tasks must not exceed 100 (configurable per deployment).

4. **Valid input references**: If `input_from_task` is specified, it must reference a task number that appears earlier in the `tasks` array. Self-references and forward references are invalid.

5. **Command validation**: The `command` field must not be empty and should reference a registered tool or AU.

### Server-Side Validation (AGQ)

AGQ should validate:

1. **Schema compliance**: JSON structure matches this specification.

2. **ID uniqueness**: `job_id` is unique across all active and recent jobs.

3. **Plan ID consistency**: `plan_id` references a known Plan (if Plan registry is implemented).

---

## Execution Expectations (AGW)

When AGW executes a Job:

1. **Sequential execution**: Tasks are executed in order by `task_number`.

2. **Data piping**: If a task specifies `input_from_task`, the stdout from the referenced task is piped to the stdin of the current task.

3. **Fail-fast behavior**: If any task fails (non-zero exit code, timeout, or crash), execution halts immediately. Partial results (stdout/stderr from completed tasks) are returned.

4. **Isolation**: All tasks in a Job execute on a single AGW worker to ensure local data access and avoid network overhead.

5. **Timeout enforcement**: Each task respects its `timeout_secs` value (or system default). On timeout, the task process is terminated (SIGTERM, then SIGKILL after grace period).

---

## RESP Protocol Submission

### AGX → AGQ Submission

AGX serializes the Job envelope to JSON and submits via the RESP protocol:

```
PLAN.SUBMIT <json_payload>
```

Or (if AGQ uses different verb):

```
JOB.SUBMIT <json_payload>
```

### AGQ Response

On success, AGQ returns:

```
+OK job_id=uuid-1234
```

On error:

```
-ERR Invalid task numbering: gap between task 2 and 4
```

AGX stores the `job_id` locally alongside the plan buffer for tracking and retrieval.

---

## Example: Multi-Task Job

```json
{
  "job_id": "job-abc123",
  "plan_id": "plan-log-analysis",
  "plan_description": "Extract errors from logs, count by severity",
  "tasks": [
    {
      "task_number": 1,
      "command": "grep",
      "args": ["-i", "error"],
      "timeout_secs": 60
    },
    {
      "task_number": 2,
      "command": "sort",
      "args": [],
      "input_from_task": 1,
      "timeout_secs": 30
    },
    {
      "task_number": 3,
      "command": "uniq",
      "args": ["-c"],
      "input_from_task": 2,
      "timeout_secs": 30
    }
  ]
}
```

**Execution flow:**
1. Task 1: `grep -i error` → outputs error lines
2. Task 2: `sort` (reads Task 1 stdout) → outputs sorted error lines
3. Task 3: `uniq -c` (reads Task 2 stdout) → outputs counted unique errors

---

## Migration Notes (from v0.1)

**Breaking changes from previous schema:**

- `steps` → `tasks`
- `step_number` → `task_number`
- `input_from_step` → `input_from_task`

**Upgrade path:**
- Update all client code (AGX) to use new field names
- Update AGQ parser to accept new field names
- Update AGW executor to use new field names
- Maintain backward compatibility parser for 1 release cycle (optional)

---

## Related Documentation

- [Execution Layers](./execution-layers.md) - Canonical nomenclature
- [System Overview](./system-overview.md) - High-level architecture
- [RESP Protocol](../api/resp-protocol.md) - Communication protocol (future)

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly or on breaking changes
**Questions?** Consult [execution-layers.md](./execution-layers.md) for nomenclature
