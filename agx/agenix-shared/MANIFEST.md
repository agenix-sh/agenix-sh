# AGEnix Project Manifest

This file lists the core components of the AGEnix ecosystem and their responsibilities.

## Core Orchestration

- **agx** (repo: `agenix-sh/agx`)
  - CLI planner/orchestrator.
  - Accepts stdin + intent, produces a JSON plan and/or executes it locally.
  - Integrates with LLM planner backends (e.g. Ollama, Tinker-fine-tuned models).

- **agq** (repo: `agenix-sh/agq`)
  - Distributed queue and scheduler.
  - Receives signed plans from `agx` and assigns them to workers.
  - Backed by a queueing system (e.g. Redis) but exposes an AGEnix-specific interface.

- **agw** (repo: `agenix-sh/agw`)
  - Dumb, zero-trust worker process.
  - Executes only signed, pre-approved plans from `agq`.
  - Runs tools as separate processes via stdin/stdout/stderr.
  - Has no direct access to LLM providers.

## Agentic Units / Tools

- **agx-ocr** (repo: `agenix-sh/agx-ocr`)
  - OCR Agentic Unit and CLI tool.
  - Accepts binary input (images/PDFs) and outputs structured JSON.
  - Can run standalone or as part of an `agx` plan (locally or via `agq`/`agw`).

Future AU tools should follow the patterns and contracts defined under `/docs/au-specs` and `/specs`.
