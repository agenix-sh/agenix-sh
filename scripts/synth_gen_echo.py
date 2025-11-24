#!/usr/bin/env python3
import os
import json
import argparse
import requests
import time

# LLM Configuration
API_BASE = os.environ.get("LLM_API_BASE", "http://spark-a2ae:8080/api")
API_KEY = os.environ.get("LLM_API_KEY", "sk-2876c77587b34f8ba226258824685fc6")

SYSTEM_PROMPT = """
You are an expert data generator for an AI Agent named "Echo".
Echo is a helpful, professional DevOps assistant.
Echo's capabilities:
1.  **Clarification**: If a user request is vague, Echo asks clarifying questions.
2.  **Tool Use**: Echo can use tools like `get_worker_status`, `list_jobs`, `check_pipeline`.
3.  **Rejection**: Echo politely refuses impossible tasks (e.g., "make coffee", "buy groceries").
4.  **Handoff**: Once the intent is clear, Echo outputs a structured `PLAN_REQUEST` for the planner (Delta).

Generate 5 synthetic conversation logs (JSON) between "User" and "Echo" based on a provided scenario.

Output Format (JSON):
{
  "conversations": [
      {
          "scenario": "User asks to fix a build...",
          "messages": [...]
      },
      ...
  ]
}
"""

SCENARIOS = [
    "User asks for a system check (requires tool use).",
    "User gives a vague file command (requires clarification).",
    "User asks for something impossible (requires rejection).",
    "User asks to deploy to production (requires confirmation and status check).",
    "User asks to clean up old docker images (straightforward request)."
]

def call_llm(prompt: str, model: str = "gpt-oss:120b") -> str:
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": f"Generate a conversation for this scenario: {prompt}"}
        ],
        "temperature": 0.8,
        "max_tokens": 2048
    }
    
    try:
        response = requests.post(f"{API_BASE}/chat/completions", headers=headers, json=payload, timeout=300)
        response.raise_for_status()
        data = response.json()
        content = data["choices"][0]["message"]["content"]
        # Extract JSON if wrapped in markdown
        if "```json" in content:
            content = content.split("```json")[1].split("```")[0].strip()
        elif "```" in content:
            content = content.split("```")[1].split("```")[0].strip()
        return content
    except Exception as e:
        print(f"API Error: {e}")
        return "{}"

def main():
    parser = argparse.ArgumentParser(description="Generate Echo conversation data")
    parser.add_argument("--output", default="research/train_echo_chat.jsonl", help="Output JSONL")
    parser.add_argument("--count", type=int, default=5, help="Number of examples per scenario")
    parser.add_argument("--model", default="gpt-oss:120b", help="LLM model to use")
    args = parser.parse_args()

    with open(args.output, "w") as f:
        for scenario in SCENARIOS:
            print(f"Generating for scenario: {scenario} (Target: {args.count})")
            generated_count = 0
            
            while generated_count < args.count:
                try:
                    json_str = call_llm(scenario, model=args.model)
                    data = json.loads(json_str)
                    
                    new_items = data.get("conversations", [])
                    if not new_items:
                        # Fallback for single item response
                        if "messages" in data:
                            new_items = [data]
                        else:
                            print("  No conversations in response, retrying...")
                            continue

                    for item in new_items:
                        f.write(json.dumps(item) + "\n")
                        generated_count += 1
                    
                    print(f"  Progress: {generated_count}/{args.count}")
                        
                except json.JSONDecodeError:
                    print(f"  Failed to parse JSON")
                except Exception as e:
                    print(f"  Error: {e}")
                time.sleep(1)

if __name__ == "__main__":
    main()
