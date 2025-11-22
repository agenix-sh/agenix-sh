# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`agx-ocr` is an AGEniX Agentic Unit (AU) for OCR using DeepSeek GGUF models. It's designed to be orchestrated by the AGEniX planner (`agx`) and executed on workers (`agw`) in a zero-trust environment.

**Key Architecture Points:**
- Reads binary image data from `stdin` (PNG, JPEG)
- Writes structured JSON to `stdout`
- Logs/errors go to `stderr`
- Never downloads models automatically; requires explicit model path via `--model-path` or `$MODEL_PATH`

## Development Commands

### Build
```bash
cargo build
cargo build --release
```

### Run
```bash
# With model path
cat image.png | cargo run -- --model-path /path/to/model.gguf

# Get AU description (model card)
cargo run -- --describe
```

### Tests
```bash
# Run all tests
cargo test

# Run a single test
cargo test placeholder

# Run integration tests (in tests/ directory)
cargo test --test basic
```

### Linting
```bash
cargo clippy
cargo fmt --check
```

## Code Architecture

### Module Structure
- **main.rs**: CLI entry point using `clap`. Handles `--describe` flag and stdin/stdout I/O
- **types.rs**: Stable AU contract types (`OcrResult`, `OcrRegion`) - these define the public API
- **model.rs**: Configuration for model loading (strict mode: path MUST be provided)
- **ocr.rs**: OCR execution layer that bridges between image bytes and DeepSeek engine
- **describe.rs**: AU model card generation for AGEniX registry (follows `describe.schema.json`)

### Important Architectural Patterns

**Two-Layer Type System:**
The codebase maintains separation between:
1. **AU contract types** (`types.rs`): Stable public API for AGEniX pipelines
2. **Engine types** (`ocr.rs`): Internal `EngineRegion`/`EngineResult` that map to the `deepseek-ocr` library

This allows the AU contract to remain stable even if the underlying engine API changes.

**Stub Implementation:**
The `run_engine()` function in `ocr.rs:53` is currently a stub that returns an error. This is where the actual DeepSeek OCR engine integration needs to be wired up using the `deepseek-ocr` crate from `https://github.com/agenix-sh/deepseek-ocr.rs`.

**Strict Model Loading:**
Unlike typical ML tools, this AU deliberately fails if no model path is provided (no defaults, no auto-downloads). This is intentional for the zero-trust worker environment.

### Dependencies
- `deepseek-ocr`: Git dependency from `agenix-sh/deepseek-ocr.rs` (engine integration)
- `clap`: CLI argument parsing with derive macros
- `image`: Image decoding from memory
- `serde`/`serde_json`: JSON serialization for stdout

### Release Profile
The release build is optimized for size (`opt-level = "z"`, LTO enabled) suitable for distribution to workers.

## AU Contract Compliance

This AU follows the AGEniX specification (see `https://github.com/agenix-sh/agenix`):
- `--describe` flag outputs JSON matching `describe.schema.json`
- Binary stdin, JSON stdout, errors to stderr
- No automatic model downloads (worker provides model via path)

Refer to `docs/CONTRACT.md` for the full AU contract details.

## Shared Claude Code Skills & Agents

This repository uses shared Claude Code configuration from the agenix repo (via git submodule at `agenix-shared/.claude/`):

### Available Skills (Auto-Activated)
- **agenix-architecture** - Enforces execution layer nomenclature (Task/Plan/Job/Action/Workflow)
- **agenix-security** - OWASP Top 10, zero-trust principles, constant-time comparisons
- **agenix-testing** - TDD practices, 80% coverage minimum, 100% for security-critical code
- **rust-agenix-standards** - Rust error handling, async patterns, type safety idioms

### Available Agents (Explicit Invocation)
- **rust-engineer** - Deep Rust expertise for async, performance, safety
- **security-auditor** - Vulnerability detection and prevention
- **github-manager** - Issue/PR creation with proper templates and labels
- **multi-repo-coordinator** - Cross-repository change coordination

See `.claude/README.md` for detailed documentation on skill activation and agent usage.
