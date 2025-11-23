import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

# Configuration
base_model_name = "Qwen/Qwen2.5-7B-Instruct"
adapter_path = "./qlora-out"
device = "cuda"

print(f"Loading base model: {base_model_name}")
model = AutoModelForCausalLM.from_pretrained(
    base_model_name,
    torch_dtype=torch.bfloat16,
    device_map="auto",
    load_in_4bit=True,
    attn_implementation="sdpa"  # Force SDPA to avoid flash-attn issues
)
tokenizer = AutoTokenizer.from_pretrained(base_model_name)

print(f"Loading adapter: {adapter_path}")
model = PeftModel.from_pretrained(model, adapter_path)

# Inference loop
print("Model loaded. Enter your instruction (Ctrl+C to exit):")
while True:
    try:
        instruction = input("\nInstruction: ")
        if not instruction:
            continue
            
        messages = [
            {"role": "system", "content": "You are the AGX Planner, an intelligent agent responsible for creating execution plans."},
            {"role": "user", "content": instruction}
        ]
        
        text = tokenizer.apply_chat_template(
            messages,
            tokenize=False,
            add_generation_prompt=True
        )
        
        model_inputs = tokenizer([text], return_tensors="pt").to(device)
        
        generated_ids = model.generate(
            model_inputs.input_ids,
            max_new_tokens=512,
            do_sample=True,
            temperature=0.7
        )
        
        generated_ids = [
            output_ids[len(input_ids):] for input_ids, output_ids in zip(model_inputs.input_ids, generated_ids)
        ]
        
        response = tokenizer.batch_decode(generated_ids, skip_special_tokens=True)[0]
        print("\nResponse:")
        print(response)
        
    except KeyboardInterrupt:
        print("\nExiting...")
        break
    except Exception as e:
        print(f"\nError: {e}")
