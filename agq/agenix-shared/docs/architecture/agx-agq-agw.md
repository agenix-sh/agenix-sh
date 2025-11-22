# AGX / AGQ / AGW Architecture

This document captures the responsibilities and boundaries between the three core orchestration components.

## AGX — Planner / Orchestrator

- Accepts stdin plus a natural-language intent.
- Uses a planner model (LLM backend) to generate a JSON plan.
- Can execute the plan locally or submit it to `agq` for distributed execution.
- Knows about available tools via a registry and `--describe` model cards.

## AGQ — Queue / Scheduler

- Receives signed plans plus input payloads from `agx`.
- Stores jobs in a queueing backend (e.g. Redis).
- Assigns jobs to workers (`agw`) based on capabilities, queues and policies.
- Tracks job state (queued, running, completed, failed) and exposes status APIs/CLI.

## AGW — Worker

- Registers with `agq` using a session key and capability description.
- Pulls jobs from `agq` and verifies plan signatures.
- Executes plan steps **linearly**, using stdin/stdout/stderr for tools.
- Has no direct access to LLMs; it only runs tools defined in the plan.
- Returns outputs and logs back to `agq` (or directly to `agx`).

The system is intentionally **embarrassingly parallel**: there is no inter-worker communication layer.
