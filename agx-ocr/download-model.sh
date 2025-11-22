#!/bin/bash
# Download DeepSeek-OCR model from HuggingFace
# This script uses the 'hf' CLI to download the required model files

set -e

# Activate venv if it exists in current directory
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

MODEL_DIR="${1:-$HOME/models/deepseek-ocr}"
HF_REPO="deepseek-ai/DeepSeek-OCR"

echo "=== DeepSeek-OCR Model Download Script ==="
echo ""
echo "Target directory: $MODEL_DIR"
echo "HuggingFace repo: $HF_REPO"
echo ""

# Check if huggingface CLI is installed
if ! command -v hf &> /dev/null && ! command -v huggingface-cli &> /dev/null; then
    echo "ERROR: Hugging Face CLI not found!"
    echo ""
    echo "Please install it using one of these methods:"
    echo ""
    echo "  Option 1: Using pip"
    echo "    pip install huggingface_hub"
    echo ""
    echo "  Option 2: Using uv (recommended)"
    echo "    uv pip install huggingface_hub"
    echo ""
    echo "  Option 3: Using pipx (isolated)"
    echo "    pipx install huggingface_hub"
    echo ""
    echo "After installation, the CLI will be available as 'hf'"
    echo ""
    exit 1
fi

# Use 'hf' if available, otherwise 'huggingface-cli' for older versions
if command -v hf &> /dev/null; then
    HF_CLI="hf"
else
    HF_CLI="huggingface-cli"
fi

# Create model directory
mkdir -p "$MODEL_DIR"

echo "Downloading model files (this will take a while, ~6.67 GB total)..."
echo ""

# Download the essential files for agx-ocr
echo "==> Downloading config.json..."
$HF_CLI download "$HF_REPO" config.json --local-dir "$MODEL_DIR"

echo "==> Downloading tokenizer.json..."
$HF_CLI download "$HF_REPO" tokenizer.json --local-dir "$MODEL_DIR"

echo "==> Downloading model weights (model-00001-of-000001.safetensors, ~6.67 GB)..."
echo "    This may take 10-30 minutes depending on your internet connection..."
$HF_CLI download "$HF_REPO" model-00001-of-000001.safetensors --local-dir "$MODEL_DIR"

# Rename the weights file to match agx-ocr's expectation
echo ""
echo "==> Renaming model weights..."
mv "$MODEL_DIR/model-00001-of-000001.safetensors" "$MODEL_DIR/model.safetensors"

echo ""
echo "=== Download Complete! ==="
echo ""
echo "Model files installed to: $MODEL_DIR"
echo ""
echo "Directory contents:"
ls -lh "$MODEL_DIR"
echo ""
echo "Total size:"
du -sh "$MODEL_DIR"
echo ""
echo "You can now test agx-ocr:"
echo "  export MODEL_PATH=$MODEL_DIR"
echo "  cat test-assets/sample-receipt.png | ./target/release/agx-ocr"
echo ""
