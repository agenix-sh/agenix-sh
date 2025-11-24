#!/usr/bin/env python3
import os
import sys
import asyncio
import tinker

# Configuration
TINKER_API_KEY = os.environ.get("TINKER_API_KEY")
# From logs: tinker://4891cc84-76cb-5204-a924-02ab64169ad0:train:0/weights/final
# But create_sampling_client might expect just the ID or a specific format.
# Let's try the full path first.
MODEL_PATH = "tinker://4891cc84-76cb-5204-a924-02ab64169ad0:train:0/weights/final"

async def chat_loop():
    if not TINKER_API_KEY:
        print("Error: TINKER_API_KEY not set.")
        return

    print("Initializing Tinker Client...")
    service_client = tinker.ServiceClient()
    
    print(f"Connecting to model: {MODEL_PATH}")
    # Note: If this fails, we might need to use the job ID or a different format
    sampling_client = service_client.create_sampling_client(model_path=MODEL_PATH)
    
    print("\n=== Agenix Echo (Tinker) ===")
    print("Type 'exit' to quit.\n")
    
    messages = [
        {"role": "system", "content": "You are Echo, an intelligent assistant for the Agenix platform."}
    ]
    
    while True:
        user_input = input("User > ")
        if user_input.lower() in ["exit", "quit"]:
            break
            
        messages.append({"role": "user", "content": user_input})
        
        print("Echo > ", end="", flush=True)
        
        # Stream response
        full_response = ""
        async for token in sampling_client.sample_stream(
            messages=messages, 
            max_tokens=512, 
            temperature=0.7
        ):
            print(token, end="", flush=True)
            full_response += token
            
        print("\n")
        messages.append({"role": "assistant", "content": full_response})

if __name__ == "__main__":
    try:
        asyncio.run(chat_loop())
    except KeyboardInterrupt:
        print("\nGoodbye!")
    except Exception as e:
        print(f"\nError: {e}")
