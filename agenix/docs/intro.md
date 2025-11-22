---
sidebar_position: 1
---

# Welcome to AGEniX

**AGEniX** is a local-first agentic execution platform that enables LLM-generated plans to be executed deterministically on your own hardware.

## Key Principles

- **Zero external dependencies** - Pure Rust binaries, no cloud services required
- **Local-first** - Your data stays on your machine
- **Deterministic execution** - Plans execute predictably and reproducibly
- **Zero-trust architecture** - Workers never generate plans, only execute them
- **Unix philosophy** - Composable tools that do one thing well

## System Components

AGEniX consists of three core components:

### AGX (Planner)
The planner CLI that transforms natural-language instructions into deterministic JSON plans using local LLMs (Ollama, llama.cpp).

- **Repository**: [github.com/agenix-sh/agx](https://github.com/agenix-sh/agx)
- **Role**: Plan generation and orchestration
- **Phase**: 1 (Active development)

### AGQ (Queue)
The queue manager that stores plans, manages jobs, and coordinates worker execution via RESP protocol.

- **Repository**: [github.com/agenix-sh/agq](https://github.com/agenix-sh/agq)
- **Role**: Job scheduling and state management
- **Phase**: 1 (Active development)

### AGW (Worker)
The stateless worker that executes plan tasks deterministically, with no LLM capabilities.

- **Repository**: [github.com/agenix-sh/agw](https://github.com/agenix-sh/agw)
- **Role**: Task execution
- **Phase**: 1 (Active development)

## Quick Start

:::info Coming Soon
Installation instructions and quick start guide will be added when Phase 1 components reach stable release.
:::

## Learn More

- [System Architecture](./architecture/system-overview.md) - Understand how the components work together
- [Execution Layers](./architecture/execution-layers.md) - Learn about Tasks, Plans, Jobs, Actions, and Workflows
- [Security Guidelines](./development/security-guidelines.md) - Security-first development practices
- [Roadmap](./roadmap/roadmap-structure.md) - See what's coming next

## Community

- **GitHub Organization**: [github.com/agenix-sh](https://github.com/agenix-sh)
- **Issues**: [github.com/agenix-sh/agenix/issues](https://github.com/agenix-sh/agenix/issues)
- **License**: Dual licensed under MIT OR Apache-2.0
