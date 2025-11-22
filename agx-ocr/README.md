# agx-ocr

AGEniX Agentic Unit (AU) for OCR using DeepSeek OCR models.

**For AU contract specifications, architecture guidelines, and the AGEniX ecosystem overview, see the [AGEniX central repository](https://github.com/agenix-sh/agenix).**

## Overview

`agx-ocr` is an **Agentic Unit** designed to work within the AGEniX ecosystem. It performs optical character recognition (OCR) on images using the DeepSeek-OCR vision-language model, following the AGEniX principles of:

- **Zero-trust execution**: No automatic model downloads; explicit model path required
- **Stdin/stdout interface**: Reads binary image data from stdin, writes JSON to stdout
- **Stateless operation**: Each invocation is independent
- **Structured outputs**: Always returns valid JSON conforming to the AU contract
- **Custom prompts**: Task-specific instructions for charts, tables, invoices, etc.

### Key Features

- 📊 **Chart Extraction** - Parse bar charts, line charts, pie charts into structured data
- 📋 **Table Recognition** - Convert visual tables to JSON/CSV
- 💰 **Invoice/Financial Docs** - Extract structured data from receipts and invoices
- 🎯 **Custom Prompts** - Specify extraction requirements via CLI
- 🚀 **Metal GPU Support** - Fast inference on Apple Silicon
- 📦 **Small Binary** - 7MB optimized release build

## Quick Start

### Prerequisites

- Rust 1.91+ (edition 2021)
- ~13 GB RAM minimum (for model + inference)
- Apple Silicon Mac with Metal support (recommended) or x86_64 CPU

### 1. Build

```bash
cargo build --release
```

### 2. Download Model

**Recommended: Use the download script**

```bash
# Install Hugging Face CLI first (if not already installed)
pip install huggingface_hub
# or: uv pip install huggingface_hub

# Download model files (~6.68 GB)
./download-model.sh
```

This downloads the DeepSeek-OCR model from HuggingFace to `~/models/deepseek-ocr`:
- `config.json` - Model configuration
- `tokenizer.json` - Tokenizer
- `model.safetensors` - FP16 weights (~6.67 GB)

See [docs/MODEL_SETUP.md](docs/MODEL_SETUP.md) for alternative download methods.

### 3. Run OCR

**Basic Usage:**
```bash
# Using --model-path flag
cat image.png | ./target/release/agx-ocr --model-path ~/models/deepseek-ocr

# Or using MODEL_PATH environment variable
export MODEL_PATH=~/models/deepseek-ocr
cat image.png | ./target/release/agx-ocr
```

**With Custom Prompts:**
```bash
# Extract chart data as JSON
cat chart.png | ./target/release/agx-ocr \
  "<image>\nExtract all chart data as JSON" \
  --model-path ~/models/deepseek-ocr

# Or using --prompt flag
cat chart.png | ./target/release/agx-ocr \
  --model-path ~/models/deepseek-ocr \
  --prompt "<image>\nExtract table data with headers and rows"

# Using prompt templates
cat invoice.png | ./target/release/agx-ocr \
  "$(cat prompts/chart-to-json.txt)" \
  --model-path ~/models/deepseek-ocr
```

**Example output:**

```json
{
  "text": "Extracted text content from the image...",
  "regions": [],
  "model": "deepseek-ocr (~/models/deepseek-ocr)"
}
```

### 4. Test

```bash
# Run the test script
./test.sh ~/models/deepseek-ocr
```

## Usage

### Get AU Description

```bash
./target/release/agx-ocr --describe
```

Returns the AU model card in JSON format (capabilities, inputs, outputs, configuration).

### Run OCR

```bash
# From stdin
cat /path/to/image.{png,jpg,jpeg} | agx-ocr --model-path <model-dir>

# Using environment variable
export MODEL_PATH=/path/to/model
cat image.png | agx-ocr
```

### Supported Image Formats

- PNG
- JPEG
- Any format supported by the `image` crate

## Architecture

### Module Structure

- **main.rs**: CLI entry point using `clap`
- **types.rs**: Stable AU contract types (`OcrResult`, `OcrRegion`)
- **model.rs**: Model configuration and loading
- **ocr.rs**: OCR execution layer bridging image bytes to DeepSeek engine
- **describe.rs**: AU model card generation

### Two-Layer Type System

The codebase maintains separation between:
1. **AU contract types** (`types.rs`): Stable public API for AGEniX pipelines
2. **Engine types** (`ocr.rs`): Internal types that map to the `deepseek-ocr` library

This allows the AU contract to remain stable even if the underlying engine API changes.

## Model Requirements

The `--model-path` should point to a directory containing:

```
model-directory/
├── config.json          # Model configuration
├── tokenizer.json       # Tokenizer configuration
└── model.safetensors    # Model weights (~6.3 GB FP16)
```

**Memory Requirements:**
- Model weights: ~6.3 GB
- Runtime (model + activations): ~13 GB
- Recommended: 16 GB+ RAM/VRAM

**Supported Devices:**
- Apple Silicon (Metal) - Recommended
- CPU (slower, but works)
- NVIDIA CUDA (experimental via upstream)

## Development

### Build

```bash
# Debug build
cargo build

# Release build (optimized for size)
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
cargo fmt --check
```

## Documentation

### General
- [MODEL_SETUP.md](docs/MODEL_SETUP.md) - Detailed model download and setup instructions
- [MODEL_INFO.md](docs/MODEL_INFO.md) - Model specifications and details
- [CONTRACT.md](docs/CONTRACT.md) - AU contract specification
- [USAGE.md](docs/USAGE.md) - Usage examples and patterns

### Platform-Specific
- [DGX_SPARK_TARGET.md](docs/DGX_SPARK_TARGET.md) - NVIDIA DGX Spark technical analysis
- [DGX_SPARK_BUILD_GUIDE.md](docs/DGX_SPARK_BUILD_GUIDE.md) - DGX Spark build instructions
- [DGX_SPARK_SUMMARY.md](docs/DGX_SPARK_SUMMARY.md) - DGX Spark deployment summary

### Development
- [CLAUDE.md](CLAUDE.md) - Development guidelines for Claude Code
- [POC_SUMMARY.md](docs/POC_SUMMARY.md) - Proof of concept completion summary

## AGEniX Integration

This AU is designed to be orchestrated by the AGEniX planner (`agx`) and executed on workers (`agw`). The AU contract ensures:

- **Predictable I/O**: stdin (binary) → stdout (JSON)
- **Error handling**: All errors go to stderr
- **No side effects**: No file writes, no network calls (except model loading)
- **Explicit configuration**: No implicit defaults or auto-downloads

## Project Status

**Current:** ✅ PoC implementation complete
- Core OCR engine integration working
- AU contract fully implemented
- Model loading and inference functional
- Tested on macOS ARM64
- **DGX Spark build target ready** (CPU-only, ARM64 Linux)

**Platform Support:**
- ✅ macOS ARM64 (Metal GPU) - Production ready
- ✅ NVIDIA DGX Spark (ARM64, CPU) - Production ready
- ⏳ NVIDIA DGX Spark (Blackwell GPU) - Waiting on Candle CUDA support

**Upcoming:**
- Integration tests for OCR pipeline
- CI/CD automation
- GGUF model support (quantized weights)
- GPU acceleration for DGX Spark (when Candle CUDA matures)

## License

MIT

## Credits

Built on top of:
- [deepseek-ocr.rs](https://github.com/agenix-sh/deepseek-ocr.rs) - Rust implementation of DeepSeek-OCR
- [DeepSeek-OCR](https://github.com/deepseek-ai/DeepSeek-OCR) - Original Python implementation by DeepSeek
