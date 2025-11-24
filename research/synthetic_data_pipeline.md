# Synthetic Data Generation Pipeline

## Objective
Generate high-quality, verified (intent, plan) pairs for fine-tuning Echo (Chat) and Delta (Planner) models using the Tinker SaaS platform.

## Strategy
We will use a **Self-Verifying Execution Loop**:
1.  **Seed**: A list of high-level domains (e.g., "File System", "Git", "Network", "System Info").
2.  **Generate**: Use a Frontier Model (Claude 3.5/GPT-4) to generate diverse *intents* and corresponding *bash plans* for a given domain.
3.  **Verify**: Execute the generated plans in a safe sandbox (Docker/Native) to ensure they run successfully (exit code 0) and produce expected output.
4.  **Format**: Convert successful pairs into JSONL formats for Echo (Conversation) and Delta (Instruction -> JSON DAG).

## Pipeline Components

### 1. `scripts/synth_gen.py` (Generator)
- **Input**: `domains.yaml` (List of topics/skills).
- **LLM Interface**: Calls Anthropic/OpenAI API.
- **Prompting**:
    - Ask for "complex, multi-step intents" (e.g., "Find all large log files and compress them").
    - Ask for the corresponding "Bash script" or "JSON DAG".
- **Output**: `raw_candidates.jsonl`

### 2. `scripts/synth_verify.py` (Verifier)
- **Input**: `raw_candidates.jsonl`
- **Execution Engine**:
    - Uses `agw`'s Sandbox or a temporary Docker container.
    - Sets up a "pre-condition" if needed (e.g., create dummy files to find). *Self-correction: This is hard. Easier to verify syntax or run "safe" read-only commands, OR run in a disposable container where destruction is fine.*
    - **Decision**: We will use a **Disposable Docker Container** for verification.
- **Validation Logic**:
    - Run command.
    - Check Exit Code == 0.
    - (Optional) Check if output matches a regex or heuristic.
- **Output**: `verified_dataset.jsonl`

## Data Formats

### Delta (Planner) Format
```json
{
  "instruction": "Find all PDF files in Documents and move them to a 'Papers' folder",
  "output": [
    { "command": "mkdir -p ~/Documents/Papers" },
    { "command": "find ~/Documents -name '*.pdf' -exec mv {} ~/Documents/Papers/ \\;" }
  ]
}
```

### Echo (Chat) Format
```json
{
  "messages": [
    { "role": "user", "content": "I need to organize my PDFs." },
    { "role": "assistant", "content": "I can help with that. I'll look for PDF files in your Documents folder and move them to a new 'Papers' directory. Shall I proceed?" },
    { "role": "user", "content": "Yes, please." },
    { "role": "assistant", "content": "EXECUTING_PLAN: ..." }
  ]
}
```

## Implementation Steps
1.  Create `domains.yaml` with initial categories.
2.  Implement `synth_gen.py` with a mock LLM provider (or real one if key available).
3.  Implement `synth_verify.py` using `docker run`.
4.  Run pipeline to generate ~100 examples as a proof of concept.
