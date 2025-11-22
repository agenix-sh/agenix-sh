# AGEnix Meta Repository

This repository is the **central coordination point** for the AGEnix ecosystem.

It owns:

- Product vision and roadmap for all AGEnix components
- Architecture and interface specifications
- Agentic Unit (AU) standards and tool contracts
- Zero-trust and security guidelines
- Deployment patterns and reference topologies
- The AGEnix public website and documentation

Individual implementation repos:

- `agenix-sh/agx` — CLI planner/orchestrator (intent → plan → execution)
- `agenix-sh/agq` — queue and scheduler for distributed execution
- `agenix-sh/agw` — worker process for executing approved plans
- `agenix-sh/agx-ocr` — OCR Agentic Unit and CLI tool

All component repos should treat this repo as the **single source of truth** for architecture, specs and roadmap.
