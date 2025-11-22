# 0004. Dual MIT/Apache-2.0 Licensing with Separate Model License

**Date:** 2025-11-17
**Status:** Accepted
**Deciders:** AGX Core Team
**Tags:** licensing, legal, open-source, ai-models

---

## Context

AGEniX consists of multiple components with different licensing considerations:

### Software Components

- **Core infrastructure** (AGX, AGQ, AGW): Rust binaries, libraries, CLI tools
- **Agentic Units** (agx-ocr, future AUs): Standalone tools with AU contracts
- **Central documentation** (agenix repo): Architecture, specs, contracts

### AI Models (Current and Future)

- **Planning models** (Echo, Delta): Fine-tuned models for AGX planning
- **AU-embedded models** (e.g., DeepSeek-OCR in agx-ocr): Models bundled with AUs
- **Future distributed models**: Models downloaded/cached by users

### Requirements

1. **Maximize adoption**: Permissive license to encourage ecosystem growth
2. **Rust ecosystem compatibility**: Align with Rust community standards
3. **Patent protection**: Protect contributors and users from patent trolls
4. **GPLv2 compatibility**: Allow use in GPLv2 projects (if needed)
5. **Commercial-friendly**: Enable commercial use without barriers
6. **Model distribution clarity**: Separate licensing for AI model weights
7. **Attribution requirements**: Ensure credit to contributors

### Constraints

- Cannot use copyleft (GPL) - prevents commercial adoption
- Must be OSI-approved for core software
- Model licenses may have use-based restrictions (acceptable)
- Must be simple for contributors to understand

---

## Decision

**We will use dual MIT/Apache-2.0 licensing for all AGEniX software code, with separate model licenses for AI weights.**

### Software License: MIT OR Apache-2.0

All repositories containing code (Rust, scripts, tooling) will be dual-licensed:

```
Licensed under either of:
 * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
```

Users can choose which license to apply.

### Model License: Custom Per-Model

AI model weights will have their own licenses:

1. **Upstream model licenses** (e.g., DeepSeek Model License)
   - Preserved for models we use/redistribute
   - Documented in `MODEL-LICENSE` file alongside weights

2. **AGEniX-trained models** (future)
   - Custom license similar to DeepSeek: permissive with use-based restrictions
   - Separate from software license
   - Documented in `MODEL-LICENSE` in model repositories

### Documentation License: CC-BY-4.0 (Optional)

For pure documentation (guides, tutorials), we may use Creative Commons Attribution 4.0 International (CC-BY-4.0) to allow broader redistribution.

Core architecture docs remain under software license (MIT/Apache-2.0) as they're part of the product.

---

## Alternatives Considered

### Option 1: MIT Only

**Pros:**
- Simplest license (very short, easy to understand)
- Most permissive
- GPLv2 compatible
- Highest adoption on GitHub

**Cons:**
- No explicit patent grant
- No protection from patent trolls
- Less protective for contributors

### Option 2: Apache-2.0 Only

**Pros:**
- Explicit patent grant (protects users and contributors)
- Well-understood in enterprise
- Contributor license agreement built-in
- Strong in Rust ecosystem

**Cons:**
- GPLv2 incompatible (rare issue, but exists)
- Slightly more complex than MIT
- Longer license text

### Option 3: Dual MIT/Apache-2.0 (CHOSEN)

**Pros:**
- **Best of both worlds**: Users choose based on their needs
- **Rust ecosystem standard**: Aligns with Rust project itself
- **Maximum compatibility**: MIT for GPLv2 projects, Apache-2.0 for patent protection
- **Patent protection available**: Users opting for Apache-2.0 get patent grant
- **Simple for contributors**: Standard Rust contribution model

**Cons:**
- Two license files to maintain (minimal overhead)
- Slightly more complex explanation for new contributors

### Option 4: GPL/AGPL (Copyleft)

**Pros:**
- Forces derivatives to stay open-source
- Prevents proprietary forks

**Cons:**
- **Prevents commercial adoption** (major barrier)
- **Not compatible with Rust ecosystem norms**
- **Restricts downstream use** (against our goal of ecosystem growth)
- **Would kill commercial AU development**

### Option 5: Single Custom License

**Pros:**
- Tailored exactly to our needs

**Cons:**
- Not OSI-approved
- Legal review overhead
- Confuses contributors
- Reduces trust and adoption

### Decision Rationale

Dual MIT/Apache-2.0 chosen because:

