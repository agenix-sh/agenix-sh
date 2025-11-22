# agx-eval

**Generic LLM Evaluation Agentic Unit for AGEniX**

`agx-eval` is a generic wrapper around LLM inference that performs evaluation and reasoning over any data with any intent. It follows the Unix philosophy: do one thing (LLM reasoning) and do it well.

## Core Principle

**The AU provides the mechanism (LLM reasoning), the user provides the meaning (context + intent).**

- ❌ **Wrong**: Hardcoded task types (cv_screening, compliance, anomaly_detection)
- ✅ **Right**: Generic evaluation with user-defined context and criteria

## Quick Start

```bash
# Install
cargo install --path .

# Example: CV screening
cat resume.txt | agx-eval \
  --context "Job requirements: Senior backend engineer, 3+ years Rust, distributed systems" \
  --prompt "Does this candidate meet requirements? Decision (accept/reject), confidence, reasoning."

# Example: Data quality check
echo '{"age": -5, "email": "invalid"}' | agx-eval \
  --context "Valid data rules: age must be 0-120, email must match regex" \
  --prompt "List all validation failures with severity."

# Example: Anomaly detection
cat metrics.json | agx-eval \
  --context "Baseline: API response times 50-200ms, error rate <0.1%" \
  --prompt "Is this an anomaly? Classify severity and explain."
```

## Agentic Unit Contract

### Input (stdin)
- Structured JSON: `{"data": "..."}`
- Unstructured text: Raw string
- Piped from previous task output

### Arguments

```
agx-eval \
  --context <string>      Background info, criteria, domain knowledge (REQUIRED)
  --prompt <string>       Evaluation question/instruction (REQUIRED)
  [--model <name>]        LLM model (default: qwen2.5:1.5b)
  [--temperature <float>] Sampling temperature (default: 0.1)
  [--max-tokens <int>]    Max response tokens (default: 500)
  [--format json|text]    Output format (default: json)
```

### Output (stdout)

**JSON format (default):**
```json
{
  "status": "success",
  "result": {
    "decision": "accept",
    "reasoning": "Candidate has 5+ years Rust experience...",
    "confidence": 0.85,
    "evidence": ["Built distributed database", "Contributed to Tokio"]
  },
  "metadata": {
    "model": "qwen2.5:1.5b",
    "tokens_used": 245,
    "latency_ms": 1234
  }
}
```

**Text format:**
```
Decision: accept
Reasoning: Candidate has 5+ years Rust experience with strong distributed systems background.
Confidence: 0.85
```

## Use Cases

### 1. CV Screening
```bash
cat resume.txt | agx-eval \
  --context "Job: Senior Backend Engineer. Must have: 3+ years Rust, distributed systems, async programming" \
  --prompt "Evaluate candidate fit. Provide decision (accept/reject/maybe), confidence (0-1), and detailed reasoning."
```

### 2. Compliance Checking
```bash
cat expense-report.json | agx-eval \
  --context "Policy: Expenses <$500 with receipts. >$500 needs manager approval. Meals <$50." \
  --prompt "Is this report compliant? List violations if any."
```

### 3. Sentiment Analysis
```bash
cat feedback.txt | agx-eval \
  --context "Analyzing customer satisfaction for support team" \
  --prompt "Extract sentiment (positive/negative/mixed), key concerns, actionable items."
```

### 4. Anomaly Detection
```bash
cat metrics.json | agx-eval \
  --context "Normal: 50-200ms latency, <0.1% errors, 100-1000 RPS" \
  --prompt "Detect anomalies. Classify severity (low/medium/high) and recommend actions."
```

### 5. Custom Business Logic
```bash
cat transaction.json | agx-eval \
  --context "Rules: Transactions >$10k need approval. Weekend transactions need audit. Foreign currency flagged." \
  --prompt "Should this be flagged? List applicable policies."
```

## Integration with AGX Plans

### Single Evaluation
```json
{
  "tasks": [
    {
      "task_number": 1,
      "command": "cat",
      "args": ["data.txt"]
    },
    {
      "task_number": 2,
      "command": "agx-eval",
      "args": [
        "--context", "Your criteria here",
        "--prompt", "Your question here"
      ],
      "input_from_task": 1
    }
  ]
}
```

