# Model Selection for Agenix (Echo & Delta)

## Objectives
- **Echo**: General-purpose chat, instruction following, tool use.
- **Delta**: Complex planning, reasoning, DAG generation, coding.

## Hardware Constraints
- **Local Dev (Mac M-series)**: Efficient inference (MLX/GGUF), 8B-14B range preferred.
- **Worker 1 (Ubuntu, RTX 4090 24GB)**: Fast fine-tuning for 7B-14B models (QLoRA), inference for up to 32B (4-bit).
- **Worker 2 (DGX Spark, Blackwell)**: Heavy lifting, fine-tuning 32B-70B models, high-throughput inference.

## Recommendations

### 1. Delta (Planner)
**Role**: Decompose complex user requests into executable DAGs of tasks. Requires strong reasoning and coding capabilities.

**Top Contender: Qwen 2.5 Coder (32B)**
- **Pros**: State-of-the-art coding performance (rivals GPT-4o in benchmarks). Excellent instruction following. 32B fits comfortably on the Blackwell GPU for fine-tuning and can run on the 4090 (4-bit quantized) for inference.
- **Cons**: 32B is heavy for local Mac dev (requires ~18GB RAM at 4-bit).
- **Fallback**: **Qwen 2.5 Coder (7B)**. Excellent for its size, runs everywhere.

**Strategy**:
- Start with **Qwen 2.5 Coder 32B Instruct** as the baseline for Delta.
- Fine-tune on the Blackwell GPU using the "Fine-Tuning Pipeline".
- Distill to 7B if latency/resource usage on the 4090 is an issue.

### 2. Echo (Chat)
**Role**: Conversational interface, clarification, summarization.

**Top Contender: Llama 3.1 8B Instruct**
- **Pros**: Industry standard for this size class. Robust, uncensored enough for dev tasks, excellent tool use support in libraries. Fast inference on all nodes.
- **Cons**: Context window management can be tricky (128k supported but heavy).
- **Alternative**: **Mistral Nemo 12B**. Great middle ground, very capable.

**Strategy**:
- Use **Llama 3.1 8B Instruct** as the default Echo model.
- It's lightweight enough to run alongside the Planner without choking the 4090.

## Fine-Tuning Targets
We will create two LoRA adapters:
1.  `agenix-delta-v1`: Fine-tuned on Qwen 2.5 Coder 32B.
    -   **Dataset**: Synthetic planning examples (User Request -> JSON DAG).
2.  `agenix-echo-v1`: Fine-tuned on Llama 3.1 8B.
    -   **Dataset**: Agentic interaction logs (clarification questions, status reporting).

## Action Plan
1.  **Baseline**: Deploy `agx` configured to use Qwen 2.5 Coder 32B (via vLLM/Ollama on Worker 2) for Planning and Llama 3.1 8B (local or Worker 1) for Chat.
2.  **Data Gen**: Use the "Echo" chat logs to bootstrap a dataset.
3.  **Train**: Use the `scripts/train.py` pipeline on the Blackwell node.
