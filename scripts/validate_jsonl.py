import json
import sys

def validate(path):
    print(f"Validating {path}...")
    with open(path, "r") as f:
        for i, line in enumerate(f):
            try:
                item = json.loads(line)
                messages = item.get("messages", [])
                if not isinstance(messages, list):
                    print(f"Line {i+1}: messages is not a list")
                    continue
                
                for j, msg in enumerate(messages):
                    if not isinstance(msg, dict):
                        print(f"Line {i+1}, msg {j}: not a dict")
                        continue
                    
                    if "role" not in msg or not isinstance(msg["role"], str):
                        print(f"Line {i+1}, msg {j}: role missing or not str: {msg.get('role')}")
                    
                    if "content" not in msg:
                        print(f"Line {i+1}, msg {j}: content missing")
                    elif not isinstance(msg["content"], str):
                        print(f"Line {i+1}, msg {j}: content is not str (type: {type(msg['content'])}): {msg['content']}")
                        
            except json.JSONDecodeError:
                print(f"Line {i+1}: Invalid JSON")

if __name__ == "__main__":
    validate(sys.argv[1])
