# agx-eval Development Guidelines

**Repository:** agx-eval (Generic LLM Evaluation AU)
**Version:** 0.1.0
**Status:** Active Development

---

## 1. Introduction

This document provides guidelines for AI agents working on agx-eval. This is an **Agentic Unit (AU)** - a composable, single-purpose tool in the AGEniX ecosystem that follows the Unix philosophy.

**Key principle**: agx-eval provides **zero domain logic**. It's a pure LLM wrapper. All intelligence comes from user-provided context and prompts.

---

## 2. AU Contract Requirements

### 2.1 Input (stdin)

agx-eval MUST accept data via stdin in two formats:

1. **Structured JSON**: `{"data": "..."}`  or any valid JSON object
2. **Unstructured text**: Raw string

```bash
# JSON input
echo '{"transaction_id": 123, "amount": 9999}' | agx-eval ...

# Text input
cat resume.txt | agx-eval ...
```

### 2.2 Output (stdout)

agx-eval MUST write results to stdout in valid JSON format:

```json
{
  "status": "success|error",
  "result": {
    "decision": "...",
    "reasoning": "...",
    "confidence": 0.0-1.0,
    "evidence": ["..."]
  },
  "metadata": {
    "model": "qwen2.5:1.5b",
    "tokens_used": 245,
    "latency_ms": 1234
  },
  "error": null
}
```

**Critical**: NEVER write logs, debug output, or progress messages to stdout. Use stderr or tracing.

### 2.3 Exit Codes

- **0**: Success (evaluation completed, result in JSON)
- **1**: Generic error (LLM failed, parsing error, etc.)
- **2**: Invalid arguments (missing --context or --prompt)

### 2.4 No Side Effects

agx-eval MUST NOT:
- Write files
- Make network calls (except to LLM endpoint)
- Modify system state
- Persist data

---

## 3. Generic Design Principles

### 3.1 No Hardcoded Task Types

❌ **NEVER DO THIS**:
```rust
enum TaskType {
    CvScreening,
    ComplianceCheck,
    AnomalyDetection,
}

match task_type {
    TaskType::CvScreening => screen_cv(data),
    ...
}
```

✅ **DO THIS**:
```rust
// Generic evaluation - user defines everything
let prompt = PromptBuilder::new()
    .with_context(&args.context)
    .with_data(&stdin_data)
    .with_instruction(&args.prompt)
    .build();

let result = llm.evaluate(&prompt).await?;
```

### 3.2 User-Defined Intent

All specificity comes from THREE user-provided arguments:

1. **--context**: Domain knowledge, criteria, baseline (user provides)
2. **--prompt**: Evaluation question/instruction (user provides)
3. **stdin data**: The data to evaluate (from previous task or file)

### 3.3 Prompt Structure

The AU provides a GENERIC prompt template:

```
# Context
{user_context}

# Data to Evaluate
{stdin_data}

# Task
{user_prompt}

Provide your response in JSON format with:
- "decision" or "result": Your evaluation
- "reasoning": Explain step-by-step
- "confidence": 0-1 score
- "evidence": Key facts supporting your decision

Response:
```

**Never modify this structure for specific use cases**. It must work for ALL evaluation tasks.

---

## 4. Development Workflow

### 4.1 Test-Driven Development (TDD)

1. Write test FIRST
2. Implement minimal code to pass
3. Refactor
4. Repeat

```bash
# Run tests
cargo test

# Run specific test
cargo test test_cv_screening_evaluation
```

### 4.2 Code Structure

```
src/
├── main.rs           # CLI args, stdin/stdout, orchestration
├── prompt.rs         # Generic prompt builder
├── llm.rs            # LLM client (Ollama integration)
├── parser.rs         # Response parsing and validation
└── lib.rs            # Public API (for testing)
```

**Never add**:
- `src/cv_screening.rs` ❌
- `src/compliance.rs` ❌
- `src/anomaly_detection.rs` ❌

These would violate the generic design.

### 4.3 Error Handling

Use `anyhow` for application code, `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

fn evaluate(data: &str) -> Result<EvaluationResult> {
    let prompt = build_prompt(data)
        .context("Failed to build prompt")?;

    llm.call(&prompt)
        .await
        .context("LLM inference failed")?
}
```

**Never panic** in production code paths.

---

## 5. Security Considerations

### 5.1 Prompt Injection Prevention

User-provided context and prompts could contain malicious instructions. Validate:

