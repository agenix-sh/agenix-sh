#!/usr/bin/env python3
import os
import json
import argparse
import requests
import time

# Configuration
TINKER_API_BASE = os.environ.get("TINKER_API_BASE", "https://api.tinker.ai/v1")
TINKER_API_KEY = os.environ.get("TINKER_API_KEY")

def upload_file(file_path: str, purpose: str = "fine-tune") -> str:
    """Uploads a file to Tinker and returns the file ID."""
    print(f"Uploading {file_path}...")
    headers = {"Authorization": f"Bearer {TINKER_API_KEY}"}
    
    with open(file_path, "rb") as f:
        files = {"file": f}
        data = {"purpose": purpose}
        response = requests.post(f"{TINKER_API_BASE}/files", headers=headers, files=files, data=data)
    
    if response.status_code != 200:
        raise Exception(f"Upload failed: {response.text}")
        
    file_id = response.json()["id"]
    print(f"  File ID: {file_id}")
    return file_id

def create_job(training_file: str, model: str, suffix: str, epochs: int = 3) -> str:
    """Creates a fine-tuning job."""
    print(f"Starting fine-tuning job for {model}...")
    headers = {
        "Authorization": f"Bearer {TINKER_API_KEY}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "training_file": training_file,
        "model": model,
        "suffix": suffix,
        "hyperparameters": {
            "n_epochs": epochs
        }
    }
    
    response = requests.post(f"{TINKER_API_BASE}/fine_tuning/jobs", headers=headers, json=payload)
    
    if response.status_code != 200:
        raise Exception(f"Job creation failed: {response.text}")
        
    job_id = response.json()["id"]
    print(f"  Job ID: {job_id}")
    return job_id

def main():
    parser = argparse.ArgumentParser(description="Submit training jobs to Tinker")
    parser.add_argument("--delta-data", default="research/train_delta.jsonl", help="Delta training data")
    parser.add_argument("--echo-data", default="research/train_echo_chat.jsonl", help="Echo training data")
    parser.add_argument("--base-model", default="meta-llama/Llama-3.1-8B-Instruct", help="Base model ID")
    args = parser.parse_args()

    if not TINKER_API_KEY:
        print("Error: TINKER_API_KEY environment variable not set.")
        return

    # 1. Upload Delta Data
    print("--- Processing Delta (Planner) ---")
    delta_file_id = upload_file(args.delta_data)
    delta_job_id = create_job(delta_file_id, args.base_model, "agenix-delta-v1")
    print(f"Delta Job Started: {delta_job_id}")

    # 2. Upload Echo Data
    print("\n--- Processing Echo (Chat) ---")
    echo_file_id = upload_file(args.echo_data)
    echo_job_id = create_job(echo_file_id, args.base_model, "agenix-echo-v1")
    print(f"Echo Job Started: {echo_job_id}")

    print("\nJobs submitted successfully! Check your Tinker dashboard for progress.")

if __name__ == "__main__":
    main()
