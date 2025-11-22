---
description: Fine-Tune LLM using Unsloth/QLoRA
---

# Fine-Tuning Workflow

This workflow defines the standard DAG for fine-tuning LLMs using Unsloth and QLoRA.

## Steps

1.  **Data Preparation** (CPU)
    - **Command**: `python3 scripts/prep_data.py --input <input_file> --output <output_dir>`
    - **Queue**: `queue:default` (CPU)
    - **Output**: JSONL or Parquet files ready for training.

2.  **Training** (GPU)
    - **Command**: `python3 scripts/train.py --dataset <prep_output> --model <base_model> --output_dir <lora_output>`
    - **Queue**: `queue:gpu`
    - **Requirements**: GPU with at least 16GB VRAM (for 7B models).
    - **Output**: LoRA adapters in `<lora_output>`.

3.  **Evaluation** (GPU)
    - **Command**: `python3 scripts/evaluate.py --model <base_model> --adapters <lora_output> --benchmark <benchmark_name>`
    - **Queue**: `queue:gpu`
    - **Dependencies**: Training must complete successfully.

4.  **Publish** (CPU)
    - **Command**: `python3 scripts/publish.py --adapters <lora_output> --repo <hf_repo_id>`
    - **Queue**: `queue:default`
    - **Dependencies**: Evaluation must pass threshold.

## Example Plan JSON

```json
{
  "plan_id": "ft-llama3-001",
  "tasks": [
    {
      "task_number": 1,
      "command": "python3",
      "args": ["scripts/prep_data.py", "--input", "data/raw.txt", "--output", "data/processed"],
      "tags": ["cpu"]
    },
    {
      "task_number": 2,
      "command": "python3",
      "args": ["scripts/train.py", "--dataset", "data/processed", "--model", "unsloth/llama-3-8b-bnb-4bit", "--output_dir", "models/lora-001"],
      "tags": ["gpu"],
      "dependencies": [1]
    },
    {
      "task_number": 3,
      "command": "python3",
      "args": ["scripts/evaluate.py", "--adapters", "models/lora-001"],
      "tags": ["gpu"],
      "dependencies": [2]
    }
  ]
}
```