```rust
fn sanitize_user_input(input: &str) -> Result<String> {
    // Limit length (prevent token exhaustion)
    if input.len() > 10000 {
        return Err(Error::InputTooLarge);
    }

    // No null bytes
    if input.contains('\0') {
        return Err(Error::InvalidCharacters);
    }

    Ok(input.to_string())
}
```

### 5.2 Resource Limits

- **Max input size**: 10KB for --context, 1KB for --prompt, 1MB for stdin
- **Timeout**: 30s for LLM calls (configurable)
- **Token limit**: Enforce via --max-tokens (default 500)

### 5.3 No Secrets in Output

Never log or output:
- Full prompts (may contain sensitive data)
- LLM API keys (use env vars only)
- User data verbatim (summarize in reasoning only)

---

## 6. Testing Requirements

### 6.1 Unit Tests

Test each module in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_combines_context_and_data() {
        let prompt = PromptBuilder::new()
            .with_context("Job: Senior Rust dev")
            .with_data("Candidate: 5 years Rust")
            .with_instruction("Evaluate fit")
            .build();

        assert!(prompt.contains("Job: Senior Rust dev"));
        assert!(prompt.contains("Candidate: 5 years Rust"));
    }
}
```

### 6.2 Integration Tests

Test end-to-end with mock LLM:

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_cv_screening_workflow() {
    let input = "Candidate: John, 5 years Rust...";
    let context = "Job: Senior Backend Engineer...";
    let prompt = "Does candidate meet requirements?";

    let result = evaluate_with_mock_llm(input, context, prompt).await?;

    assert_eq!(result.status, "success");
    assert!(result.result.confidence > 0.5);
}
```

### 6.3 Property-Based Tests

Use `proptest` to test invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_any_context_and_prompt_produces_valid_json(
        context in "\\PC{1,100}",
        prompt in "\\PC{1,100}",
        data in "\\PC{1,500}",
    ) {
        let result = evaluate(&data, &context, &prompt);
        assert!(result.is_ok());
        assert!(serde_json::from_str::<EvaluationResult>(&result?).is_ok());
    }
}
```

### 6.4 Coverage Target

- **Minimum 80% code coverage**
- **100% for prompt building and parsing** (security-critical)

---

## 7. Performance Considerations

### 7.1 Latency Targets

- **Prompt building**: <1ms
- **LLM call**: <5s (model-dependent, use qwen2.5:1.5b for speed)
- **Response parsing**: <10ms
- **Total**: <5.1s end-to-end

### 7.2 Memory Limits

- **Max heap**: 100MB
- **No unbounded allocations** (validate input sizes)

---

## 8. Common Patterns

### 8.1 Reading Stdin

```rust
use std::io::{self, Read};

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)
        .context("Failed to read stdin")?;

    // Limit size
    if buffer.len() > 1_048_576 {
        return Err(Error::InputTooLarge);
    }

    Ok(buffer)
}
```

### 8.2 Ollama API Call

```rust
use reqwest::Client;

pub struct OllamaClient {
    endpoint: String,
    client: Client,
}

impl OllamaClient {
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
            }
        });

        let response = self.client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&request)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .context("Ollama API call failed")?;

        let body: serde_json::Value = response.json().await?;
        let text = body["response"]
            .as_str()
            .ok_or_else(|| Error::MissingField("response"))?;

        Ok(text.to_string())
    }
}
```

### 8.3 JSON Extraction from Markdown

LLMs often wrap JSON in markdown blocks:

```rust
fn extract_json_from_markdown(raw: &str) -> Result<String> {
    // Try to find ```json ... ``` block
    if let Some(start) = raw.find("```json") {
        if let Some(end) = raw[start..].find("```") {
            let json_start = start + 7; // len("```json\n")
            let json_end = start + end;
            return Ok(raw[json_start..json_end].trim().to_string());
        }
    }

    // If no markdown wrapper, assume entire response is JSON
    Ok(raw.trim().to_string())
}
```

---

## 9. Common Mistakes to Avoid

### 9.1 Adding Use-Case Logic

❌ **DON'T**:
```rust
if args.context.contains("resume") {
    // Special CV screening logic
    return screen_cv(&data);
}
```

This breaks the generic design. ALL logic should be LLM-driven.

### 9.2 Modifying Prompts for Specific Tasks

❌ **DON'T**:
```rust
let prompt = if is_compliance_check {
    format!("Check compliance: {}", data)
} else {
    format!("Evaluate: {}", data)
};
```

The prompt template must be task-agnostic.

### 9.3 Hardcoding Model Names

❌ **DON'T**:
```rust
const MODEL: &str = "qwen2.5:1.5b";
```

✅ **DO**:
```rust
let model = args.model.unwrap_or("qwen2.5:1.5b".to_string());
```

Users should control model selection.

---

## 10. Git Workflow

```bash
# Create feature branch
git checkout -b feat/prompt-builder

