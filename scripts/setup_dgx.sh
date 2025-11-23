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

# Check for OpenSSL/pkg-config (required for some crates even with rustls)
if ! command -v pkg-config &> /dev/null; then
    echo "⚠️  pkg-config not found. Attempting to install..."
    if command -v sudo &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y pkg-config libssl-dev
    else
        echo "❌ sudo not found. Please install 'pkg-config' and 'libssl-dev' manually."
        exit 1
    fi
fi

if ! pkg-config --exists openssl; then
    echo "⚠️  OpenSSL dev headers not found. Attempting to install..."
    if command -v sudo &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y libssl-dev
    else
        echo "❌ sudo not found. Please install 'libssl-dev' manually."
        exit 1
    fi
fi

# 2. Build Binaries
echo "📦 Building Binaries..."

# Build AGQ (Queue) and AGW (Worker)
echo "   - Building AGQ & AGW..."
cargo build --release --bin agq --bin agw

# Build Training Wrapper (The only thing needed for training besides python env)
echo "   - Building Training Wrapper..."
cargo build --release --bin agx_train --no-default-features

# Build AGX (Optional, CPU only, no default features to avoid 'accelerate' framework issue on Linux)
echo "   - Building AGX (Minimal)..."
cargo build --release --bin agx --no-default-features

# Note: We skip generate_data and full agx build to avoid CUDA/Accelerate dependency hell on DGX.
# The user can run planning/generation on their laptop.

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
