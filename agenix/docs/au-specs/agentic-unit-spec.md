# Agentic Unit (AU) Specification

**Version:** 1.0
**Status:** Canonical Contract Specification
**Last Updated:** 2025-11-17

This document defines the contract for Agentic Units (AUs) in the AGEniX ecosystem. AUs are specialized tools that AGX can orchestrate and AGW can execute.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [AU Contract](#au-contract)
4. [Command-Line Interface](#command-line-interface)
5. [Input/Output Protocol](#inputoutput-protocol)
6. [Model Card (--describe)](#model-card---describe)
7. [Error Handling](#error-handling)
8. [Security Requirements](#security-requirements)
9. [Examples](#examples)
10. [Validation Checklist](#validation-checklist)

---

## 1. Overview

### What is an Agentic Unit?

An **Agentic Unit (AU)** is a command-line tool that:
- Performs a specialized task (OCR, PDF parsing, data transformation, etc.)
- Follows a strict stdin/stdout contract for AGEniX orchestration
- Provides machine-readable metadata via `--describe`
- Runs in zero-trust worker environments

### AU vs Regular CLI Tool

| Aspect | Regular CLI Tool | Agentic Unit |
|--------|------------------|--------------|
| **Input** | Files, arguments, prompts | Binary stdin + arguments |
| **Output** | Files, stdout (text) | Structured JSON to stdout |
| **Errors** | Mixed stdout/stderr | Errors only to stderr |
| **Discovery** | Manual documentation | `--describe` JSON schema |
| **Environment** | Assumes network, filesystem | Zero-trust, sandboxed |
| **Model Loading** | Auto-download, defaults | Explicit path, no auto-download |

---

## 2. Core Principles

### 2.1 Single Responsibility

Each AU does **one thing well**:
- ✅ `agx-ocr`: Extract text from images
- ✅ `agx-pdf`: Parse PDF to structured data
- ❌ `agx-swiss-army-knife`: OCR + PDF + translation + ...

### 2.2 Stateless Execution

AUs are **pure functions**:
- No persistent state between invocations
- No databases, caches, or session files
- Idempotent: Same input → same output

### 2.3 Data Flow Contract

```
┌─────────────────────────────────────────┐
│  AGW Worker (Untrusted Environment)    │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │   Agentic Unit Process            │  │
│  │                                   │  │
│  │  stdin ──────▶ [AU Logic] ──────▶ stdout (JSON)
│  │                     │             │  │
│  │                     └──────────▶ stderr (Errors)
│  │                                   │  │
│  └───────────────────────────────────┘  │
│                                         │
└─────────────────────────────────────────┘
```

### 2.4 Zero-Trust Execution

AUs must assume:
- No network access (blocked by worker)
- Limited filesystem access (only specified paths)
- No auto-downloading of models or dependencies
- Timeouts enforced externally (by AGW)
- Sandboxed execution (cgroups, namespaces)

---

## 3. AU Contract

### 3.1 Required Behaviors

Every AU **MUST**:

1. **Accept binary input via stdin**
   - Read data as byte stream (not text)
   - Handle arbitrary binary formats (images, PDFs, etc.)

2. **Produce structured JSON output via stdout**
   - Must be valid JSON
   - Must match declared output schema
   - Pretty-printed recommended (for debugging)

3. **Send errors to stderr only**
   - Never mix errors with stdout JSON
   - Use structured logging if needed (stderr)

4. **Implement `--describe` flag**
   - Output JSON matching `describe.schema.json`
   - Includes name, version, capabilities, inputs, outputs, config

5. **Accept configuration via CLI args or environment variables**
   - `--model-path` or `$MODEL_PATH` for models
   - Other config via flags (e.g., `--prompt`, `--format`)

6. **Exit with appropriate codes**
   - `0` = Success
   - `1` = General error
   - `2` = Invalid input/arguments
   - `3+` = Domain-specific errors (optional)

### 3.2 Forbidden Behaviors

AUs **MUST NOT**:

1. **Auto-download models or dependencies**
   - Models must be provided via explicit path
   - Fail fast if model not found

2. **Write to filesystem without explicit permission**
   - No temp files unless in specified temp dir
   - No logging to files (use stderr)

3. **Make network requests**
   - No API calls to external services
   - No telemetry or analytics

4. **Read configuration from global files**
   - No `~/.config`, no `/etc/` files
   - Config via CLI args or env vars only

5. **Rely on specific working directory**
   - All paths must be absolute or relative to explicit base

---

## 4. Command-Line Interface

### 4.1 Required Flags

```bash
AU_NAME [OPTIONS] [PROMPT]
```

**`--describe`** (required)
- Prints AU model card as JSON
- Exits after printing (does not process stdin)
- Example: `agx-ocr --describe`

### 4.2 Common Optional Flags

**`--model-path <PATH>`** or **`$MODEL_PATH`** (common for ML-based AUs)
- Filesystem path to model file (GGUF, ONNX, etc.)
- No default value (fail if not provided)
- Example: `agx-ocr --model-path /models/deepseek-ocr.gguf`

**`--prompt <TEXT>`** (optional, for vision/language models)
- Custom user prompt
- Example: `agx-ocr --prompt "Extract table data"`

**`PROMPT`** (positional argument, alternative to `--prompt`)
- Example: `agx-ocr "Extract chart values" < chart.png`

### 4.3 Example Invocations

```bash
# Get model card
agx-ocr --describe

# Run OCR with model
cat image.png | agx-ocr --model-path /models/ocr.gguf > result.json

# Run with custom prompt
echo "image.png" | agx-ocr --model-path /models/ocr.gguf --prompt "Extract invoice data"

# Positional prompt
agx-ocr "Describe this chart" --model-path /models/ocr.gguf < chart.png
```

---

## 5. Input/Output Protocol

### 5.1 Input: Binary stdin

**Format:**
- Raw binary bytes (PNG, JPEG, PDF, etc.)
- No assumptions about encoding
- Read entire stream until EOF

**Example (Rust):**
```rust
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    // Process buf as binary data
    let result = process_image(&buf)?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### 5.2 Output: JSON stdout

**Format:**
- Valid JSON (parseable by any JSON parser)
- Pretty-printed recommended (readability)
- No extra text before/after JSON
- Schema matches declared output in `--describe`

**Example Output (agx-ocr):**
```json
{
  "text": "Hello World",
  "regions": [
    {
      "text": "Hello",
      "confidence": 0.98,
      "bbox": [10.0, 20.0, 100.0, 50.0]
    },
    {
      "text": "World",
      "confidence": 0.95,
      "bbox": [110.0, 20.0, 200.0, 50.0]
    }
  ],
  "model": "deepseek-ocr-v1"
}
```

### 5.3 Errors: stderr

**Format:**
- Human-readable error messages
- Optional: Structured logging (JSON lines)
- Never printed to stdout

**Example (stderr):**
```
Error: Failed to decode image
Caused by: Invalid PNG header
```

**Example (structured logging to stderr):**
```json
{"level":"error","msg":"Failed to decode image","error":"Invalid PNG header"}
```

---

## 6. Model Card (--describe)

### 6.1 Schema

Model card must conform to `specs/describe.schema.json`:

```json
{
  "name": "string",           // AU name (e.g., "agx-ocr")
  "version": "string",        // Semver version
  "description": "string",    // Human-readable summary
  "capabilities": ["string"], // Tags: ["ocr", "image-to-text"]
  "inputs": [
    {
      "media_type": "string", // MIME type (e.g., "image/*")
      "description": "string" // Human-readable
    }
  ],
  "outputs": [
    {
      "media_type": "string", // MIME type (e.g., "application/json")
      "description": "string"
    }
  ],
  "config": {
    "param-name": {
      "type": "string",       // Type: string, integer, boolean
      "description": "string",
      "default": null         // Default value (or null if required)
    }
  }
}
```

### 6.2 Example (agx-ocr)

```json
{
  "name": "agx-ocr",
  "version": "0.1.0",
  "description": "Agentic Unit for OCR using DeepSeek GGUF models. Reads image bytes from stdin and outputs structured JSON.",
  "capabilities": ["ocr", "image-to-text"],
  "inputs": [
    {
      "media_type": "image/*",
      "description": "Binary image data (PNG, JPEG) via stdin"
    }
  ],
  "outputs": [
    {
      "media_type": "application/json",
      "description": "OCR result as structured JSON (text, regions, confidences)"
    }
  ],
  "config": {
    "model-path": {
      "type": "string",
      "description": "Filesystem path to DeepSeek GGUF model file.",
      "default": null
    }
  }
}
```

### 6.3 Capabilities Taxonomy

Common capability tags:

| Capability | Description |
|------------|-------------|
| `ocr` | Optical character recognition |
| `image-to-text` | Image understanding/description |
| `pdf` | PDF parsing/extraction |
| `table-extraction` | Extract tables from documents |
| `nlp` | Natural language processing |
| `translation` | Language translation |
| `summarization` | Text summarization |
| `classification` | Classify documents/images |
| `entity-extraction` | Named entity recognition |

---

## 7. Error Handling

### 7.1 Error Categories

**Input Errors (exit code 2):**
- Invalid image format
- Corrupt binary data
- Missing required arguments
- Model file not found

**Processing Errors (exit code 1):**
- Model inference failed
- Out of memory
- Unsupported feature

**Example:**
```rust
use anyhow::{bail, Context, Result};

fn main() -> Result<()> {
    let model_path = std::env::var("MODEL_PATH")
        .context("MODEL_PATH not set")?;

    if !std::path::Path::new(&model_path).exists() {
        bail!("Model file not found: {}", model_path);
    }

    // ... process
    Ok(())
}
```

### 7.2 Error Messages

**Good error messages:**
- ✅ `Error: Model file not found: /models/ocr.gguf`
- ✅ `Error: Invalid image format. Expected PNG or JPEG, got PDF`
- ✅ `Error: Image too large (50MB). Maximum supported: 20MB`

**Bad error messages:**
- ❌ `Error: Something went wrong`
- ❌ `Segmentation fault`
- ❌ `panic: index out of bounds`

### 7.3 Graceful Degradation

When possible, return partial results:

```json
{
  "text": "Partial OCR result",
  "regions": [...],
  "warnings": [
    "Low confidence region skipped",
    "Unrecognized character: \u00ff"
  ]
}
```

---

## 8. Security Requirements

### 8.1 Input Validation

**Always validate:**
- File format magic bytes
- Size limits (prevent DoS)
- Image dimensions (prevent OOM)
- String lengths (prevent buffer overflows)

**Example:**
```rust
const MAX_INPUT_SIZE: usize = 50 * 1024 * 1024; // 50MB

fn validate_input(buf: &[u8]) -> Result<()> {
    if buf.len() > MAX_INPUT_SIZE {
        bail!("Input too large: {}MB (max 50MB)", buf.len() / 1024 / 1024);
    }

    // Check magic bytes for PNG
    if buf.len() >= 8 && &buf[0..8] != b"\x89PNG\r\n\x1a\n" {
        bail!("Invalid PNG header");
    }

    Ok(())
}
```

### 8.2 Path Validation

**Never trust user-provided paths:**
```rust
use std::path::{Path, PathBuf};

fn validate_model_path(path: &str) -> Result<PathBuf> {
    let p = PathBuf::from(path).canonicalize()?;

    // Prevent path traversal
    if path.contains("..") {
        bail!("Path traversal detected");
    }

    // Ensure file exists
    if !p.exists() {
        bail!("Model file not found: {}", path);
    }

    Ok(p)
}
```

### 8.3 Resource Limits

AUs should be defensive about resource usage:
- Limit memory allocation
- Set timeouts on inference
- Bound output size

```rust
use std::time::Duration;

const MAX_INFERENCE_TIME: Duration = Duration::from_secs(300); // 5 min
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB
```

### 8.4 No Secrets in Logs

Never log sensitive data to stderr:
- ❌ API keys
- ❌ Model weights (binary)
- ❌ User input verbatim (may contain PII)

---

## 9. Examples

### 9.1 Minimal AU (Echo)

```rust
use std::io::{self, Read};
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct EchoResult {
    input_size: usize,
    sha256: String,
}

fn main() -> Result<()> {
    // Read stdin
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    // Process
    let hash = sha256::digest(&buf);
    let result = EchoResult {
        input_size: buf.len(),
        sha256: hash,
    };

    // Output JSON
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### 9.2 AU with --describe

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    describe: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.describe {
        print_model_card()?;
        return Ok(());
    }

    // ... normal processing
    Ok(())
}

fn print_model_card() -> Result<()> {
    let card = serde_json::json!({
        "name": "agx-echo",
        "version": "1.0.0",
        "description": "Echo AU that returns input hash",
        "capabilities": ["hash", "checksum"],
        "inputs": [{"media_type": "*/*", "description": "Any binary data"}],
        "outputs": [{"media_type": "application/json", "description": "SHA256 hash"}]
    });
    println!("{}", serde_json::to_string_pretty(&card)?);
    Ok(())
}
```

### 9.3 AU with Model Loading

```rust
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long, env = "MODEL_PATH")]
    model_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let model_path = cli.model_path
        .ok_or_else(|| anyhow::anyhow!("MODEL_PATH required"))?;

    if !model_path.exists() {
        bail!("Model not found: {:?}", model_path);
    }

    let model = load_model(&model_path)?;

    // ... process with model
    Ok(())
}
```

---

## 10. Validation Checklist

Before publishing an AU, verify:

### Core Contract
- [ ] `--describe` flag implemented
- [ ] Outputs valid JSON matching describe.schema.json
- [ ] Reads binary data from stdin
- [ ] Writes structured JSON to stdout
- [ ] Errors go to stderr only
- [ ] Exit codes: 0 (success), 1 (error), 2 (invalid input)

### Security
- [ ] No auto-download of models
- [ ] Path validation prevents traversal
- [ ] Input size limits enforced
- [ ] No secrets logged to stderr
- [ ] No network requests

### Error Handling
- [ ] Descriptive error messages
- [ ] Graceful handling of corrupt input
- [ ] Model file not found → clear error
- [ ] Out of memory → graceful exit

### Documentation
- [ ] README.md with usage examples
- [ ] Model card includes all capabilities
- [ ] Config parameters documented
- [ ] License files present (MIT/Apache-2.0)

### Testing
- [ ] Unit tests for core logic
- [ ] Integration tests with sample inputs
- [ ] `--describe` output validates against schema
- [ ] Handles malformed input gracefully

### Performance
- [ ] Processes typical inputs within reasonable time
- [ ] Memory usage bounded
- [ ] No leaks or resource exhaustion

---

## Related Documentation

- [Testing AUs](./testing-au.md) - Comprehensive testing guide
- [Example AU Template](./example-au-template.md) - Boilerplate for new AUs
- [Describe Schema](../../specs/describe.schema.json) - JSON schema for `--describe`
- [Security Guidelines](../development/security-guidelines.md) - Security best practices
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Worker security model

---

**Maintained by:** AGX Core Team
**Review cycle:** Per major version release
**Questions?** Open an issue in the relevant AU repository or `agenix/agenix`