# Make changes with TDD
cargo test
cargo clippy
cargo fmt

# Commit
git commit -m "feat(prompt): add generic prompt builder

Implement PromptBuilder that combines user context, stdin data,
and evaluation instruction into a structured prompt.

No hardcoded task types - fully generic design."

# Push and create PR
git push -u origin feat/prompt-builder
gh pr create --title "feat: Generic prompt builder" --body "..."
```

---

## 11. LLM Backend Strategy

### 11.1 Current Implementation: Ollama (MVP)

For rapid development and testing, agx-eval uses **Ollama** as the LLM backend:

**Advantages:**
- ✅ Quick to set up and test
- ✅ Model flexibility (any Ollama-compatible model)
- ✅ Simple HTTP API
- ✅ Platform agnostic
- ✅ Separation of concerns (Ollama handles GPU/optimization)

**Usage:**
```bash
# Start Ollama with a model
ollama pull qwen2.5:1.5b
ollama serve

# Use agx-eval
echo "data" | agx-eval --context "..." --prompt "..."
```

**Limitations:**
- ❌ Requires external Ollama service
- ❌ Not suitable for air-gapped environments
- ❌ Network dependency (even if localhost)

### 11.2 Future: Candle Backend (Air-gapped Deployment)

For **production deployment in secure/air-gapped environments**, agx-eval will support embedded inference using **Candle + GGUF models**.

**Advantages:**
- ✅ True standalone binary (AU + model in one package)
- ✅ No external dependencies at runtime
- ✅ Air-gapped compatible
- ✅ Deterministic (same model = same results)
- ✅ GPU acceleration (Metal, CUDA, etc.)

**Planned Architecture:**
```rust
// Future: Backend selection via CLI
--backend ollama              // Default (current)
--backend candle --model-path ~/models/specialized-eval.gguf

// Or environment variable
EVAL_BACKEND=candle
EVAL_MODEL_PATH=/opt/models/eval.gguf
```

**Use Cases:**
- **Development/Testing**: Use Ollama backend for flexibility
- **Production (air-gapped)**: Use Candle backend with fine-tuned specialized GGUF model
- **Hybrid**: Support both backends, user selects at runtime

### 11.3 Model Selection for Candle Backend

When implementing Candle backend, consider:

1. **Specialized fine-tuned models** for evaluation tasks
2. **Quantized GGUF format** (Q4_K_M or Q5_K_M) for size/speed balance
3. **Model size constraints**: 1-7B parameters ideal for AU deployment
4. **Task-specific variants**: Different models for different evaluation types

**Example deployment:**
```bash
# Single binary with embedded model
./agx-eval-candle-v1.bin \
  --model-path /bundled/eval-7b-q4.gguf \
  --context "..." \
  --prompt "..." < data.txt
```

### 11.4 Implementation Roadmap

**Phase 1 (Current):** Ollama backend only
- ✅ MVP-001: Prompt builder
- ✅ MVP-002: Ollama client
- ✅ MVP-003: Response parser
- 🔄 MVP-004: Stdin orchestration
- 🔄 MVP-005: Test suite
- 🔄 MVP-006: Error handling

**Phase 2 (Future):** Abstract backend interface
- Create `Backend` trait with `generate(prompt) -> Result<String>`
- Implement `OllamaBackend` (refactor current code)
- Add `--backend` CLI flag

**Phase 3 (Production):** Candle backend
- Implement `CandleBackend` using candle-core
- Support GGUF model loading
- GPU acceleration (Metal/CUDA)
- Bundle model with binary for air-gapped deployment

**Phase 4 (Advanced):** Multi-model support
- Model routing based on task complexity
- Fast model for simple evals, powerful for complex
- Model ensemble/voting for high-stakes decisions

---

## 12. References

- [Agentic Unit Specification](https://github.com/agenix-sh/agenix/blob/main/docs/au-specs/agentic-unit-spec.md)
- [AGEniX Issue #12: agx-eval spec](https://github.com/agenix-sh/agenix/issues/12)
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Remember**: agx-eval is a **generic LLM wrapper**. It has ZERO domain knowledge. All intelligence comes from user-provided context and prompts. Keep it generic, keep it simple, keep it composable.
