# 0003. Dual-Model Planning (Echo + Delta)

**Date:** 2025-11-17
**Status:** Accepted
**Deciders:** AGX Core Team
**Tags:** architecture, planning, llm, ai, agx

---

## Context

AGX needs to convert natural-language user intent into deterministic, executable JSON plans. This transformation involves two orthogonal cognitive tasks:

1. **Understanding human intent**: Interpreting ambiguous, incomplete, variable natural language
2. **Generating executable plans**: Producing deterministic, schema-compliant, minimal JSON

### Requirements

- **Correctness**: Plans must be valid JSON, matching strict schemas
- **Determinism**: Same intent should produce same plan (or controlled variations)
- **Safety**: No hallucinated tools, no invalid arguments, no unsafe commands
- **Clarity**: User should understand what will happen before execution
- **Interactivity**: Ability to clarify ambiguous requests with user
- **Auditability**: Plans must be inspectable and verifiable

### Problem with Single-Model Approach

A single LLM trying to do both tasks simultaneously leads to:
- **Hallucinations**: Making up tools that don't exist
- **Schema violations**: Invalid JSON, missing required fields
- **Non-determinism**: Different outputs for same input
- **Brittle planning**: Small prompt changes cause large plan changes
- **Mixed concerns**: Reasoning and formatting entangled

---

## Decision

**We will use a two-model architecture: Echo (intent interpreter) + Delta (plan compiler).**

This mirrors classic compiler design:
```
Source Code → Parser → IR → Optimizer → Compiler → Machine Code
User Intent → Echo → Structured Intent → Delta → JSON Plan
```

### Echo Model (Intent Interpreter)

**Role**: Understand and clarify human intent

**Characteristics**:
- Conversational, flexible
- High reasoning bandwidth
- Tolerant of ambiguity
- Temperature: 0.5-0.7
- May use chain-of-thought
- Interacts with user to clarify

**Input**: Natural language user intent
**Output**: Structured Intent (intermediate representation)

**Example**:
```
User: "Remove duplicates and find lines mentioning invoices"

Echo Output (Structured Intent):
{
  "task": "text-filtering",
  "steps": [
    {"action": "remove-duplicates"},
    {"action": "grep", "pattern": "invoice"}
  ],
  "constraints": [],
  "output": "stdout"
}
```

### Delta Model (Plan Compiler)

**Role**: Generate deterministic executable plans

**Characteristics**:
- Strict, deterministic
- Schema-compliant output
- Temperature: 0.0
- No hallucinations
- Minimal, safe plans
- Tool registry-driven

**Input**: Structured Intent
**Output**: Valid JSON Plan (per `/specs/plan.schema.json`)

**Example**:
```
Delta Output (AGX Plan):
{
  "plan_id": "uuid-5678",
  "plan_description": "Filter text: deduplicate and find invoices",
  "tasks": [
    {
      "task_number": 1,
      "command": "uniq",
      "args": [],
      "timeout_secs": 30
    },
    {
      "task_number": 2,
      "command": "grep",
      "args": ["invoice"],
      "input_from_task": 1,
      "timeout_secs": 30
    }
  ]
}
```

---

## Alternatives Considered

### Option 1: Single Large Model (GPT-4 class)

**Pros:**
- Simpler architecture (one model)
- Can handle both understanding and formatting
- Fewer moving parts

**Cons:**
- Still prone to hallucinations
- Non-deterministic (even at temp=0)
- Schema violations require retry loops
- Expensive to run
- Difficult to fine-tune for both tasks
- Mixed reasoning and formatting

### Option 2: Prompt Engineering Only (Single Model + Complex Prompts)

**Pros:**
- No architectural complexity
- Fast to iterate
- Works with existing models

**Cons:**
- Fragile (prompt changes break plans)
- Still non-deterministic
- Hallucinations persist
- Hard to guarantee schema compliance
- Difficult to maintain as requirements grow

### Option 3: Rule-Based Planning (No LLM)

**Pros:**
- Perfectly deterministic
- No hallucinations
- Fast execution
- Predictable behavior

**Cons:**
- Cannot handle natural language
- Requires precise, structured input
- Inflexible (no intent understanding)
- Poor user experience

### Option 4: Three-Model Pipeline (Intent → Reasoning → Planning → Verification)

