#!/bin/bash
set -e

# Set CUDA_HOME if not set
if [ -z "$CUDA_HOME" ]; then
    if command -v nvcc &> /dev/null; then
        export CUDA_HOME=$(dirname $(dirname $(command -v nvcc)))
    else
        export CUDA_HOME=/usr
    fi
fi

export CUDA_VISIBLE_DEVICES=0,1

# Activate virtual environment
source .venv/bin/activate

# Run inference
# We use the same config but override lora_model_dir to point to our trained adapter
accelerate launch --num_processes=1 -m axolotl.cli.inference training/axolotl.yaml \
    --lora_model_dir="./qlora-out" \
    --load_in_4bit=true \
    --flash_attention=false
