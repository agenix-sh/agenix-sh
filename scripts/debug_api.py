import requests
import os

API_BASE = "http://spark-a2ae:8080"
API_KEY = "sk-2876c77587b34f8ba226258824685fc6"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

endpoints = [
    "/v1/models",
    "/api/models",
    "/models",
    "/v1/chat/completions",
    "/api/chat/completions",
    "/ollama/api/chat"
]

print(f"Probing {API_BASE}...")

for ep in endpoints:
    url = f"{API_BASE}{ep}"
    try:
        # Try GET first
        print(f"GET {url}...", end=" ")
        resp = requests.get(url, headers=headers, timeout=5)
        print(f"{resp.status_code}")
        if resp.status_code == 200:
            print(f"  Response: {resp.text[:100]}...")
            
        # Try POST for chat endpoints
        if "chat" in ep:
            print(f"POST {url}...", end=" ")
            resp = requests.post(url, headers=headers, json={"model": "gpt-oss:120b", "messages": []}, timeout=5)
            print(f"{resp.status_code}")
            
    except Exception as e:
        print(f"Error: {e}")
