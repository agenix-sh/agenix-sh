# Model Setup for agx-ocr

## DeepSeek-OCR Model Requirements

The `agx-ocr` AU requires the DeepSeek-OCR model files in a specific directory structure. The `--model-path` argument should point to a directory containing:

```
<model-directory>/
├── config.json          # Model configuration
├── tokenizer.json       # Tokenizer configuration
└── model.safetensors    # Model weights (~6.3 GB FP16)
    OR
└── model.gguf          # Quantized model weights (if using GGUF)
```

## Quick Setup Using deepseek-ocr-cli

The easiest way to download the model is to use the `deepseek-ocr-cli` tool from the `deepseek-ocr.rs` repository:

### Step 1: Build deepseek-ocr-cli

```bash
cd /Users/lewis/work/deepseek-ocr.rs
cargo build --release -p deepseek-ocr-cli
```

### Step 2: Run once to auto-download models

```bash
# Create a test image first
echo "Sample text for OCR" | convert -size 400x100 -background white -fill black -font Arial -pointsize 24 label:@- /tmp/test.png

# Run the CLI (this will auto-download the model to the cache)
./target/release/deepseek-ocr-cli /tmp/test.png
```

The model will be downloaded to the platform-specific cache directory:

| Platform | Model cache location |
|----------|---------------------|
| macOS    | `~/Library/Caches/deepseek-ocr/models/deepseek-ocr/` |
| Linux    | `~/.cache/deepseek-ocr/models/deepseek-ocr/` |
| Windows  | `%LOCALAPPDATA%\deepseek-ocr\models\deepseek-ocr\` |

### Step 3: Symlink or copy to a known location

```bash
# For macOS
mkdir -p ~/models/deepseek-ocr
ln -s ~/Library/Caches/deepseek-ocr/models/deepseek-ocr/* ~/models/deepseek-ocr/

# Or copy the files
cp ~/Library/Caches/deepseek-ocr/models/deepseek-ocr/* ~/models/deepseek-ocr/
```

## Direct Download Using HuggingFace CLI (Recommended)

### Step 1: Install Hugging Face CLI

```bash
# Option 1: Using pip
pip install huggingface_hub

# Option 2: Using uv (recommended)
uv pip install huggingface_hub

# Option 3: Using pipx (isolated environment)
pipx install huggingface_hub
```

This installs the `hf` command-line tool for downloading models from HuggingFace.

### Step 2: Run the download script

We provide a convenient script that downloads all required files:

```bash
# Download to default location (~/models/deepseek-ocr)
./download-model.sh

# Or specify a custom directory
./download-model.sh /path/to/custom/location
```

### Step 3: Manual download (if you prefer)

```bash
# Set target directory
MODEL_DIR=~/models/deepseek-ocr
mkdir -p "$MODEL_DIR"

# Download required files using 'hf' CLI
hf download deepseek-ai/DeepSeek-OCR config.json --local-dir "$MODEL_DIR"
hf download deepseek-ai/DeepSeek-OCR tokenizer.json --local-dir "$MODEL_DIR"
hf download deepseek-ai/DeepSeek-OCR model-00001-of-000001.safetensors --local-dir "$MODEL_DIR"

# Rename the weights file to match agx-ocr's expectation
mv "$MODEL_DIR/model-00001-of-000001.safetensors" "$MODEL_DIR/model.safetensors"
```

**Files downloaded:**
- `config.json` (~2.67 KB) - Model configuration
- `tokenizer.json` (~9.98 MB) - Tokenizer configuration
- `model-00001-of-000001.safetensors` (~6.67 GB) - Model weights (renamed to `model.safetensors`)

**Total download size:** ~6.68 GB

## Using agx-ocr with the Model

Once you have the model directory set up, you can use `agx-ocr`:

```bash
# Example: OCR on a PNG image
cat image.png | ./target/debug/agx-ocr --model-path ~/models/deepseek-ocr

# Or using the MODEL_PATH environment variable
export MODEL_PATH=~/models/deepseek-ocr
cat image.png | ./target/debug/agx-ocr
```

## Hardware Requirements

- **RAM/VRAM**: ~13 GB minimum (6.3 GB model + 6-7 GB activations during inference)
- **Disk Space**: ~6.5 GB for FP16 model weights
- **GPU**: Optional but recommended
  - Apple Silicon (Metal): Fastest on macOS
  - NVIDIA (CUDA): Experimental support
  - CPU: Supported but slower

## Model Specifications

- **Type**: DeepSeek-OCR (Vision + Language model)
- **Architecture**: SAM+CLIP vision tower + DeepSeek-V2 MoE decoder
- **Parameters**: ~3B total (~570M active per token)
- **Precision**: FP16 (Metal/CUDA) or BF16 (CPU)
- **Memory**: ~6.3 GB weights + ~6-7 GB runtime
