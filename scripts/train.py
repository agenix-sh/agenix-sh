import argparse
import os
import torch
from unsloth import FastLanguageModel
from datasets import load_dataset
from trl import SFTTrainer
from transformers import TrainingArguments

def train(args):
    print(f"Starting training with model {args.model} on dataset {args.dataset}")
    
    # 1. Load Model
    max_seq_length = 2048
    dtype = None # None for auto detection
    load_in_4bit = True 

    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name = args.model,
        max_seq_length = max_seq_length,
        dtype = dtype,
        load_in_4bit = load_in_4bit,
    )

    # 2. Add LoRA adapters
    model = FastLanguageModel.get_peft_model(
        model,
        r = 16,
        target_modules = ["q_proj", "k_proj", "v_proj", "o_proj",
                          "gate_proj", "up_proj", "down_proj",],
        lora_alpha = 16,
        lora_dropout = 0, 
        bias = "none", 
        use_gradient_checkpointing = "unsloth", 
        random_state = 3407,
        use_rslora = False,  
        loftq_config = None, 
    )

    # 3. Load Dataset
    # Assuming dataset is a local directory or HF repo
    if os.path.exists(args.dataset):
        dataset = load_dataset("json", data_files=os.path.join(args.dataset, "*.jsonl"), split="train")
    else:
        dataset = load_dataset(args.dataset, split="train")

    # 4. Training Arguments
    training_args = TrainingArguments(
        per_device_train_batch_size = 2,
        gradient_accumulation_steps = 4,
        warmup_steps = 5,
        max_steps = 60, # Short run for demo
        learning_rate = 2e-4,
        fp16 = not torch.cuda.is_bf16_supported(),
        bf16 = torch.cuda.is_bf16_supported(),
        logging_steps = 1,
        optim = "adamw_8bit",
        weight_decay = 0.01,
        lr_scheduler_type = "linear",
        seed = 3407,
        output_dir = args.output_dir,
    )

    # 5. Trainer
    trainer = SFTTrainer(
        model = model,
        tokenizer = tokenizer,
        train_dataset = dataset,
        dataset_text_field = "text",
        max_seq_length = max_seq_length,
        dataset_num_proc = 2,
        packing = False, 
        args = training_args,
    )

    # 6. Train
    trainer_stats = trainer.train()
    print(f"Training complete. Stats: {trainer_stats}")

    # 7. Save
    model.save_pretrained(args.output_dir) 
    tokenizer.save_pretrained(args.output_dir)
    print(f"Model saved to {args.output_dir}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fine-tune LLM using Unsloth")
    parser.add_argument("--model", type=str, required=True, help="Base model name or path")
    parser.add_argument("--dataset", type=str, required=True, help="Dataset path or HF repo")
    parser.add_argument("--output_dir", type=str, required=True, help="Output directory for adapters")
    
    args = parser.parse_args()
    train(args)
