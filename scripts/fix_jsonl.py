import json
import sys
import os

def fix_jsonl(input_path, output_path):
    print(f"Fixing {input_path} -> {output_path}...")
    fixed_count = 0
    skipped_count = 0
    
    with open(input_path, "r") as fin, open(output_path, "w") as fout:
        for line in fin:
            try:
                item = json.loads(line)
                messages = item.get("messages", [])
                
                if not messages:
                    skipped_count += 1
                    continue
                    
                new_messages = []
                for msg in messages:
                    if not isinstance(msg, dict):
                        continue
                        
                    role = msg.get("role")
                    content = msg.get("content")
                    
                    # Fix Role
                    if not role:
                        # Heuristic: if content looks like a plan request, it's Echo (Assistant)
                        # But usually we can't easily guess. 
                        # If it's None, let's skip this message or default to "assistant" if it looks like output?
                        # Looking at the logs: "Line 456, msg 0: role missing or not str: None"
                        # This implies the whole message structure is broken.
                        # Let's skip messages with no role.
                        continue
                    
                    # Fix Content
                    if not isinstance(content, str):
                        if content is None:
                            content = ""
                        else:
                            content = json.dumps(content)
                            
                    new_messages.append({
                        "role": role,
                        "content": content
                    })
                
                if new_messages:
                    item["messages"] = new_messages
                    fout.write(json.dumps(item) + "\n")
                    fixed_count += 1
                else:
                    skipped_count += 1
                    
            except json.JSONDecodeError:
                skipped_count += 1
                continue

    print(f"Fixed {fixed_count} lines. Skipped {skipped_count} lines.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python fix_jsonl.py <input_file>")
        sys.exit(1)
        
    input_file = sys.argv[1]
    # Overwrite in place (safe because we read all lines first? No, better to write to temp and move)
    # Actually, let's write to a new file and then rename
    temp_file = input_file + ".tmp"
    fix_jsonl(input_file, temp_file)
    os.rename(temp_file, input_file)
    print("Done.")
