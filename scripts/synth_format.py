#!/usr/bin/env python3
import json
import argparse
import os

def format_for_echo(item):
    """
    Formats a verified item for Echo (Chat) fine-tuning.
    Echo needs to learn to:
    1. Acknowledge the user's intent.
    2. Propose the plan (or just execute it if confident).
    
    For this synthetic data, we'll simulate a direct execution response.
    """
    intent = item['intent']
    plan = item['plan']
    
    # Construct a natural language response that includes the plan execution
    # In a real chat, the agent might ask for confirmation, but for "instruction tuning"
    # we want it to learn to generate the plan.
    
    return {
        "messages": [
            {
                "role": "user", 
                "content": intent
            },
            {
                "role": "assistant", 
                "content": f"I will execute the following plan:\n\nEXECUTING_PLAN: {json.dumps(plan)}"
            }
        ]
    }

def format_for_delta(item):
    """
    Formats a verified item for Delta (Planner) fine-tuning.
    Delta takes an instruction and outputs a JSON DAG of tasks.
    """
    return {
        "instruction": item['intent'],
        "output": item['plan']
    }

def main():
    parser = argparse.ArgumentParser(description="Format verified synthetic data")
    parser.add_argument("--input", default="research/verified_dataset.jsonl", help="Input verified JSONL")
    parser.add_argument("--output-echo", default="research/train_echo.jsonl", help="Output for Echo")
    parser.add_argument("--output-delta", default="research/train_delta.jsonl", help="Output for Delta")
    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"Error: Input file {args.input} not found.")
        return

    echo_count = 0
    delta_count = 0

    with open(args.input, "r") as fin, \
         open(args.output_echo, "w") as f_echo, \
         open(args.output_delta, "w") as f_delta:
        
        for line in fin:
            try:
                item = json.loads(line)
                if not item.get("verified"):
                    continue
                
                # Echo Format
                echo_item = format_for_echo(item)
                f_echo.write(json.dumps(echo_item) + "\n")
                echo_count += 1
                
                # Delta Format
                delta_item = format_for_delta(item)
                f_delta.write(json.dumps(delta_item) + "\n")
                delta_count += 1
                
            except json.JSONDecodeError:
                continue

    print(f"Formatted {echo_count} examples for Echo -> {args.output_echo}")
    print(f"Formatted {delta_count} examples for Delta -> {args.output_delta}")

if __name__ == "__main__":
    main()
