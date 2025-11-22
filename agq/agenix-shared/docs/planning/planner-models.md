# Planner Models and Backends

`agx` delegates planning to an LLM backend. This document describes how planner models are used and how backends can be swapped.

## Responsibilities of the Planner

Given:

- an input payload (stdin),
- a natural-language intent,
- a tool registry (available tools + `--describe` metadata),

the planner must produce:

- a valid JSON plan matching `/specs/plan.schema.json`,
- safe, deterministic tool invocations,
- minimal, composable steps that can run in local or distributed mode.

## Backends

Examples of supported/planned backends:

- Local LLMs via Ollama.
- Fine-tuned models via Tinker.
- Remote APIs (OpenAI, DeepSeek, Anthropic) where appropriate.

All backends must implement a common trait so `agx` can switch planner models without changing core logic.
