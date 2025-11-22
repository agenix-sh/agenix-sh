# DGX Spark Build Guide

This guide covers building `agx-ocr` for NVIDIA DGX Spark (ARM64 + Ubuntu 24.04).

## Two Build Approaches

### Option A: Native Build on DGX Spark (Recommended)
Build directly on the device - simpler and avoids cross-compilation issues.

### Option B: Cross-Compile from macOS
Build on your Mac and copy the binary to DGX Spark.

---

## Option A: Native Build on DGX Spark

### Prerequisites

SSH into your DGX Spark:
```bash
ssh user@dgx-spark-hostname
```

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Build Dependencies

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config git
```

### 3. Clone and Build agx-ocr

```bash
# Clone the repository
cd ~
git clone <your-agx-ocr-repo-url>
cd agx-ocr

# Build release binary (CPU-only)
cargo build --release

# Verify build
ls -lh target/release/agx-ocr
file target/release/agx-ocr
# Should show: ELF 64-bit LSB executable, ARM aarch64
```

### 4. Download Model

```bash
# Install Python and huggingface_hub
sudo apt-get install -y python3-pip
pip3 install huggingface_hub

# Download model
./download-model.sh ~/models/deepseek-ocr
```

### 5. Test

```bash
# Test --describe
./target/release/agx-ocr --describe

# Test OCR
cat test-assets/sample-receipt.png | \
  ./target/release/agx-ocr --model-path ~/models/deepseek-ocr
```

### Expected Performance on DGX Spark (CPU-only)

- **First run**: ~30-60 seconds (model loading + inference)
- **Subsequent runs**: ~30-60 seconds per image
- **Memory usage**: ~13 GB
- **Precision**: BF16

---

## Option B: Cross-Compile from macOS

### Prerequisites on macOS

Install the ARM64 Linux target:
```bash
rustup target add aarch64-unknown-linux-gnu
```

### Method 1: Using `cross` (Easiest)

Install cross-compilation tool:
```bash
cargo install cross
```

Build:
```bash
cross build --release --target aarch64-unknown-linux-gnu
```

The binary will be at: `target/aarch64-unknown-linux-gnu/release/agx-ocr`

### Method 2: Using Docker

Create a Dockerfile for cross-compilation:

```dockerfile
FROM rust:1.91

# Install ARM64 cross-compilation tools
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu

# Add ARM64 target
RUN rustup target add aarch64-unknown-linux-gnu

WORKDIR /build
COPY . .

# Build
RUN cargo build --release --target aarch64-unknown-linux-gnu
```

Build:
```bash
docker build -t agx-ocr-builder .
docker run --rm -v $(pwd)/target:/build/target agx-ocr-builder
```

### Transfer Binary to DGX Spark

```bash
# Copy binary
scp target/aarch64-unknown-linux-gnu/release/agx-ocr \
  user@dgx-spark:~/agx-ocr

# Copy test assets
scp -r test-assets user@dgx-spark:~/agx-ocr/

# SSH and test
ssh user@dgx-spark
cd agx-ocr
./agx-ocr --describe
```

---

## Cargo Configuration for Cross-Compilation

Create `.cargo/config.toml` in the agx-ocr directory:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[build]
# Uncomment to make ARM64 Linux the default target
# target = "aarch64-unknown-linux-gnu"

[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip symbols for smaller binary
```

---

## Dockerfile for DGX Spark Deployment

Create `Dockerfile.dgx`:

```dockerfile
# Minimal container for agx-ocr on DGX Spark
FROM ubuntu:24.04

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create model directory
RUN mkdir -p /models/deepseek-ocr

# Copy the pre-built ARM64 binary
COPY target/aarch64-unknown-linux-gnu/release/agx-ocr /usr/local/bin/agx-ocr
RUN chmod +x /usr/local/bin/agx-ocr

# Set model path environment variable
ENV MODEL_PATH=/models/deepseek-ocr

# Verify binary works
RUN agx-ocr --describe

# Default: read from stdin, write JSON to stdout
ENTRYPOINT ["agx-ocr"]
```

