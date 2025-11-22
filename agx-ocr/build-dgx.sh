#!/bin/bash
# Build script for NVIDIA DGX Spark (ARM64 Linux)
#
# This script handles both cross-compilation and native builds

set -e

BUILD_METHOD="${1:-cross}"  # cross, native, or docker

echo "=== agx-ocr DGX Spark Build Script ==="
echo ""
echo "Build method: $BUILD_METHOD"
echo ""

case "$BUILD_METHOD" in
  cross)
    echo "Building using 'cross' tool (cross-compilation)..."
    echo ""

    # Check if cross is installed
    if ! command -v cross &> /dev/null; then
      echo "Installing 'cross' tool..."
      cargo install cross
    fi

    # Add ARM64 Linux target if not present
    rustup target add aarch64-unknown-linux-gnu || true

    # Build
    echo "Building for aarch64-unknown-linux-gnu..."
    cross build --release --target aarch64-unknown-linux-gnu

    # Verify
    echo ""
    echo "Build complete!"
    ls -lh target/aarch64-unknown-linux-gnu/release/agx-ocr
    file target/aarch64-unknown-linux-gnu/release/agx-ocr
    ;;

  native)
    echo "Building natively (assumes you're on ARM64 Linux)..."
    echo ""

    # Verify architecture
    ARCH=$(uname -m)
    if [ "$ARCH" != "aarch64" ]; then
      echo "Warning: You're not on ARM64 architecture (detected: $ARCH)"
      echo "This build may not work on DGX Spark."
      read -p "Continue anyway? [y/N] " -n 1 -r
      echo
      if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
      fi
    fi

    # Build
    echo "Building release binary..."
    cargo build --release

    # Verify
    echo ""
    echo "Build complete!"
    ls -lh target/release/agx-ocr
    file target/release/agx-ocr
    ;;

  docker)
    echo "Building using Docker (cross-compilation in container)..."
    echo ""

    # Create temporary Dockerfile for building
    cat > Dockerfile.build <<'EOF'
FROM rust:1.91

# Install ARM64 cross-compilation tools
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Add ARM64 target
RUN rustup target add aarch64-unknown-linux-gnu

# Set up cross-compilation environment
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

WORKDIR /build
COPY . .

# Build
RUN cargo build --release --target aarch64-unknown-linux-gnu

# Output for extraction
CMD ["sh", "-c", "cp target/aarch64-unknown-linux-gnu/release/agx-ocr /output/"]
EOF

    # Build the builder image
    echo "Building Docker builder image..."
    docker build -f Dockerfile.build -t agx-ocr-builder .

    # Run builder and extract binary
    echo "Running build in container..."
    mkdir -p target/docker-output
    docker run --rm \
      -v "$(pwd)/target/docker-output:/output" \
      agx-ocr-builder

    # Move to standard location
    mkdir -p target/aarch64-unknown-linux-gnu/release
    mv target/docker-output/agx-ocr target/aarch64-unknown-linux-gnu/release/

    # Clean up
    rm Dockerfile.build
    rm -rf target/docker-output

    # Verify
    echo ""
    echo "Build complete!"
    ls -lh target/aarch64-unknown-linux-gnu/release/agx-ocr
    file target/aarch64-unknown-linux-gnu/release/agx-ocr
    ;;

  *)
    echo "Unknown build method: $BUILD_METHOD"
    echo ""
    echo "Usage: $0 [cross|native|docker]"
    echo ""
    echo "  cross  - Use 'cross' tool for cross-compilation (default)"
    echo "  native - Build natively (run on DGX Spark)"
    echo "  docker - Build in Docker container"
    echo ""
    exit 1
    ;;
esac

echo ""
echo "=== Build Summary ==="
echo ""
echo "Binary location:"
if [ "$BUILD_METHOD" = "native" ]; then
  echo "  target/release/agx-ocr"
else
  echo "  target/aarch64-unknown-linux-gnu/release/agx-ocr"
fi
echo ""
echo "Next steps:"
echo "  1. Copy binary to DGX Spark:"
echo "     scp target/aarch64-unknown-linux-gnu/release/agx-ocr user@dgx-spark:~/"
echo ""
echo "  2. Test on DGX Spark:"
echo "     ssh user@dgx-spark"
echo "     ./agx-ocr --describe"
echo ""
echo "  3. Or build Docker container:"
echo "     docker build -f Dockerfile.dgx -t agx-ocr:dgx ."
echo ""