### Multi-Stage Pipeline
```json
{
  "tasks": [
    {
      "task_number": 1,
      "command": "agx-ocr",
      "args": ["--input", "resume.pdf"]
    },
    {
      "task_number": 2,
      "command": "agx-eval",
      "args": [
        "--context", "Job requirements: Senior Rust developer",
        "--prompt", "Does candidate meet requirements?"
      ],
      "input_from_task": 1
    },
    {
      "task_number": 3,
      "command": "agx-eval",
      "args": [
        "--context", "Company culture: remote-first, open source contributors preferred",
        "--prompt", "Assess cultural fit based on resume content"
      ],
      "input_from_task": 1
    }
  ]
}
```

## Features

### Phase 1 (MVP)
- [x] Generic prompt builder (context + data + instruction)
- [x] Ollama integration (any model)
- [x] JSON response parsing with fallback
- [x] Stdin → evaluation → stdout pipeline
- [x] Configurable temperature and max tokens
- [x] Error handling for malformed LLM responses
- [x] Text and JSON output formats

### Phase 2 (Backend Abstraction)
- [ ] Abstract `Backend` trait for LLM inference
- [ ] Candle backend with GGUF model support
- [ ] `--backend` CLI flag (ollama|candle)
- [ ] Air-gapped deployment support

### Phase 3 (Advanced Features)
- [ ] Multi-model evaluation (compare outputs)
- [ ] Confidence calibration
- [ ] Few-shot examples via `--examples` flag
- [ ] Chain-of-thought prompting
- [ ] Response caching for identical evaluations
- [ ] Batch processing mode

## Architecture

```
┌─────────────┐
│   stdin     │ (data to evaluate)
└──────┬──────┘
       │
       v
┌─────────────────────────────────────┐
│  Prompt Builder                     │
│  - Combines context + data + prompt │
│  - Structures evaluation request    │
└──────┬──────────────────────────────┘
       │
       v
┌─────────────────────────────────────┐
│  LLM Client (Ollama)                │
│  - Sends prompt to local model      │
│  - Handles streaming/non-streaming  │
└──────┬──────────────────────────────┘
       │
       v
┌─────────────────────────────────────┐
│  Response Parser                    │
│  - Extracts JSON from markdown      │
│  - Validates structure              │
│  - Maps to EvaluationResult         │
└──────┬──────────────────────────────┘
       │
       v
┌─────────────┐
│   stdout    │ (evaluation result)
└─────────────┘
```

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug ./target/release/agx-eval --context "..." --prompt "..."

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## LLM Backend

### Current: Ollama (MVP)

For development and testing, agx-eval uses **Ollama** as the LLM backend:

```bash
# Install Ollama (if not already)
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull qwen2.5:1.5b

# Start Ollama
ollama serve

# Use agx-eval
echo "data" | agx-eval --context "criteria" --prompt "evaluate"
```

### Future: Candle Backend (Air-gapped)

For **production deployment in secure/air-gapped environments**, agx-eval will support embedded inference using **Candle + GGUF models** (similar to agx-ocr).

This will enable:
- ✅ True standalone binary (no external dependencies)
- ✅ Fine-tuned specialized models for evaluation
- ✅ Air-gapped deployment
- ✅ Deterministic results

**Planned usage:**
```bash
# Future: Candle backend with embedded model
./agx-eval --backend candle --model-path /path/to/eval-model.gguf \
  --context "..." --prompt "..." < data.txt
```

See [CLAUDE.md §11](CLAUDE.md#11-llm-backend-strategy) for full backend strategy.

## Environment Variables

- `OLLAMA_ENDPOINT`: Ollama API endpoint (default: `http://localhost:11434`)
- `RUST_LOG`: Logging level (debug, info, warn, error)

## Error Handling

All errors are reported via JSON on stdout with `"status": "error"`:

```json
{
  "status": "error",
  "error": {
    "code": "llm_connection_failed",
    "message": "Failed to connect to Ollama",
    "details": "Connection refused at http://localhost:11434"
  }
}
```

## License

MIT OR Apache-2.0

## Contributing

See [CLAUDE.md](CLAUDE.md) for AI development guidelines.

## References

- [Agentic Unit Specification](https://github.com/agenix-sh/agenix/blob/main/docs/au-specs/agentic-unit-spec.md)
- [AGEniX Architecture](https://github.com/agenix-sh/agenix/blob/main/docs/architecture/)
- [Issue #12: agx-eval specification](https://github.com/agenix-sh/agenix/issues/12)
