# NVIDIA DGX Spark Build Target

## System Specifications

### Hardware
- **Superchip**: NVIDIA GB10 Grace Blackwell Superchip
- **CPU**: 20-core ARM64 (ARMv9.2)
  - 2 clusters × 5 Cortex-X925 (high-performance) + 5 Cortex-A725 (high-efficiency)
- **GPU**: Integrated Blackwell GPU
  - Compute Capability: **sm_121** (Blackwell architecture)
  - 5th generation Tensor Cores
  - 4th generation RT Cores
- **Memory**: 128 GB LPDDR5x unified memory (UMA)
  - Bandwidth: 273 GB/s
  - Shared between CPU and GPU
- **Performance**: 1 petaFLOP AI at FP4

### Software
- **OS**: DGX OS (Ubuntu 24.04 LTS)
- **CUDA**: CUDA 13.0.1
- **Architecture**: `aarch64-unknown-linux-gnu`

## Challenge: Candle + CUDA + Blackwell

### Current Status

**⚠️ Candle CUDA Support is Alpha**

According to deepseek-ocr.rs README:
> **CUDA (alpha)** – experimental support via `--features cuda` + `--device cuda --dtype f16`; expect rough edges while we finish kernel coverage.

**⚠️ Blackwell Compatibility Issues**

Research findings:
1. **Compute Capability**: Blackwell uses sm_120/sm_121
2. **CUDA Requirement**: CUDA 12.8+ for sm_120, CUDA 12.9+ for sm_121
3. **Candle Issues**: Known compatibility problems with Blackwell GPUs
   - Error: "Runtime compute cap 120 is not compatible with compile time compute cap 120"
   - Requires recompilation with proper PTX support
4. **Rebuild Required**: Binaries with only cubins need to be rebuilt for Blackwell

### Build Options for DGX Spark

Given the challenges, we have **three approaches**:

## Option 1: CPU-Only Build (Safest) ✅

Use the ARM64 CPU with BF16 precision, bypassing GPU complications.

**Pros:**
- ✅ No CUDA compatibility issues
- ✅ Works immediately
- ✅ Unified memory architecture benefits CPU
- ✅ 128 GB memory available
- ✅ 20 ARM cores can parallelize well

**Cons:**
- ❌ Slower inference (~30-60s per image vs ~2-5s)
- ❌ Doesn't utilize GPU Tensor Cores

**Build Command:**
```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

**Runtime:**
```rust
let device = Device::Cpu;
let dtype = DType::BF16;
```

## Option 2: CUDA Build (Experimental) ⚠️

Attempt to use Candle's CUDA backend with Blackwell GPU.

**Pros:**
- ✅ If it works, significantly faster inference
- ✅ Utilizes Tensor Cores
- ✅ Better memory efficiency with FP16

**Cons:**
- ❌ Candle CUDA is alpha quality
- ❌ Blackwell sm_121 may not be supported yet
- ❌ Requires CUDA 13.0 compatibility
- ❌ May need custom Candle build
- ❌ High risk of runtime failures

**Build Command:**
```bash
# Requires CUDA 13.0 toolkit installed
cargo build --release \
  --target aarch64-unknown-linux-gnu \
  --features cuda
```

**Potential Issues:**
- Candle may not support sm_121 compute capability
- cubin/PTX compatibility with Blackwell
- Flash-attention kernel compatibility

## Option 3: Python Implementation (Proven) ✅

Use the existing Python-based Docker container from `../aoa-talk/agents/ocr`.

**Pros:**
- ✅ Already working on DGX Spark
- ✅ PyTorch has mature CUDA 13 + Blackwell support
- ✅ Flash-attention 2.5.6 tested and working
- ✅ Production-ready

**Cons:**
- ❌ Not a Rust binary (doesn't align with agx-ocr goals)
- ❌ Larger memory footprint
- ❌ Slower cold start

## Recommended Approach

### Phase 1: CPU-Only Build (Immediate)

Start with a **CPU-only ARM64 build** to validate the AU contract on DGX Spark:

```bash
# Cross-compile from macOS
cargo build --release --target aarch64-unknown-linux-gnu

# Or build natively on DGX Spark
cargo build --release
```

This gives us:
1. ✅ Working binary immediately
2. ✅ Validates AU integration
3. ✅ Establishes baseline performance
4. ✅ No CUDA/Blackwell complications

### Phase 2: Monitor Candle CUDA Progress

Track these upstream issues:
- Candle CUDA maturity status
- Blackwell sm_121 support in Candle
- DeepSeek-OCR.rs CUDA backend stability

### Phase 3: CUDA When Ready

Once Candle CUDA + Blackwell is stable:
1. Update dependencies
2. Enable CUDA feature flag
3. Test on DGX Spark
4. Benchmark vs CPU-only

## Build Configuration

### Cross-Compilation Setup (from macOS)

Create `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[build]
target = "aarch64-unknown-linux-gnu"
```

Install cross-compilation tools:
```bash
brew install FiloSottile/musl-cross/musl-cross

