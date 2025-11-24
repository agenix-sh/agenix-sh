#!/usr/bin/env python3
import os
import sys
import argparse
import shutil
import subprocess
import glob
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

# Ensure API Key
if "TINKER_API_KEY" not in os.environ:
    print("Error: TINKER_API_KEY environment variable not set.")
    sys.exit(1)

def download_adapter(tinker_path, output_parent):
    """Downloads the adapter weights using Tinker CLI."""
    print(f"Downloading adapter from {tinker_path}...")
    
    # Ensure parent dir exists
    os.makedirs(output_parent, exist_ok=True)
    
    # Determine path to tinker executable
    # It should be in the same directory as the python executable
    python_dir = os.path.dirname(sys.executable)
    tinker_exe = os.path.join(python_dir, "tinker")
    
    if not os.path.exists(tinker_exe):
        # Fallback to just "tinker" in path
        tinker_exe = "tinker"

    # Use CLI
    cmd = [
        tinker_exe, "checkpoint", "download", 
        tinker_path, 
        "--output", output_parent,
        "--force"
    ]
    
    try:
        subprocess.check_call(cmd)
    except subprocess.CalledProcessError:
        print("Failed to download checkpoint via Tinker CLI.")
        sys.exit(1)
        
    # Find the created subdirectory
    # The CLI creates a directory named after the checkpoint ID/path
    # We just look for the only subdirectory in output_parent
    subdirs = [d for d in os.listdir(output_parent) if os.path.isdir(os.path.join(output_parent, d))]
    if not subdirs:
        print(f"Error: No directory found in {output_parent} after download.")
        sys.exit(1)
        
    # Return the full path to the adapter
    return os.path.join(output_parent, subdirs[0])

def merge_model(base_model_id, adapter_dir, output_dir):
    """Merges the LoRA adapter with the base model."""
    print(f"Loading base model: {base_model_id}...")
    try:
        # Load Base Model
        base_model = AutoModelForCausalLM.from_pretrained(
            base_model_id,
            torch_dtype=torch.float16,
            device_map="auto",
            trust_remote_code=True
        )
        tokenizer = AutoTokenizer.from_pretrained(base_model_id)
        
        print(f"Loading adapter from {adapter_dir}...")
        # Load Adapter
        model = PeftModel.from_pretrained(base_model, adapter_dir)
        
        print("Merging weights...")
        model = model.merge_and_unload()
        
        print(f"Saving merged model to {output_dir}...")
        model.save_pretrained(output_dir)
        tokenizer.save_pretrained(output_dir)
        print("Merge complete.")
        
    except Exception as e:
        print(f"Error during merge: {e}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Prepare local GGUF model from Tinker")
    parser.add_argument("--tinker-path", required=True, help="Tinker model path (tinker://...)")
    parser.add_argument("--base-model", default="meta-llama/Llama-3.1-8B-Instruct", help="Base HF model ID")
    parser.add_argument("--output-dir", default="models/agenix-echo-v1-merged", help="Output directory for merged model")
    args = parser.parse_args()

    # 1. Download Adapter
    temp_dl_dir = "models/temp_dl"
    # Clean temp dir first
    if os.path.exists(temp_dl_dir):
        shutil.rmtree(temp_dl_dir)
        
    adapter_dir = download_adapter(args.tinker_path, temp_dl_dir)
    print(f"Adapter downloaded to: {adapter_dir}")

    # 2. Merge
    merge_model(args.base_model, adapter_dir, args.output_dir)

    # 3. Cleanup
    print("Cleaning up temp files...")
    shutil.rmtree(temp_dl_dir)

    # 4. Instructions for GGUF Conversion
    print("\n=== Ready for GGUF Conversion ===")
    print(f"Merged model saved to: {args.output_dir}")
    print("To convert to GGUF, run the following command (requires Docker):")
    print(f"\ndocker run --rm -v {os.getcwd()}:/app ghcr.io/ggerganov/llama.cpp:full-cuda \\")
    print(f"  python3 convert_hf_to_gguf.py /app/{args.output_dir} --outfile /app/models/agenix-echo-v1.gguf --outtype q8_0")
    print("\nOr if you have llama.cpp installed locally:")
    print(f"python3 /path/to/llama.cpp/convert_hf_to_gguf.py {args.output_dir} --outfile models/agenix-echo-v1.gguf --outtype q8_0")

if __name__ == "__main__":
    main()