1. **Rust ecosystem alignment**: This is the standard for Rust projects (used by `rustc`, `cargo`, most popular crates)
2. **Maximum compatibility**: Satisfies both GPLv2 and patent-protection needs
3. **Simple for contributors**: Well-understood, no CLA required
4. **Commercial-friendly**: Enables ecosystem growth and AU marketplace
5. **Patent protection available**: Apache-2.0 option provides explicit patent grant
6. **Proven model**: Thousands of successful Rust projects use this

---

## Consequences

### Positive

- **Broad adoption**: No licensing barriers for users
- **Commercial-friendly**: Companies can build proprietary AUs
- **Patent protection**: Available via Apache-2.0 option
- **Rust ecosystem fit**: Aligns with community norms
- **Simple contribution**: No CLA, standard license headers
- **Model flexibility**: Separate model licenses allow appropriate restrictions
- **Clear separation**: Software (permissive) vs Models (potentially restricted)

### Negative

- **Two licenses to maintain**: Slightly more overhead (mitigated by being standard)
- **License choice complexity**: Users must choose (mitigated by "either" language)
- **Model license complexity**: Different rules for software vs models (necessary trade-off)

### Neutral

- **No copyleft**: Proprietary forks are possible (but we accept this trade-off)
- **Attribution requirement**: Both licenses require attribution (standard practice)

---

## Implementation Notes

### Repository Structure

Each code repository gets:

```
repo/
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md (explains dual licensing)
└── src/
    └── lib.rs (standard license header)
```

### License Headers

Standard Rust license header in source files:

```rust
// Licensed under either of
//
// * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
// * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
//
// at your option.
```

### Contribution Guidelines

Standard Rust contribution clause in CONTRIBUTING.md:

```markdown
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
```

### Model Licensing

Repositories containing models:

```
agx-ocr/
├── LICENSE-MIT (for code)
├── LICENSE-APACHE (for code)
├── MODEL-LICENSE (for model weights)
├── models/
│   ├── deepseek-ocr/
│   │   ├── README.md (cites upstream DeepSeek license)
│   │   └── LICENSE (DeepSeek Model License)
│   └── tinker-delta-1.5b/ (future AGEniX-trained model)
│       ├── README.md (cites AGEniX Model License)
│       └── LICENSE (AGEniX Model License)
```

### Model License Template (Future AGEniX-Trained Models)

When we train our own models, use a permissive license with use-based restrictions:

**Inspiration**: DeepSeek Model License

**Key terms**:
- Commercial use: Permitted
- Modification: Permitted
- Redistribution: Permitted with attribution
- Patent grant: Included
- Use restrictions: Prohibit illegal or harmful activities
- Attribution: Required
- Copyleft: No (derivatives can use different licenses)

### README Licensing Section

Each repo README.md should include:

```markdown
## License

This project is dual-licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### AI Model Weights

AI model weights (if included) are licensed separately. See [MODEL-LICENSE](MODEL-LICENSE) for details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
```

---

## Model Distribution Strategy

### Bundled Models (e.g., agx-ocr + DeepSeek)

**Approach**: Include model weights in repository with upstream license

**Files**:
- Code: MIT/Apache-2.0
- Model: DeepSeek Model License (preserved)
- Clear README explaining license split

**User agreement**: Implied acceptance when downloading/using

### Downloaded Models (Future)

**Approach**: Prompt user to accept model license before download

**Flow**:
```bash
$ agx model install tinker-delta-1.5b
This model is licensed under the AGEniX Model License.
Key terms:
  - Commercial use permitted
  - Attribution required
  - Prohibited: illegal/harmful activities

View full license: https://agenix.sh/licenses/model
Accept? [y/N]: y
Downloading...
```

**Storage**:
- Models cached in `~/.agx/models/`
- License stored alongside: `~/.agx/models/<model-name>/LICENSE`

---

## Related Decisions

- Contribution guidelines flow from this decision
- AU marketplace licensing will build on this foundation
- Future ADR: AGEniX Model License specification (when we train models)

---

## References

- [Rust Project Licensing](https://www.rust-lang.org/policies/licenses)
- [Rust API Guidelines: Licensing](https://rust-lang.github.io/api-guidelines/necessities.html)
- [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/licenses/MIT)
- [DeepSeek Model License](https://github.com/deepseek-ai/DeepSeek-V3/blob/main/LICENSE-MODEL)
- [Dual Licensing Rationale](https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/8952)
- [OSI Approved Licenses](https://opensource.org/licenses)
