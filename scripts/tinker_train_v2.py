#!/usr/bin/env python3
import sys
import os
import json
import asyncio
import chz
from tinker_cookbook import cli_utils, model_info
from tinker_cookbook.renderers import TrainOnWhat
from tinker_cookbook.supervised import train
from tinker_cookbook.supervised.data import FromConversationFileBuilder
from tinker_cookbook.supervised.types import ChatDatasetBuilderCommonConfig

# Ensure API Key is present
if "TINKER_API_KEY" not in os.environ:
    print("Error: TINKER_API_KEY environment variable not set.")
    sys.exit(1)

def convert_delta_to_chat(input_path, output_path):
    """Converts Delta (Instruction/Output) to Chat format for Tinker."""
    print(f"Converting {input_path} to {output_path}...")
    with open(input_path, "r") as fin, open(output_path, "w") as fout:
        for line in fin:
            try:
                item = json.loads(line)
                # Delta format: {"instruction": "...", "output": [...]}
                # Chat format: {"messages": [{"role": "user", "content": "..."}, {"role": "assistant", "content": "..."}]}
                
                # Ensure output is a string (JSON dump if it's a list/dict)
                output_content = item["output"]
                if not isinstance(output_content, str):
                    output_content = json.dumps(output_content)

                chat_item = {
                    "messages": [
                        {"role": "user", "content": item["instruction"]},
                        {"role": "assistant", "content": output_content}
                    ]
                }
                fout.write(json.dumps(chat_item) + "\n")
            except Exception as e:
                print(f"Skipping line due to error: {e}")

def run_training(job_name, data_path, model_name="meta-llama/Llama-3.1-8B-Instruct"):
    print(f"\n=== Starting Training Job: {job_name} ===")
    print(f"Model: {model_name}")
    print(f"Data: {data_path}")
    
    log_path = f"tinker_logs/{job_name}"
    
    renderer_name = model_info.get_recommended_renderer_name(model_name)
    common_config = ChatDatasetBuilderCommonConfig(
        model_name_for_tokenizer=model_name,
        renderer_name=renderer_name,
        max_length=4096, # Increased for complex plans
        batch_size=4,    # Reduced for stability
        train_on_what=TrainOnWhat.ALL_ASSISTANT_MESSAGES,
    )
    
    dataset = FromConversationFileBuilder(
        common_config=common_config, 
        file_path=os.path.abspath(data_path)
    )

    # Create configuration
    config = chz.Blueprint(train.Config).apply({
        "log_path": log_path,
        "model_name": model_name,
        "dataset_builder": dataset,
        "learning_rate": 2e-4,
        "lr_schedule": "linear",
        "num_epochs": 3,
        "eval_every": 50,
        "save_every": 100,
    }).make()

    # Run training
    # cli_utils.check_log_dir(config.log_path, behavior_if_exists="overwrite")
    asyncio.run(train.main(config))
    print(f"=== Job {job_name} Completed ===")

def main():
    # 1. Prepare Delta Data
    delta_raw = "research/train_delta.jsonl"
    delta_chat = "research/train_delta_chat_tinker.jsonl"
    convert_delta_to_chat(delta_raw, delta_chat)

    # 2. Train Delta
    # run_training("agenix-delta-v1", delta_chat)
    
    # 3. Train Echo
    # Echo data is already in chat format
    echo_chat = "research/train_echo_chat.jsonl"
    run_training("agenix-echo-v1", echo_chat)
    
    # Note: Running both sequentially might take time. 
    # Uncomment Delta above to run it too. 
    # For now, let's run Echo as a test since it's ready.
    
    print("\nAll jobs finished.")

if __name__ == "__main__":
    main()
