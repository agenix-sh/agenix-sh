# AGEniX System Overview

**Version:** 1.0
**Status:** Canonical High-Level Architecture
**Last Updated:** 2025-11-17

This document provides a high-level overview of the AGEniX ecosystem architecture. For implementation-specific details, refer to individual component repositories.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Core Philosophy](#core-philosophy)
3. [System Components](#system-components)
4. [Execution Model](#execution-model)
5. [Communication Architecture](#communication-architecture)
6. [Security Model](#security-model)
7. [Data Flow](#data-flow)
8. [Future Extensions](#future-extensions)

---

## 1. Introduction

The **AGEniX ecosystem** is a minimal, Unix-philosophy-aligned system for agentic plan generation and deterministic execution on local hardware.

### Key Objectives

- **Local-first execution**: Run entirely on your hardware with zero cloud dependencies
- **LLM-assisted planning**: Use AI to generate execution plans, not to execute them
- **Deterministic execution**: Plans are JSON specifications executed consistently
- **Composable tools**: Single-purpose Agentic Units (AUs) that combine via stdin/stdout
- **Zero-trust execution**: Isolated, sandboxed tool execution with minimal privileges

### Target Use Cases

- **Batch data processing**: Transform datasets using composable AU tools
- **Document analysis**: OCR, extraction, summarization pipelines
- **Research workflows**: Orchestrate analysis tools across large datasets
- **Local AI workflows**: Chain together local AI models without cloud APIs

---

## 2. Core Philosophy

### 2.1 Unix Philosophy Alignment

AGEniX adheres to classic Unix principles:

- **Small, focused tools**: Each AU does one thing well
- **Composability**: Tools chain via stdin/stdout piping
- **Text as universal interface**: JSON for structured data, plain text for everything else
- **Separation of concerns**: Planning ≠ Scheduling ≠ Execution

### 2.2 Zero External Dependencies

- **Pure Rust** binaries (cross-platform: macOS, Linux)
- **Embedded database** (redb) - no separate database servers
- **No cloud services** - runs entirely local
- **Self-contained deployment** - single binary per component

### 2.3 Clear Separation of Responsibilities

```
AGX   → Planning (generates JSON plans using LLMs)
AGQ   → Scheduling (queues jobs, tracks state, manages workers)
AGW   → Execution (runs deterministic plan steps)
AUs   → Tools (single-purpose specialized capabilities)
```

---

## 3. System Components

### 3.1 AGX - Planner/Orchestrator

**Purpose:** Convert natural-language intents into deterministic JSON plans

**Key Features:**
- LLM-assisted plan generation (dual-model: Echo + Delta)
- Tool registry awareness (via `--describe` contracts)
- Local or distributed execution modes
- Job submission to AGQ
- Operations monitoring (jobs, workers, queue stats)

**Modes:**
- `PLAN` mode: Create, preview, and submit plans
- `OPS` mode: Monitor jobs, workers, queue status
- Future: Interactive REPL

**See:** `agx/docs/ARCHITECTURE.md` for implementation details

---

### 3.2 AGQ - Queue Manager/Scheduler

**Purpose:** Manage job queuing, scheduling, and worker coordination

**Key Features:**
- Embedded redb database (single-file ACID storage)
- RESP protocol server (Redis-compatible API)
- Job lifecycle tracking (pending → running → completed/failed)
- Worker registration and heartbeat monitoring
- Plan storage and reuse
- Action fan-out (one Plan → many Jobs)

**Security:**
- Session-key authentication for all commands
- Rate limiting on authentication attempts
- Input validation and sanitization
- No plan contents in logs (security-critical)

**See:** `agq/ARCHITECTURE.md` for implementation details

---

### 3.3 AGW - Worker

**Purpose:** Execute deterministic plan steps in isolation

**Key Features:**
- RESP client (connects to AGQ)
- Stateless execution (no plan state maintained)
- Sequential task execution
- stdin/stdout piping between tasks
- Fail-fast behavior (halt on first error)
- Capability-based registration

**Security:**
- No LLM access (deterministic execution only)
- Session-key authentication
- Tool execution sandboxing
- Timeout enforcement per task
- Resource limits

**See:** `agw/docs/ARCHITECTURE.md` for implementation details

---

### 3.4 Agentic Units (AUs)

**Purpose:** Composable, single-purpose tools that implement specific capabilities

**Examples:**
- `agx-ocr`: Document OCR using DeepSeek models
- Future: `agx-summarize`, `agx-extract`, `agx-embed`, etc.

**Contract Requirements:**
- `--describe` flag returns JSON model card (capabilities, inputs, outputs)
- stdin → stdout operation
- Exit codes for success/failure
- JSON-structured output (when applicable)

**See:** `docs/au-specs/agentic-unit-spec.md` for AU contract specification

---

## 4. Execution Model

AGEniX uses a **five-layer execution model** for clear nomenclature and separation of concerns.

### The Five Execution Layers

| Layer | Name | Description | Created By | Executed By | Managed By |
|-------|------|-------------|------------|-------------|------------|
| 1 | **Task** | Atomic tool/AU call (stdin → stdout) | AGX | AGW | - |
| 2 | **Plan** | Ordered list of Tasks (reusable definition) | AGX | AGW | AGQ |
| 3 | **Job** | Runtime execution of a Plan (with specific input) | AGQ | AGW | AGQ |
| 4 | **Action** | Many Jobs (same Plan, different inputs) | AGX | AGW (many) | AGQ |
| 5 | **Workflow** | Multi-Action orchestration (future) | AGX | AGQ/AGW | AGQ |

**See:** [execution-layers.md](./execution-layers.md) for detailed specification

---

## 5. Communication Architecture

### 5.1 Protocol: RESP (Redis Protocol)

All AGEniX components communicate using **RESP** (REdis Serialization Protocol):

- **Why RESP?** Simple, text-based, well-tested, Redis-compatible
- **Transport:** TCP sockets (future: Unix domain sockets, mTLS)
- **Authentication:** Session-key based (32+ byte cryptographic random keys)

### 5.2 Key API Commands

**AGX → AGQ:**
```
PLAN.SUBMIT <json_plan>      # Store a Plan
ACTION.SUBMIT <json_action>  # Create Jobs from Plan + inputs
JOB.STATUS <job_id>          # Query Job status
JOB.LIST <action_id>         # List Jobs for an Action
```

**AGW → AGQ:**
```
AUTH <session_key>           # Authenticate worker
WORKER.REGISTER <capabilities> # Register worker capabilities
BRPOP queue:ready            # Pull next Job from queue
JOB.UPDATE <job_id> <status> # Report Job progress
```

**See:** `docs/tools/tool-contracts.md` and future `docs/api/resp-protocol.md`

---

## 6. Security Model

### 6.1 Zero-Trust Execution

AGEniX assumes **all execution is untrusted**:

- Workers cannot access LLMs or external APIs
- Tools execute in isolation with minimal privileges
- No inter-worker communication (embarrassingly parallel)
- Session keys required for all authenticated operations

### 6.2 Security Layers

**Authentication:**
- Session-key based (constant-time comparison)
- Key rotation support
- No credentials in logs or error messages

**Input Validation:**
- Strict JSON schema validation
- Command injection prevention
- Path traversal prevention
- Resource limits (max plan size, task count, timeout)

**Execution Isolation:**
- Tools run as separate processes
- Timeout enforcement per task
- No shell interpretation (direct process spawning)
- stdout/stderr capture and sanitization

**See:** `docs/zero-trust/zero-trust-execution.md` for detailed security architecture

---

## 7. Data Flow

### 7.1 Typical Workflow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User provides natural-language intent + input data      │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. AGX (Planner)                                            │
│    - Consults tool registry (--describe contracts)          │
│    - Uses LLM to generate JSON Plan                         │
│    - Validates Plan against schema                          │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Local or Distributed Execution?                          │
│    ┌────────────────┐              ┌─────────────────┐      │
│    │ Local (Phase 1)│              │ Distributed     │      │
│    │ AGX executes   │              │ Submit to AGQ   │      │
│    │ directly       │              │                 │      │
│    └────────────────┘              └────────┬────────┘      │
└─────────────────────────────────────────────┼───────────────┘
                                              │
                                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. AGQ (Queue Manager)                                      │
│    - Stores Plan (reusable)                                 │
│    - Creates Job(s) with input data                         │
│    - Enqueues to queue:ready                                │
│    - Tracks Job state                                       │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. AGW (Worker) - pulls Job via BRPOP                       │
│    - Validates session key                                  │
│    - Executes Tasks sequentially                            │
│    - Pipes stdout between Tasks (input_from_task)           │
│    - Captures stdout/stderr/exit codes                      │
│    - Halts on first failure                                 │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Results returned to AGQ                                  │
│    - Job status updated (completed/failed)                  │
│    - Outputs stored                                         │
│    - Logs captured                                          │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. User retrieves results via AGX                           │
│    - Query Job status                                       │
│    - Fetch outputs                                          │
│    - Review logs                                            │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Action Fan-Out (Batch Processing)

```
AGX: Create Action (1 Plan + N inputs)
  ↓
AGQ: Fan out to N Jobs
  ↓
AGW Workers: Pull Jobs in parallel
  ↓
AGQ: Aggregate results
  ↓
AGX: Retrieve all results
```

---

## 8. Future Extensions

### Phase 2: Enhanced Capabilities
- Graph-based execution (DAGs instead of linear plans)
- Workflow orchestration (chained Actions with conditionals)
- AU lifecycle manager (install, update, version management)
- Semantic tool registry (LLM-searchable capability database)

### Phase 3: Distributed Enhancements
- Clustered AGQ (multi-node queue manager)
- Worker capability matching (schedule Jobs to specialized workers)
- Result streaming (real-time Job output)
- Advanced retry strategies (exponential backoff, circuit breakers)

### Phase 4: Observability & Operations
- Metrics and monitoring (Prometheus integration)
- Distributed tracing (OpenTelemetry)
- Web UI for operations (Job visualization, debugging)
- Agent evaluation framework (AU performance benchmarks)

### Security Enhancements
- Unix domain sockets (eliminate network exposure)
- mTLS for distributed deployments
- Scoped session keys (per-worker, per-job)
- Audit logging (compliance, forensics)

---

## Related Documentation

### Architecture
- [Execution Layers](./execution-layers.md) - Canonical nomenclature (Task, Plan, Job, Action, Workflow)
- [Job Schema](./job-schema.md) - Job envelope specification
- [AGX Dual-Model Planning](./agx-dual-model.md) - Echo/Delta planning architecture
- [Component Boundaries](./agx-agq-agw.md) - Detailed responsibilities

### Specifications
- [Agentic Unit Spec](../au-specs/agentic-unit-spec.md) - AU contract requirements
- [Tool Contracts](../tools/tool-contracts.md) - `--describe` and CLI behavior

### Security
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Security model and constraints

### Deployment
- [Deployment Overview](../deployment/overview.md) - Deployment patterns and topologies

### Planning
- [Planner Models](../planning/planner-models.md) - LLM backend configuration

---

## Glossary

- **AU (Agentic Unit)**: Single-purpose tool that implements a specific capability
- **Task**: Atomic execution unit (single tool/AU call)
- **Plan**: Reusable definition of ordered Tasks
- **Job**: Runtime instance of a Plan with specific input data
- **Action**: Batch execution of same Plan with multiple inputs
- **Workflow**: Multi-Action orchestration with conditional logic (future)
- **RESP**: REdis Serialization Protocol (communication standard)
- **Session Key**: Cryptographic authentication token

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly or on major architectural changes
**Questions?** Start with [execution-layers.md](./execution-layers.md) for nomenclature
