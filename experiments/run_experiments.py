import os
import json
import uuid
import subprocess
import argparse
import sys
import time

# Configuration
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BASE_CONFIG_PATH = os.path.join(SCRIPT_DIR, "../training/axolotl.yaml")
EXPERIMENTS_DIR = os.path.join(SCRIPT_DIR, "../training/experiments")
REDIS_HOST = "localhost"
REDIS_PORT = 6379

class RedisClient:
    def __init__(self, host, port):
        self.host = host
        self.port = str(port)

    def run_cmd(self, *args):
        cmd = ["redis-cli", "-h", self.host, "-p", self.port] + list(args)
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise Exception(f"Redis command failed: {result.stderr}")
        return result.stdout.strip()

    def set(self, key, value):
        return self.run_cmd("SET", key, value)

    def lpush(self, key, value):
        return self.run_cmd("LPUSH", key, value)

def load_base_config():
    with open(BASE_CONFIG_PATH, "r") as f:
        return f.read()

def generate_config(base_config, lr, rank, dropout, output_dir):
    # Simple string replacement for now. 
    # In a real scenario, we'd use a yaml parser, but we want to preserve comments/structure.
    config = base_config
    config = config.replace("learning_rate: 0.0002", f"learning_rate: {lr}")
    config = config.replace("lora_r: 64", f"lora_r: {rank}")
    config = config.replace("lora_dropout: 0.05", f"lora_dropout: {dropout}")
    config = config.replace("output_dir: ./qlora-out", f"output_dir: {output_dir}")
    return config

def submit_job(redis, config_path, job_id):
    job = {
        "id": job_id,
        "action_id": f"action-{job_id}",
        "plan_id": f"plan-{job_id}",
        "task_number": 1,
        "command": "train_model",
        "args": [config_path],
        "env": {},
        "status": "pending",
        "tags": ["gpu"]
    }
    
    job_json = json.dumps(job)
    
    print(f"Submitting job {job_id}...")
    redis.set(f"job:{job_id}", job_json)
    redis.lpush("queue:default", job_id)
    print(f"Job {job_id} submitted.")

def main():
    parser = argparse.ArgumentParser(description="Run fine-tuning experiments")
    parser.add_argument("--dry-run", action="store_true", help="Don't submit jobs")
    args = parser.parse_args()

    if not os.path.exists(EXPERIMENTS_DIR):
        os.makedirs(EXPERIMENTS_DIR)

    base_config = load_base_config()
    redis = RedisClient(REDIS_HOST, REDIS_PORT)

    # Grid Search
    learning_rates = [1e-4, 2e-4]
    lora_ranks = [32, 64]
    
    for lr in learning_rates:
        for rank in lora_ranks:
            exp_id = f"exp_lr{lr}_r{rank}".replace(".", "")
            config_filename = f"axolotl_{exp_id}.yaml"
            config_path = os.path.join(EXPERIMENTS_DIR, config_filename)
            output_dir = os.path.join(EXPERIMENTS_DIR, f"out_{exp_id}")
            
            new_config = generate_config(base_config, lr, rank, 0.05, output_dir)
            
            with open(config_path, "w") as f:
                f.write(new_config)
            
            print(f"Generated config: {config_path}")
            
            if not args.dry_run:
                job_id = f"job-{exp_id}-{uuid.uuid4().hex[:8]}"
                submit_job(redis, config_path, job_id)

if __name__ == "__main__":
    main()
