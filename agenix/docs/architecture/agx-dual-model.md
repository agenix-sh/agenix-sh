# AGX Dual-Model Planning Architecture  
**Echo (Intent Interpreter) + Delta (Plan Compiler)**  
AGEniX Meta Repository

AGX uses a *two-model* planning architecture that separates  
❶ **understanding and clarifying human intent** (Echo) from  
❷ **compiling deterministic machine-executable plans** (Delta).

This enables:
- Strong correctness guarantees  
- Zero-trust execution  
- Deterministic plan compilation  
- Easier fine-tuning  
- Modular upgrades of reasoning + planning components

---

# 1. Motivation

Human intent is:
- ambiguous  
- incomplete  
- highly variable  
- not directly executable  

AGX plans must be:
- deterministic  
- verifiable  
- minimal  
- safe  
- strictly JSON-schema compliant  

These are **orthogonal cognitive modes**, so AGX adopts a **dual-model architecture**, mirroring classic CS concepts:

```
Interpreter (Echo) → Intermediate Representation → Compiler (Delta) → Execution
```

---

# 2. Echo → Delta Pipeline

```
User Intent
    │
    ▼
(ECHO) Intent Interpreter
 - reflect user intent
 - clarify ambiguities
 - expose hidden assumptions
 - request missing information
 - produce a machine-friendly Structured Intent

Structured Intent (Intermediate Representation)
    │
    ▼
(DELTA) Plan Compiler
 - select appropriate AUs/tools
 - produce minimal executable steps
 - validate argument schemas
 - enforce zero-trust constraints
 - output valid JSON plan

Deterministic Plan
    │
    ▼
AGX Executor / AGQ / AGW
 (zero-trust sequential execution)
```

Echo **understands**.  
Delta **executes**.

---

# 3. Echo Model (Intent Interpreter)

Echo’s function is **interpretation**, not planning.

## Echo Responsibilities
- Understand natural language  
- Clarify vague user requirements  
- Ask for missing constraints or parameters  
- Break down high-level tasks into conceptual substeps  
- Validate that a request is safe / plannable  
- Produce **Structured Intent**, a clean, IR-style specification  

## Echo Characteristics
- conversational  
- flexible  
- high reasoning bandwidth  
- tolerant of ambiguity  
- temperature up to 0.7  
- may use chain-of-thought internally  
- assists the user interactively  

## Echo Output Example

User:
> “Remove duplicates and then find lines mentioning invoices.”

Echo Output (Structured Intent):

```json
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

This is **not** an executable plan.  
It is a structured description of human intentions.

---

# 4. Structured Intent (Intermediate Representation)

This IR is the **contract** between Echo and Delta.

Properties:
- machine-readable  
- human-auditable  
- stable across versions  
- independent of specific tools  
- not yet executable  

Why it matters:
- Allows Echo to iterate with the user  
- Allows Delta to be smaller and more deterministic  
- Enables training Echo and Delta separately  
- Improves maintainability  

---

# 5. Delta Model (Plan Compiler)

Delta’s job is **strict, deterministic plan generation**.

## Delta Responsibilities
- Consume Structured Intent  
- Select appropriate tools (AUs) from registry  
- Generate an execution plan matching `/specs/plan.schema.json`  
- Ensure:
  - no hallucinated tools  
  - no invalid arguments  
  - no unsafe shell commands  
- Produce **minimal** sequential plans  
- Temperature **0.0**  
- Deterministic for same inputs  

## Delta Example

Input (Structured Intent):
```json
{
  "task": "text-filtering",
  "steps": [
    {"action": "remove-duplicates"},
    {"action": "grep", "pattern": "invoice"}
  ]
}
```

Delta Output (AGX Plan):
```json
{
  "version": "1.0",
  "steps": [
    {
      "id": "dedupe",
      "tool": "uniq",
      "args": []
    },
    {
      "id": "filter",
      "tool": "grep",
      "args": ["invoice"]
    }
  ]
}
```

---

# 6. Why Two Models?

| Function | Echo | Delta |
|----------|-------|--------|
| Understand human language | ✔️ | ❌ |
| Clarify ambiguous requests | ✔️ | ❌ |
| Decompose tasks | ✔️ | ❌ |
| Follow schemas | ❌ | ✔️ |
| Strict JSON | ❌ | ✔️ |
| Deterministic output | ❌ | ✔️ |
| Creative reasoning | ✔️ | ❌ |
| Safe tool invocation | ❌ | ✔️ |

A single model cannot achieve both without:
- hallucinations  
- invalid JSON  
- fragile planning  
- broken determinism  

This architecture is mirrored in:
- Microsoft’s **Planner → Executor**  
- Anthropic’s **Toolformer**  
- Stanford ACE  
- Google's **Thinking → Acting** separation  

---

# 7. Echo + Delta Naming Rationale

Borrowed lightly from “Forward-Deployed Engineers” without militaristic baggage:

## Echo
- reflects user intent  
- gathers requirements  
- ensures correctness before compiling  
- establishes shared understanding  

## Delta
- applies the transformation  
- turns intent into actionable steps  
- enforces precision, safety, determinism  
- is the “agent of change” that moves from intent → reality  

The metaphor is functional, not hierarchical.

---

# 8. Placement in the AGEniX Architecture

Echo and Delta sit entirely within **AGX**, not AGQ/AGW.

```
+--------+     +---------+     +---------+     +----------+
| Human  | --> |  Echo   | --> |  Delta  | --> | Executor |
+--------+     +---------+     +---------+     +----------+
                     |                |             
             Structured Intent   AGX Plan (JSON)
```

AGQ and AGW never talk to Echo or Delta.  
They only consume **compiled AGX Plans**.

This ensures:
- zero LLMs in workers  
- deterministic execution  
- auditability  
- replayability  

---

# 9. Training & Fine-Tuning Strategy (Tinker)

## Echo Fine-tuning Corpus
Examples of:
- messy user intent → structured intent  
- ambiguous tasks → disambiguated forms  
- multi-turn clarifications  
- requirement gathering  

Ideal model: **VibeThinker-1.5B**, DeepSeek-R1-Distill.

## Delta Fine-tuning Corpus
Examples of:
- structured intent → AGX plans  
- tool registry-driven planning  
- strict schema adherence  

Ideal model: **Phi-4-Mini**, Qwen2.5 1.5B-Instruct, Llama-3.2-3B-Instruct.

---

# 10. Future Extensions

- **Plan Verifier AU**  
  Validate AGX plan correctness before execution.

- **Plan Repair Loop**  
  Delta → Verifier → Delta (until valid).

- **Echo-Delta Negotiation Layer**  
  Echo can highlight missing info; Delta can ask Echo for clarifications.

- **Contextual Executors**  
  Different Delta models per tool domain (CV pipelines, DB pipelines, doc pipelines).

---

# 11. Repository Placement

Place this document at:

```
agenix/
  docs/
    architecture/
      agx-dual-model.md
```

Child repos (`agx`, `agq`, `agw`, `agx-ocr`, etc.) should reference it, not duplicate it.

---

**End of document.**