Build and test:
```bash
# Build container
docker build -f Dockerfile.dgx -t agx-ocr:dgx .

# Test
docker run --rm agx-ocr:dgx --describe

# Run OCR
cat test-assets/sample-receipt.png | \
  docker run --rm -i \
  -v ~/models/deepseek-ocr:/models/deepseek-ocr:ro \
  agx-ocr:dgx
```

---

## Build Variants

### CPU-Only (Default - Recommended)

```bash
cargo build --release
```

**Features:**
- ✅ Stable and reliable
- ✅ No GPU dependencies
- ✅ Works on all ARM64 Linux systems
- ⚠️ Slower inference (~30-60s per image)

### CUDA-Enabled (Experimental)

⚠️ **Warning**: Candle CUDA support is alpha. Blackwell sm_121 may not be supported.

```bash
# Install CUDA 13.0 toolkit first
sudo apt-get install nvidia-cuda-toolkit

# Build with CUDA
cargo build --release --features cuda
```

**Features:**
- ⚡ Faster inference (~2-5s per image) - if it works
- ⚠️ May not work with Blackwell GPU
- ⚠️ Alpha quality, expect issues

---

## Verification Steps

### 1. Check Binary Architecture

```bash
file target/release/agx-ocr
# Expected: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), dynamically linked
```

### 2. Check Dependencies

```bash
ldd target/release/agx-ocr
# Should show standard libc, libm, etc. - no CUDA libraries
```

### 3. Test AU Contract

```bash
# Model card
./target/release/agx-ocr --describe | jq .

# OCR test
cat test-assets/sample-receipt.png | \
  ./target/release/agx-ocr --model-path ~/models/deepseek-ocr | \
  jq .
```

### 4. Performance Benchmark

```bash
# Time a single inference
time (cat test-assets/sample-receipt.png | \
  ./target/release/agx-ocr --model-path ~/models/deepseek-ocr > /dev/null)

# Expected on DGX Spark CPU: ~30-60 seconds
```

---

## Troubleshooting

### Issue: "cannot execute binary file"

**Problem**: Built for wrong architecture

**Solution**:
```bash
# Check what you built
file target/release/agx-ocr

# Should be ARM aarch64, not x86_64
# If wrong, rebuild with correct target
```

### Issue: "Model path not found"

**Problem**: Model not downloaded

**Solution**:
```bash
# Download model
./download-model.sh ~/models/deepseek-ocr

# Verify files exist
ls -lh ~/models/deepseek-ocr/
# Should have: config.json, tokenizer.json, model.safetensors
```

### Issue: "Out of memory"

**Problem**: Not enough RAM

**Solution**:
```bash
# Check available memory
free -h

# DGX Spark has 128 GB - should be plenty
# If running in Docker, increase memory limit
docker run --memory=16g ...
```

### Issue: Slow build times

**Problem**: Building in debug mode or with dependencies

**Solution**:
```bash
# Always use --release
cargo build --release

# Clean and rebuild
cargo clean
cargo build --release
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build for DGX Spark

on:
  push:
    branches: [main]

jobs:
  build-arm64:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install cross
        run: cargo install cross

      - name: Build ARM64 binary
        run: cross build --release --target aarch64-unknown-linux-gnu

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: agx-ocr-arm64-linux
          path: target/aarch64-unknown-linux-gnu/release/agx-ocr
```

---

## Next Steps

After successful build on DGX Spark:

1. **Benchmark**: Measure actual inference times
2. **Integration**: Connect to AGEniX orchestrator
3. **Monitoring**: Add logging and metrics
4. **GPU Support**: Track Candle CUDA Blackwell support
5. **Optimization**: Profile and optimize hot paths

## Summary

**Recommended Path**:
1. ✅ Build natively on DGX Spark (simplest)
2. ✅ Use CPU-only mode (stable)
3. ⏳ Wait for Candle CUDA + Blackwell support (future)

The CPU-only build should work immediately and provide a solid foundation for the AGEniX integration on DGX Spark.