# Or use cross tool
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

### Native Build on DGX Spark

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build agx-ocr
cd agx-ocr
cargo build --release

# Test
cat test-assets/sample-receipt.png | \
  ./target/release/agx-ocr --model-path ~/models/deepseek-ocr
```

## Performance Expectations

### CPU-Only (BF16)
- **Inference**: ~30-60 seconds per image
- **Memory**: ~13 GB (model + activations)
- **Cores**: 20 ARM cores utilized
- **Precision**: BF16

### CUDA (FP16) - If/When Supported
- **Inference**: ~2-5 seconds per image
- **Memory**: ~13 GB VRAM (unified)
- **Tensor Cores**: Utilized
- **Precision**: FP16

## Docker Container Approach

If we want to containerize the Rust binary for DGX Spark:

```dockerfile
FROM ubuntu:24.04

# Install minimal dependencies
RUN apt-get update && apt-get install -y ca-certificates

# Copy the pre-built ARM64 binary
COPY target/aarch64-unknown-linux-gnu/release/agx-ocr /usr/local/bin/

# Copy model files
COPY models/deepseek-ocr /models/deepseek-ocr

ENV MODEL_PATH=/models/deepseek-ocr

# Test
RUN agx-ocr --describe

# Runtime
ENTRYPOINT ["agx-ocr"]
```

## Comparison with Existing Python Solution

| Aspect | agx-ocr (Rust + CPU) | aoa-talk Python |
|--------|---------------------|-----------------|
| **Language** | Rust | Python |
| **GPU Support** | ❌ Not yet (Candle alpha) | ✅ CUDA 13 + Blackwell |
| **Inference Speed** | ~30-60s (CPU) | ~2-5s (GPU) |
| **Memory** | ~13 GB | ~13 GB |
| **Binary Size** | ~7 MB | ~2+ GB (Docker) |
| **Cold Start** | Fast (~5s) | Slow (~30s) |
| **AU Compliance** | ✅ Full | ⚠️ Needs wrapper |
| **Stability** | ✅ High (CPU) | ✅ High |

## Decision Matrix

### Use Rust CPU-Only Build If:
- ✅ AU contract compliance is critical
- ✅ Small binary size is important
- ✅ Inference latency of 30-60s is acceptable
- ✅ Want to avoid Python dependencies

### Use Python GPU Build If:
- ✅ Need fastest inference (<5s per image)
- ✅ Already have working Docker container
- ✅ CUDA + Blackwell support is essential
- ✅ Can wrap Python in AU interface

### Wait for Candle CUDA If:
- ✅ Need both Rust and GPU acceleration
- ✅ Can wait for upstream support
- ✅ Want long-term Rust-native solution

## Action Plan

### Immediate (Week 1)
1. ✅ Build CPU-only ARM64 binary
2. ✅ Test on DGX Spark (if available)
3. ✅ Benchmark inference performance
4. ✅ Document limitations

### Short-term (Month 1)
1. Monitor Candle CUDA Blackwell support
2. Test CUDA build when Candle updates
3. Create Docker container for CPU build
4. Add GPU device detection

### Long-term (Quarter 1)
1. Migrate to CUDA when stable
2. Benchmark CUDA vs CPU
3. Add automatic fallback (GPU → CPU)
4. Optimize for DGX Spark architecture

## Key Foibles & Gotchas

### 1. Unified Memory Architecture
- CPU and GPU share 128 GB
- No separate VRAM allocation
- Memory pressure affects both

### 2. Blackwell sm_121
- Very new architecture (2025)
- Limited toolchain support
- Candle may not support yet

### 3. ARM64 Cross-Compilation
- Need ARM64 toolchain on macOS
- Or build natively on DGX Spark
- Static linking recommended

### 4. CUDA 13.0
- Newer than most frameworks expect
- Compatibility issues common
- Fallback to CUDA 12.x may be needed

### 5. Flash-Attention
- Critical for performance
- May not work with Candle CUDA
- CPU fallback available

## Conclusion

**Recommended**: Start with **CPU-only ARM64 build** for DGX Spark.

This gives us a working AU binary immediately while avoiding the experimental Candle CUDA + Blackwell complications. Once Candle's CUDA support matures and Blackwell compatibility is confirmed, we can add GPU acceleration as an enhancement.

The 20-core ARM CPU with 128 GB unified memory should provide reasonable performance for OCR workloads, even without GPU acceleration.
