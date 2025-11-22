# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the AGEniX ecosystem.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences.

## Format

We use a lightweight ADR format inspired by Michael Nygard's template:

- **Title**: Short noun phrase
- **Status**: Proposed | Accepted | Deprecated | Superseded
- **Context**: What is the issue that we're seeing that is motivating this decision or change?
- **Decision**: What is the change that we're actually proposing or doing?
- **Consequences**: What becomes easier or more difficult to do because of this change?

## Naming Convention

ADRs are numbered sequentially: `NNNN-title-with-dashes.md`

Example: `0001-resp-protocol.md`

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [0001](./0001-resp-protocol.md) | Use RESP Protocol for Component Communication | Accepted | 2025-11-17 |
| [0002](./0002-embedded-redb.md) | Use Embedded redb for AGQ Storage | Accepted | 2025-11-17 |
| [0003](./0003-dual-model-planning.md) | Dual-Model Planning (Echo + Delta) | Accepted | 2025-11-17 |
| [0004](./0004-licensing-strategy.md) | Dual MIT/Apache-2.0 Licensing with Separate Model License | Accepted | 2025-11-17 |

## Creating a New ADR

1. Copy `template.md` to a new file with the next sequential number
2. Fill in the sections
3. Submit as pull request for review
4. Update this README index when merged

## When to Write an ADR

Write an ADR when making a decision that:
- Is architecturally significant
- Affects multiple components
- Has long-term consequences
- Involves trade-offs between alternatives
- Would be hard to reverse later

## When NOT to Write an ADR

Don't write an ADR for:
- Implementation details (use code comments)
- Temporary decisions (use TODO comments)
- Obvious choices with no alternatives
- Decisions easily reversed

---

**Maintained by:** AGX Core Team
**Review cycle:** ADRs are immutable once accepted; create new ADRs to supersede old ones