**Pros:**
- Even more separation of concerns
- Dedicated verifier model

**Cons:**
- Overly complex
- Higher latency (3 LLM calls)
- More points of failure
- Harder to maintain

### Decision Rationale

Dual-model chosen because:

1. **Separation of concerns**: Each model does one thing well
2. **Proven pattern**: Mirrors compiler architecture (parser → compiler)
3. **Determinism**: Delta at temp=0 produces consistent plans
4. **Safety**: Delta validates against tool registry, no hallucinations
5. **Maintainability**: Can upgrade/fine-tune Echo and Delta independently
6. **User experience**: Echo handles clarification naturally
7. **Industry precedent**: Microsoft Planner/Executor, Anthropic Toolformer, Google Thinking/Acting

Compared to alternatives:
- **vs single model**: More reliable, less hallucination
- **vs prompt engineering**: More maintainable, more deterministic
- **vs rule-based**: Handles natural language
- **vs three-model**: Simpler, lower latency

---

## Consequences

### Positive

- **Correctness**: Delta produces schema-valid plans 99%+ of time
- **Determinism**: Same structured intent → same plan
- **Safety**: Tool registry prevents hallucinated tools
- **Clarity**: Structured Intent is human-readable intermediate state
- **Modularity**: Can swap Echo or Delta models independently
- **Fine-tuning**: Can train small specialized models for each role
- **Debugging**: Clear separation makes issues easier to diagnose

### Negative

- **Complexity**: Two models to manage instead of one
- **Latency**: Two sequential LLM calls (mitigated by using fast models)
- **Development overhead**: Need to define Structured Intent schema
- **More moving parts**: Two models can fail independently

### Neutral

- **Model selection**: Need to choose appropriate models for each role
- **Structured Intent design**: Need stable IR schema (but provides benefits)

---

## Implementation Notes

### Phase 1: Foundation

**Echo Model Options**:
- DeepSeek-R1-Distill-Qwen-1.5B (reasoning-capable, small)
- VibeThinker-1.5B (optimized for intent understanding)
- Qwen2.5-3B-Instruct (strong reasoning)

**Delta Model Options**:
- Phi-4-Mini (3.8B, strong schema following)
- Qwen2.5-1.5B-Instruct (fast, good at structured output)
- Llama-3.2-3B-Instruct (reliable, well-tested)

**Structured Intent Schema**:
- Define stable JSON schema for Echo → Delta contract
- Version the schema (allow evolution)
- Document in `/specs/structured-intent.schema.json`

### Phase 2: Fine-Tuning (Tinker)

**Echo Fine-Tuning Corpus**:
- Messy user intent → structured intent examples
- Multi-turn clarification dialogues
- Ambiguity resolution patterns

**Delta Fine-Tuning Corpus**:
- Structured intent → AGX plan examples
- Tool registry-driven planning
- Schema compliance examples

### Phase 3: Optimization

**Caching**:
- Cache Echo outputs for common intents
- Cache Delta plans for common structured intents

**Fast Path**:
- Simple intents bypass Echo (direct to Delta)
- Deterministic mode skips Echo entirely

### Tool Registry Integration

Delta queries tool registry via `--describe` contracts:
```bash
$ agx-ocr --describe
{
  "name": "agx-ocr",
  "version": "0.1.0",
  "capabilities": ["document-ocr", "image-to-text"],
  "inputs": ["image/png", "image/jpeg", "application/pdf"],
  "outputs": ["text/plain", "application/json"]
}
```

---

## Related Decisions

- Related to execution architecture: Plans generated here are executed by AGW
- Related to AU contracts: Delta uses `--describe` to discover capabilities
- Future ADR: Structured Intent schema versioning

---

## References

- [AGX Dual-Model Planning Documentation](../architecture/agx-dual-model.md)
- [Anthropic: Toolformer Pattern](https://www.anthropic.com/research)
- [Microsoft: Planner-Executor Architecture](https://www.microsoft.com/research/)
- [Stanford ACE: Agent-Computer Environment](https://arxiv.org/abs/2304.12244)
- [Job Schema](../architecture/job-schema.md) - Delta output format
- [Agentic Unit Spec](../au-specs/agentic-unit-spec.md) - Tool contract spec
