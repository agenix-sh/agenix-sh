#!/bin/bash
# Test script for agx-ocr
# Usage: ./test.sh [model-path]

set -e

MODEL_PATH="${1:-$HOME/models/deepseek-ocr}"

echo "=== agx-ocr Test Script ==="
echo ""

# Check if model path exists
if [ ! -d "$MODEL_PATH" ]; then
    echo "ERROR: Model directory not found at: $MODEL_PATH"
    echo ""
    echo "Please download the model first. See docs/MODEL_SETUP.md for instructions."
    echo ""
    echo "Quick setup:"
    echo "  1. Build deepseek-ocr-cli:"
    echo "     cd ../deepseek-ocr.rs && cargo build --release -p deepseek-ocr-cli"
    echo ""
    echo "  2. Run once to download model:"
    echo "     ../deepseek-ocr.rs/target/release/deepseek-ocr-cli --help"
    echo ""
    echo "  3. Create symlink or copy model:"
    echo "     mkdir -p ~/models/deepseek-ocr"
    echo "     ln -s ~/Library/Caches/deepseek-ocr/models/deepseek-ocr/* ~/models/deepseek-ocr/"
    echo ""
    exit 1
fi

# Check required model files
echo "Checking model files in: $MODEL_PATH"
for file in config.json tokenizer.json; do
    if [ ! -f "$MODEL_PATH/$file" ]; then
        echo "  ✗ Missing: $file"
        exit 1
    else
        echo "  ✓ Found: $file"
    fi
done

# Check for weights file
if [ -f "$MODEL_PATH/model.safetensors" ]; then
    echo "  ✓ Found: model.safetensors"
elif [ -f "$MODEL_PATH/model.gguf" ]; then
    echo "  ✓ Found: model.gguf"
else
    echo "  ✗ Missing: model weights (expected model.safetensors or model.gguf)"
    exit 1
fi

echo ""

# Test 1: --describe flag
echo "Test 1: Testing --describe flag"
./target/debug/agx-ocr --describe | jq .
echo "  ✓ Test 1 passed"
echo ""

# Test 2: OCR on sample image
if [ -f "test-assets/sample-receipt.png" ]; then
    echo "Test 2: Running OCR on sample-receipt.png"
    echo "  (This may take 30-60 seconds on first run as model loads into memory)"
    echo ""

    cat test-assets/sample-receipt.png | \
        ./target/debug/agx-ocr --model-path "$MODEL_PATH" | \
        jq '.'

    echo ""
    echo "  ✓ Test 2 passed"
else
    echo "Test 2: SKIPPED (no test image found)"
fi

echo ""
echo "=== All tests completed successfully! ==="
