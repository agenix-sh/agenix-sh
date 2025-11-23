#!/bin/bash
set -e

echo "🚀 Setting up Agenix on DGX..."

# 1. Check Prerequisites
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install via rustup.rs"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 not found."
    exit 1
fi

# 2. Build Binaries
echo "📦 Building Binaries..."

# Build AGQ (Queue) and AGW (Worker) - CPU is fine for queue, Worker needs to be robust
echo "   - Building AGQ & AGW..."
cargo build --release --bin agq --bin agw

# Build AGX (Planner/Tools) with CUDA support for local inference if needed
# Note: For training, we use 'accelerate' which uses python/torch, but agx might use candle
echo "   - Building AGX (with CUDA)..."
export CUDA_COMPUTE_CAP=90  # Hopper (H100) / Blackwell
cargo build --release --bin agx --features cuda

# Build Data Generator
echo "   - Building Data Generator..."
cargo build --release --bin generate_data --features cuda

# Build Training Wrapper
echo "   - Building Training Wrapper..."
cargo build --release --bin agx_train

echo "✅ Build Complete."

# 3. Python Environment
echo "🐍 Checking Python Dependencies..."
if ! python3 -c "import axolotl" &> /dev/null; then
    echo "⚠️  Axolotl not found. It is REQUIRED for training."
    echo "   Please install it: pip install -e '.[flash-attn,deepspeed]'"
    echo "   See: https://github.com/OpenAccess-AI-Collective/axolotl"
else
    echo "   - Axolotl found."
fi

if ! python3 -c "import accelerate" &> /dev/null; then
    echo "⚠️  Accelerate not found. Installing..."
    pip install accelerate
fi

if ! python3 -c "import redis" &> /dev/null; then
    echo "⚠️  Redis-py not found. Installing..."
    pip install redis
fi

# 4. Directories
mkdir -p training/experiments
mkdir -p training/qlora-out

# 5. Instructions
echo ""
echo "🎉 Setup Complete!"
echo ""
echo "To start the cluster:"
echo "1. Start Queue (in a tmux/screen):"
echo "   export AGQ_SESSION_KEY=\$(openssl rand -hex 32)"
echo "   ./target/release/agq --session-key \$AGQ_SESSION_KEY"
echo ""
echo "2. Start Worker (in another tmux/screen):"
echo "   export AGQ_SESSION_KEY=..."
echo "   ./target/release/agw --tags gpu --session-key \$AGQ_SESSION_KEY"
echo ""
echo "3. Run Experiments:"
echo "   python3 experiments/run_experiments.py"
echo ""
