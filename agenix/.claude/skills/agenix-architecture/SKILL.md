---
name: agenix-architecture
description: Apply AGEniX architectural patterns, execution layers (Task/Plan/Job/Action/Workflow), component boundaries, and nomenclature when working on agx, agq, agw, or AU development
allowed-tools: Read, Grep, Glob
---

# AGEniX Architecture Skill

This skill ensures consistent application of AGEniX architectural patterns and nomenclature across all component repositories.

## Canonical Documentation

The authoritative architecture documentation is located in the central agenix repository:

- **Execution Layers**: `/Users/lewis/work/agenix-sh/agenix/docs/architecture/execution-layers.md`
- **System Overview**: `/Users/lewis/work/agenix-sh/agenix/docs/architecture/system-overview.md`
- **Component Boundaries**: `/Users/lewis/work/agenix-sh/agenix/docs/architecture/agx-agq-agw.md`
- **Job Schema**: `/Users/lewis/work/agenix-sh/agenix/docs/architecture/job-schema.md`

## Five Execution Layers

Always use correct terminology from execution-layers.md:

1. **Task** - Atomic execution unit (single command with args)
2. **Plan** - Ordered list of Tasks (compiled by AGX)
3. **Job** - Execution of a Plan by one AGW worker
4. **Action** - Embarrassingly-parallel execution of many Jobs
5. **Workflow** - Chained Actions with conditional logic (future)

## Key Principles

### Component Responsibilities

**AGX (Planner)**:
- Interprets user intent
- Compiles Plans (Task sequences)
- Submits Jobs to AGQ
- No execution (delegates to AGW via AGQ)

**AGQ (Queue Manager)**:
- Stores Plans and Jobs
- Matches Jobs to Workers
- Tracks Job status
- Uses RESP protocol
- Embedded redb database

**AGW (Worker)**:
- Pulls Jobs from AGQ
- Executes Tasks sequentially
- Reports results back to AGQ
- Zero-trust execution model
- Never generates Plans (security boundary)

**AU (Agentic Unit)**:
- Specialized tools orchestrated by AGX
- stdin/stdout contract
- `--describe` model card
- Binary input, JSON output

### Nomenclature Rules

**ALWAYS use:**
- ✅ "Task" (not "step", "instruction", "command")
- ✅ "Plan" (not "workflow", "job", "script")
- ✅ "Job" (not "task", "work item")
- ✅ "Action" (not "batch", "parallel job")
- ✅ "task_number" (not "step_number", "index")
- ✅ "tasks" field (not "steps", "commands")

**Component Names:**
- ✅ AGX, agx (not "planner", "orchestrator")
- ✅ AGQ, agq (not "queue", "broker", "scheduler")
- ✅ AGW, agw (not "worker", "executor", "agent")
- ✅ AU (not "tool", "plugin", "extension")

## When to Reference This Skill

Activate this skill when:
- Designing or reviewing component interactions
- Writing code that touches Plan/Job/Task structures
- Creating documentation about the system
- Discussing architectural decisions
- Reviewing PRs that cross component boundaries
- Explaining the system to new contributors

## Usage Examples

### Checking Terminology

Before committing code or docs, verify:
```bash
# Check for incorrect terminology
grep -r "step" src/ docs/  # Should use "task"
grep -r "workflow" src/    # Should use "plan" or "action"
```

### Validating Component Boundaries

When making changes that affect multiple components:
1. Read execution-layers.md to confirm layer responsibilities
2. Check that AGX doesn't execute (only plans)
3. Ensure AGW doesn't generate plans (only executes)
4. Verify RESP protocol usage between AGX ↔ AGQ ↔ AGW

### Reviewing Job Schema

When working with Job structures:
```rust
// ✅ Correct nomenclature
struct Job {
    job_id: String,
    plan_id: String,
    tasks: Vec<Task>,  // Not "steps"
}

struct Task {
    task_number: usize,  // Not "step_number"
    command: String,
    args: Vec<String>,
}
```

## Progressive Disclosure

For detailed architectural context, reference the canonical documentation files directly using the Read tool.

This skill provides quick reference for common patterns - consult the full docs for comprehensive details.
