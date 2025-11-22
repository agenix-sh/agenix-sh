# agx-ocr AU Contract

This document describes the contract for `agx-ocr` as an Agentic Unit (AU).

## I/O Contract

- **Input**: binary image data via `stdin` (PNG, JPEG, etc.)
- **Output**: structured JSON via `stdout`.
- **Errors / logs**: written to `stderr`.

## Model Loading

- `agx-ocr` **never** downloads models automatically.
- A GGUF model **must** be supplied via:
  - `--model-path /path/to/model.gguf`, or
  - `$MODEL_PATH` environment variable.

If no model path is provided, the tool exits with a non-zero exit code and
does not write anything to `stdout`.

## Describe Contract

- `agx-ocr --describe` prints a JSON model card compatible with
  `specs/describe.schema.json` in the central `agenix` meta repository.
- This card can be used by `agx` to populate its tool registry and by other AUs
  to reason about capabilities.

## Architecture Reference

The canonical architecture, AU specification, and tool contracts are defined in:

- https://github.com/agenix-sh/agenix

This repository (`agx-ocr`) should not duplicate those documents; it only
implements the AU behaviour described there.
