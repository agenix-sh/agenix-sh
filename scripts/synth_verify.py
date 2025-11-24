#!/usr/bin/env python3
import json
import argparse
import subprocess
import tempfile
import os

def verify_plan(plan: list) -> (bool, str):
    """
    Executes a plan in a disposable Docker container.
    Returns (True, "") if all commands succeed.
    Returns (False, stderr) if failed.
    """
    # Create a script from the plan
    script_content = "#!/bin/bash\nset -e\n"
    for step in plan:
        script_content += f"{step['command']}\n"
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as f:
        f.write(script_content)
        script_path = f.name

    # Docker command to run the script
    # Using ubuntu:latest for better compatibility
    docker_cmd = [
        "docker", "run", "--rm",
        "-v", f"{script_path}:/run_plan.sh",
        "ubuntu:latest",
        "bash", "/run_plan.sh"
    ]

    try:
        # Run with timeout
        result = subprocess.run(
            docker_cmd, 
            capture_output=True, 
            text=True, 
            timeout=10
        )
        
        if result.returncode == 0:
            return True, result.stdout
        else:
            return False, result.stderr
            
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, str(e)
    finally:
        if os.path.exists(script_path):
            os.unlink(script_path)

def main():
    parser = argparse.ArgumentParser(description="Verify synthetic data plans")
    parser.add_argument("--input", default="research/raw_candidates.jsonl", help="Input JSONL")
    parser.add_argument("--output", default="research/verified_dataset.jsonl", help="Output JSONL")
    args = parser.parse_args()

    verified_count = 0
    total_count = 0

    with open(args.input, "r") as fin, open(args.output, "w") as fout, open("research/verification_failures.jsonl", "w") as ffail:
        for line in fin:
            total_count += 1
            try:
                item = json.loads(line)
                plan = item.get("plan", [])
                
                if not plan:
                    continue
                
                print(f"Verifying: {item['intent']}...")
                success, output = verify_plan(plan)
                if success:
                    # Add metadata
                    item["verified"] = True
                    fout.write(json.dumps(item) + "\n")
                    verified_count += 1
                    print("  ✅ Success")
                else:
                    item["error"] = output
                    ffail.write(json.dumps(item) + "\n")
                    print(f"  ❌ Failed: {output[:100]}...")
                    
            except json.JSONDecodeError:
                continue

    print(f"Verification complete. {verified_count}/{total_count} plans verified. Failures logged to research/verification_failures.jsonl")

if __name__ == "__main__":
    main()
