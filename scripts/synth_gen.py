#!/usr/bin/env python3
import os
import sys
import json
import yaml
import argparse
import time
from typing import List, Dict, Any

import requests

# LLM Configuration
API_BASE = os.environ.get("LLM_API_BASE", "http://spark-a2ae:8080/api")
API_KEY = os.environ.get("LLM_API_KEY", "sk-2876c77587b34f8ba226258824685fc6")

def call_llm(prompt: str, model: str = "gpt-oss:120b") -> str:
    """
    Call Open-WebUI API (OpenAI compatible).
    """
    print(f"Calling LLM ({model})...")
    
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": "You are a helpful assistant that outputs only valid JSON."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 4096
    }
    
    try:
        response = requests.post(f"{API_BASE}/chat/completions", headers=headers, json=payload, timeout=300)
        response.raise_for_status()
        data = response.json()
        return data["choices"][0]["message"]["content"]
    except Exception as e:
        print(f"API Error: {e}")
        # Return empty JSON structure on failure to avoid crashing the loop
        return json.dumps({"candidates": []})

def generate_prompt(domain: Dict[str, Any]) -> str:
    return f"""
    You are an expert DevOps engineer and Bash scripter.
    Generate 5 diverse, complex, and realistic user intents and their corresponding Bash execution plans for the domain: "{domain['domain']}".
    
    Description: {domain['description']}
    Examples:
    {json.dumps(domain['examples'], indent=2)}
    
    Output Format (JSON):
    {{
        "candidates": [
            {{
                "intent": "User's high-level goal",
                "plan": [
                    {{ "command": "bash command 1" }},
                    {{ "command": "bash command 2" }}
                ]
            }}
        ]
    }}
    
    Ensure commands are safe to run in a sandbox (no rm -rf /).
    """

def main():
    parser = argparse.ArgumentParser(description="Generate synthetic data for Agenix")
    parser.add_argument("--domains", default="research/domains.yaml", help="Path to domains YAML")
    parser.add_argument("--output", default="research/raw_candidates.jsonl", help="Output JSONL file")
    parser.add_argument("--model", default="gpt-oss:120b", help="LLM model to use")
    parser.add_argument("--count", type=int, default=10, help="Target examples per domain")
    args = parser.parse_args()

    # Load domains
    with open(args.domains, "r") as f:
        domains = yaml.safe_load(f)

    total_candidates = 0
    
    # Clear output file if it exists to start fresh, or append? 
    # Let's append if we want to run multiple times, but for now let's overwrite to be clean.
    # Actually, let's read existing to know how many we have? No, simple overwrite for this task.
    with open(args.output, "w") as f:
        pass

    for domain in domains:
        print(f"Generating for domain: {domain['domain']} (Target: {args.count})...")
        domain_candidates = []
        
        while len(domain_candidates) < args.count:
            prompt = generate_prompt(domain)
            try:
                response_text = call_llm(prompt, model=args.model)
                clean_text = response_text.strip()
                if clean_text.startswith("```json"):
                    clean_text = clean_text[7:]
                if clean_text.endswith("```"):
                    clean_text = clean_text[:-3]
                
                data = json.loads(clean_text)
                
                new_items = data.get("candidates", [])
                if not new_items:
                    print("  No candidates in response, retrying...")
                    continue

                for item in new_items:
                    item["domain"] = domain["domain"]
                    domain_candidates.append(item)
                    
                    # Write immediately to avoid data loss
                    with open(args.output, "a") as f:
                        f.write(json.dumps(item) + "\n")
                
                print(f"  Progress: {len(domain_candidates)}/{args.count}")
                    
            except Exception as e:
                print(f"  Error generating for {domain['domain']}: {e}")
            
            time.sleep(1) # Rate limit

        total_candidates += len(domain_candidates)

    print(f"Generated {total_candidates} candidates in {args.output}")

if __name__ == "__main__":
    main()
