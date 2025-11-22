# AGEnix Architecture Overview

AGEnix is an ecosystem of small, composable tools and services that bring Agent Oriented Architecture (AOA) principles to the command line.

## Quick Start

At a high level:

- `agx` provides **planning and orchestration** for CLI-centric workflows.
- `agq` provides a **queue/scheduler** for distributed execution.
- `agw` provides **zero-trust workers** that run approved plans.
- AU tools like `agx-ocr` provide **specialist capabilities** that can be used standalone or inside plans.

Core pattern:

```text
input (stdin) → agx [intent] → plan (JSON) → execution (local or via agq/agw) → output (stdout)
```

This repository is the canonical source of truth for how these components fit together.

---

## Architecture Documentation Structure

This directory contains the authoritative architecture documentation for the AGEniX ecosystem:

### Core Architecture Documents

1. **[System Overview](./system-overview.md)** - Comprehensive high-level architecture
   - System components and their responsibilities
   - Execution model and data flow
   - Communication architecture (RESP protocol)
   - Security model
   - Future roadmap

2. **[Execution Layers](./execution-layers.md)** - Canonical nomenclature (v0.1)
   - The five execution layers: Task → Plan → Job → Action → Workflow
   - Mapping to codebases (AGX, AGQ, AGW)
   - **Critical reference for all development**

3. **[Job Schema](./job-schema.md)** - Job envelope specification (v0.2)
   - Job structure and validation rules
   - Aligned with execution-layers nomenclature
   - RESP protocol submission format

4. **[Component Boundaries](./agx-agq-agw.md)** - AGX/AGQ/AGW responsibilities
   - Clear separation of concerns
   - Communication patterns
   - Embarrassingly parallel design

5. **[AGX Dual-Model Planning](./agx-dual-model.md)** - Echo/Delta architecture
   - Fast planning (Echo model)
   - Thorough planning (Delta model)
   - Model selection strategy

### Implementation-Specific Architecture

For detailed implementation architecture, see individual repositories:

- **AGX**: `agx/docs/ARCHITECTURE.md` - Planner implementation details
- **AGQ**: `agq/ARCHITECTURE.md` - Queue manager implementation details
- **AGW**: `agw/docs/ARCHITECTURE.md` - Worker implementation details
- **AGX-OCR**: `agx-ocr/README.md` - First AU reference implementation

---

## Navigation Guide

**New to AGEniX?** Start with:
1. [System Overview](./system-overview.md) for the big picture
2. [Execution Layers](./execution-layers.md) for nomenclature
3. [Component Boundaries](./agx-agq-agw.md) for how pieces fit together

**Developing an AU?** Read:
1. [Execution Layers](./execution-layers.md) to understand Tasks
2. [../au-specs/agentic-unit-spec.md](../au-specs/agentic-unit-spec.md) for AU contract
3. Reference `agx-ocr` as example implementation

**Working on AGX/AGQ/AGW?** Review:
1. [Execution Layers](./execution-layers.md) for canonical terminology
2. [Job Schema](./job-schema.md) for data structures
3. Component-specific docs in respective repos

**Planning integration?** Check:
1. [System Overview](./system-overview.md) for data flow
2. [../tools/tool-contracts.md](../tools/tool-contracts.md) for API contracts
3. [../zero-trust/zero-trust-execution.md](../zero-trust/zero-trust-execution.md) for security model

---

## Key Principles

1. **Single Source of Truth**: Cross-cutting architecture docs live here in `agenix`
2. **Implementation Details Stay Local**: Component-specific details remain in their repos
3. **Nomenclature is Sacred**: Always use terms from [execution-layers.md](./execution-layers.md)
4. **Link, Don't Duplicate**: Component repos link to canonical docs, not copy them

---

**Maintained by:** AGX Core Team
**Last Updated:** 2025-11-17
**Questions?** Start with [system-overview.md](./system-overview.md)
