# Distributed Execution Model

This document describes how AGEnix executes plans across `agx`, `agq` and `agw`.

## Local Mode

```text
stdin → agx [intent] → plan → execute tools locally → stdout
```

- Planning and execution both happen in the `agx` process.
- Suitable for quick, single-machine workflows and experimentation.

## Distributed Mode

```text
stdin → agx [intent] → signed plan + payload → agq → agw → stdout
```

1. `agx`:
   - Generates a JSON plan.
   - Signs the plan.
   - Submits the plan + input payload to `agq`.

2. `agq`:
   - Stores the job.
   - Assigns it to a registered `agw` worker.

3. `agw`:
   - Verifies the plan signature.
   - Executes each step as a separate tool process.
   - Collects outputs/logs, reports completion.

This model keeps LLM access and plan generation in `agx`, while `agw` remains a minimal, auditable executor.
