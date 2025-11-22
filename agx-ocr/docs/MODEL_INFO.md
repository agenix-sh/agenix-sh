# DeepSeek-OCR Model Information

## Model Details

- **Name**: DeepSeek-OCR
- **Repository**: [deepseek-ai/DeepSeek-OCR](https://huggingface.co/deepseek-ai/DeepSeek-OCR)
- **License**: MIT
- **Parameters**: ~3 billion (570M active per token)
- **Architecture**: SAM+CLIP vision tower + DeepSeek-V2 MoE decoder
- **Precision**: BF16 (original), FP16 (for Metal/CUDA inference)

## Required Files

For `agx-ocr` to work, you need exactly these 3 files:

| File | Size | Description |
|------|------|-------------|
| `config.json` | ~2.67 KB | Model configuration |
| `tokenizer.json` | ~9.98 MB | Tokenizer configuration |
| `model.safetensors` | ~6.67 GB | Model weights (FP16 format) |

**Total**: ~6.68 GB

## Download Information

### HuggingFace Repository
- **Repo ID**: `deepseek-ai/DeepSeek-OCR`
- **Original weights file**: `model-00001-of-000001.safetensors`
- **Note**: We rename it to `model.safetensors` for consistency

### Download Methods

1. **Using our script** (recommended):
   ```bash
   ./download-model.sh
   ```

2. **Using huggingface-cli directly**:
   ```bash
   huggingface-cli download deepseek-ai/DeepSeek-OCR \
     config.json tokenizer.json model-00001-of-000001.safetensors \
     --local-dir ~/models/deepseek-ocr \
     --local-dir-use-symlinks False
   ```

3. **Using deepseek-ocr.rs auto-download**:
   ```bash
   cd ../deepseek-ocr.rs
   cargo run --release -p deepseek-ocr-cli -- --help
   # Files cached to: ~/Library/Caches/deepseek-ocr/models/deepseek-ocr/
   ```

## Hardware Requirements

### Minimum
- **RAM**: 13 GB (6.67 GB model + ~6-7 GB activations)
- **Disk**: 7 GB free space
- **CPU**: Any modern x86_64 or ARM64 processor

### Recommended
- **RAM**: 16 GB+
- **GPU**: Apple Silicon (Metal) or NVIDIA with 16+ GB VRAM
- **Disk**: 10 GB free space (for model + cache)

## Performance Characteristics

| Platform | Device | Inference Speed | Memory Usage |
|----------|--------|----------------|--------------|
| macOS Apple Silicon | Metal (FP16) | Fast (~2-5s per image) | ~13 GB |
| macOS Apple Silicon | CPU (BF16) | Slow (~30-60s per image) | ~13 GB |
| Linux x86_64 | CPU (BF16) | Slow (~30-60s per image) | ~13 GB |
| Linux x86_64 | CUDA (FP16) | Fast (~2-5s per image) | ~13 GB VRAM |

*Note: Actual speeds depend on image size and complexity. First inference is slower due to model loading.*

## Model Capabilities

- **Text extraction**: Handles printed, handwritten, and complex layouts
- **Multi-language**: Supports multiple languages (trained on diverse data)
- **Document types**: PDFs, screenshots, scanned documents, forms, tables
- **Output format**: Plain text (Markdown in original implementation)

## Limitations

- **No bounding boxes**: The current Rust implementation doesn't expose region-level coordinates
- **Single-turn inference**: Each image is processed independently (no context from previous images)
- **Large memory footprint**: Requires significant RAM/VRAM
- **Cold start**: First inference takes longer (~30-60s) to load model into memory

## Alternative Models

If you need lower memory usage, consider:
- **PaddleOCR-VL**: ~4.7 GB model, ~9 GB runtime (available in deepseek-ocr.rs)
- **Tesseract**: Traditional OCR, much smaller but less accurate

## Version History

- **v0.1.0**: Initial safetensors release (~6.67 GB FP16)
- Future: Quantized versions (GGUF) may be available for lower memory usage

## Additional Resources

- [HuggingFace Model Card](https://huggingface.co/deepseek-ai/DeepSeek-OCR)
- [DeepSeek-OCR GitHub](https://github.com/deepseek-ai/DeepSeek-OCR)
- [deepseek-ocr.rs (Rust impl)](https://github.com/agenix-sh/deepseek-ocr.rs)
