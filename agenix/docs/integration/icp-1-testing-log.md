# ICP-1 Integration Testing Log

**Date:** 2025-11-18
**Session:** Initial integration testing
**Goal:** Validate end-to-end job execution flow

---

## Session Summary

Attempted first integration test of AGX → AGQ → AGW flow to validate ICP-1 requirements. Discovered critical schema mismatch that blocks full integration testing.

## Components Tested

### AGQ (Queue) - ✅ Partially Working

**Status:** AGQ-007, AGQ-008, AGQ-009 all merged and functional

**Test:** PLAN.SUBMIT endpoint
```bash
$ python3 test_plan_submit.py
AUTH response: +OK
PLAN.SUBMIT response: plan_9410cee8edc741729aea2b36f2d1e410
```

**Result:** ✅ AGQ successfully:
- Accepted authentication (session key)
- Validated plan JSON against schema
- Stored plan in database
- Returned plan_id

**Logs:**
```
[INFO] AGQ server started successfully on 127.0.0.1:6379
[INFO] Client authenticated successfully
[INFO] Plan plan_9410cee8edc741729aea2b36f2d1e410 stored successfully
```

### AGX (Planner) - ⚠️ Not Tested

**Reason:** Model backend not configured for testing
**Note:** PLAN commands work but require LLM backend setup

### AGW (Worker) - ⏸️ Not Started

**Reason:** Blocked by schema mismatch discovery
**Next:** Need schema alignment before testing worker execution

---

## Critical Discovery: Schema Mismatch

### Problem

AGQ, documentation, and test fixtures use **incompatible schemas**.

### Evidence

**AGQ Implementation (agq/src/server.rs:220):**
```json
{
  "version": "0.1",
  "tasks": [
    {
      "id": "task-1",
      "tool": "echo",
      "args": ["hello", "world"]
    }
  ]
}
```

**Documentation (docs/architecture/job-schema.md):**
```json
{
  "job_id": "uuid-1234",
  "plan_id": "uuid-5678",
  "tasks": [
    {
      "task_number": 1,
      "command": "sort",
      "args": ["-r"],
      "timeout_secs": 30,
      "input_from_task": null
    }
  ]
}
```

### Key Differences

| Aspect | AGQ (Code) | Documentation | Impact |
|--------|-----------|---------------|---------|
| Task ID | `id` (string) | `task_number` (integer) | ❌ Incompatible |
| Executable | `tool` | `command` | ❌ Incompatible |
| Job wrapper | None | `job_id`, `plan_id` | ❌ Missing envelope |
| Timeout | None | `timeout_secs` | ⚠️ Feature missing |
| Task piping | None | `input_from_task` | ⚠️ Feature missing |

### Impact Assessment

**Blocks:**
- ❌ End-to-end integration testing
- ❌ AGX job submission (unknown which schema to use)
- ❌ AGW job parsing (unknown which schema to expect)
- ❌ ICP-1 test fixture execution
- ❌ DGX deployment validation

**Affects:**
- AGX: Job envelope generation
- AGQ: Schema validation, plan storage
- AGW: Job parsing, execution engine
- Tests: All ICP-1 test fixtures
- Docs: job-schema.md, plan.schema.json

---

## Root Cause Analysis

### How This Happened

1. **AGQ-009** (PLAN.SUBMIT) implemented with simplified schema:
   - Used `tasks[].tool` following existing `specs/plan.schema.json`
   - Did not include job envelope (`job_id`, `plan_id`)
   - Omitted timeout and piping features

2. **job-schema.md** documented richer schema:
   - Used `tasks[].command` with job envelope
   - Included `task_number` for ordering
   - Added `timeout_secs` and `input_from_task` for piping
   - Based on Phase 1 requirements

3. **No cross-component schema tests:**
   - Each component developed in isolation
   - No integration test caught the mismatch
   - Documentation not used as source of truth

### Lesson Learned

**Integration testing assumption was correct:** The longer components work in isolation, the more likely critical mismatches occur. This session proved that assumption and caught the issue before it became deeply embedded in multiple codebases.

---

## Recommended Fix

### Option 1: Align to job-schema.md (RECOMMENDED)

**Pros:**
- Matches documented Phase 1 spec
- Supports required features (timeout, piping)
- Consistent nomenclature (task_number)
- Job envelope enables Action semantics (Phase 2)

**Cons:**
- Requires AGQ schema update
- May require AGX/AGW updates

**Work Required:**
- Update AGQ `PLAN_SCHEMA` constant
- Update AGQ plan storage/retrieval
- Verify AGX job envelope generation
- Verify AGW job parsing
- Update all ICP-1 test fixtures

### Option 2: Update Documentation

**Pros:**
- Minimal code changes

**Cons:**
- ❌ Loses timeout feature
- ❌ Loses task piping (`input_from_task`)
- ❌ Breaks intended Phase 1 design
- ❌ Not recommended

---

## Next Steps

### Immediate (Before ICP-1)

1. **Fix schema mismatch** (Issue #11)
   - Align AGQ to job-schema.md
   - Update test fixtures
   - Verify AGX/AGW compatibility

2. **Resume integration testing**
   - Submit aligned job via AGX or direct RESP
   - Start AGW worker
   - Verify job fetching and execution
   - Validate result posting

3. **Run ICP-1 test suite**
   - Execute automated tests
   - Fix any remaining issues
   - Document results

### Short-term (ICP-1 Completion)

1. **Add cross-component schema tests**
   - Validate AGX job envelope against schema
   - Validate AGQ storage against schema
   - Validate AGW parsing against schema
   - Fail CI if schema drift detected

2. **Make job-schema.md canonical**
   - Update `specs/plan.schema.json` to match
   - Link from AGQ/AGX/AGW documentation
   - Add schema version to job envelope

3. **Deploy to DGX**
   - Build all components with aligned schema
   - Run smoke test
   - Validate end-to-end on production hardware

---

## Test Artifacts

### Working Test Script

**File:** `/tmp/test_plan_submit.py`

Successfully submits plan to AGQ using RESP protocol:
```python
plan = {
    "version": "0.1",
    "tasks": [
        {
            "id": "task-1",
            "tool": "echo",
            "args": ["hello", "world"]
        }
    ]
}
```

### AGQ Configuration

**Session Key:** `755c8783080e64c2da06e4011b263baaec8bc8d6eec79c91168efb0761509610`
**Port:** 6379
**Database:** `/Users/lewis/.agq/data.redb`

### Test Results

- ✅ AGQ authentication works
- ✅ PLAN.SUBMIT validates and stores plan
- ✅ AGQ returns plan_id
- ✅ AGQ logs show successful storage
- ❌ Schema incompatible with documentation
- ⏸️ AGW execution blocked by schema mismatch

---

## References

- **Issue Created:** #11 - CRITICAL: Schema mismatch blocking ICP-1
- **ICP-1 Spec:** `docs/integration/icp-1-minimal-execution.md`
- **Job Schema:** `docs/architecture/job-schema.md`
- **AGQ Schema:** `agq/src/server.rs` (PLAN_SCHEMA constant)
- **Test Infrastructure:** `tests/integration/icp-1/`

---

## Conclusion

**Session Outcome:** Successfully validated AGQ functionality but discovered critical blocker.

**Key Achievement:** Caught schema mismatch early through integration testing, preventing deeper integration debt.

**Status:** ICP-1 testing paused pending schema alignment (Issue #11).

**Next Session:** Resume testing after schema fix is merged.

---

**Logged by:** Integration Testing Session
**Follow-up:** Schema alignment work (AGQ/AGX/AGW)
**Target:** Resume ICP-1 testing within 1-2 days
