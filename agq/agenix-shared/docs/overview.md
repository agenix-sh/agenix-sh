# AGEnix Documentation Overview

This directory contains the **canonical documentation** for the AGEnix ecosystem.

All component repositories (`agx`, `agq`, `agw`, `agx-ocr`, and future AU tools) should link to these files rather than copy them. This avoids drift and keeps architecture and interfaces consistent.

## Structure

- `architecture/` — high-level system diagrams and component responsibilities.
- `execution-model/` — local vs distributed execution flows and data paths.
- `planning/` — planner behaviour, LLM backends, and plan generation.
- `au-specs/` — Agentic Unit (AU) contracts and lifecycle.
- `tools/` — CLI tool conventions and `--describe` model cards.
- `zero-trust/` — security principles and constraints (stdin/stdout/stderr, signing).
- `deployment/` — reference topologies, environments and operational guidance.
- `roadmap/` — cross-repo roadmap and release planning.
