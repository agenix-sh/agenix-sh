# Agentic Unit (AU) Specification

This document defines the expectations for an AGEnix Agentic Unit (AU).

## AU Principles

- **Single responsibility**: each AU should do one class of task well (e.g. OCR, summarisation).
- **CLI-native**: AUs must expose a command-line interface that reads from stdin and writes to stdout/stderr.
- **Describeable**: AUs must support a `--describe` flag that returns a machine-readable model card.
- **Composable**: AUs should be usable standalone *and* within `agx` plans.

## AU Lifecycle

1. Design AU behaviour and input/output contract.
2. Implement AU as a CLI tool (e.g. `agx-ocr`).
3. Implement `--describe` model card output matching `/specs/describe.schema.json`.
4. Register AU in the tool registry so `agx` can plan with it.
5. Optionally add distributed support via `agq`/`agw`.
