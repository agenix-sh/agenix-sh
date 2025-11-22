# AGEnix Architecture Overview

AGEnix is an ecosystem of small, composable tools and services that bring Agent Oriented Architecture (AOA) principles to the command line.

At a high level:

- `agx` provides **planning and orchestration** for CLI-centric workflows.
- `agq` provides a **queue/scheduler** for distributed execution.
- `agw` provides **zero-trust workers** that run approved plans.
- AU tools like `agx-ocr` provide **specialist capabilities** that can be used standalone or inside plans.

Core pattern:

```text
input (stdin) → agx [intent] → plan (JSON) → execution (local or via agq/agw) → output (stdout)
```

This repo is the source of truth for how these components fit together.
