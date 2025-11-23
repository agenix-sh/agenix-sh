#!/bin/bash
set -e

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TRAINING_DIR="$ROOT_DIR/training"
CONFIG_PATH="$TRAINING_DIR/axolotl.yaml"

# Set CUDA_HOME if not set, assuming standard install or /usr
if [ -z "$CUDA_HOME" ]; then
    if command -v nvcc &> /dev/null; then
        NVCC_PATH=$(dirname $(dirname $(command -v nvcc)))
        export CUDA_HOME="$NVCC_PATH"
        echo "   Set CUDA_HOME to $CUDA_HOME"
    else
        export CUDA_HOME="/usr"
        echo "   Set CUDA_HOME to /usr (fallback)"
    fi
fi

cd "$ROOT_DIR"

echo "🚀 Setting up Local Training Environment..."

# 1. Check/Install uv
if ! command -v uv &> /dev/null; then
    echo "📦 Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# 2. Create/Activate venv
if [ ! -d ".venv" ]; then
    echo "🐍 Creating virtual environment..."
    uv venv
fi

source .venv/bin/activate

# 3. Install Dependencies
echo "⬇️  Installing dependencies..."
# Install torch first to ensure we get the right cuda version if needed, 
# but uv usually handles this well.
# We need axolotl.
if ! python3 -c "import axolotl" &> /dev/null; then
    echo "   - Installing torch first..."
    uv pip install torch
    
    # flash-attn needs torch present during build, so we use --no-build-isolation
    # Manual clone and install to avoid missing modules
    if [ ! -d "axolotl" ]; then
        git clone https://github.com/OpenAccess-AI-Collective/axolotl.git
    fi
    uv pip install -e axolotl
    uv pip install "axolotl[flash-attn,deepspeed]" # Install extras
    uv pip install accelerate redis
else
    echo "   - Axolotl already installed."
fi

# Ensure numpy and transformers are installed (sometimes missed or overwritten)
uv pip install numpy transformers pandas packaging ninja pyarrow scipy wandb bitsandbytes llvmlite numba

# 4. Run Training
echo "🔥 Starting Training..."
echo "   Config: $CONFIG_PATH"

# Check for GPUs
if command -v nvidia-smi &> /dev/null; then
    NUM_GPUS=$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)
    echo "   Found $NUM_GPUS GPUs."
else
    echo "⚠️  nvidia-smi not found. Assuming CPU or issues."
    NUM_GPUS=0
fi

# Set CUDA_VISIBLE_DEVICES if not set, to use all available
if [ -z "$CUDA_VISIBLE_DEVICES" ]; then
    export CUDA_VISIBLE_DEVICES=$(seq -s, 0 $((NUM_GPUS-1)))
fi

accelerate launch \
    --num_processes=$NUM_GPUS \
    --num_machines=1 \
    --mixed_precision=bf16 \
    -m axolotl.cli.train "$CONFIG_PATH" 2>&1 | tee training.log
