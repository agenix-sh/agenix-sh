# Tool Contracts and CLI Conventions

This document describes how tools must behave to participate in AGEnix workflows.

## CLI Behaviour

- **Input**: tools read from stdin (text or binary).
- **Output**: tools write primary results to stdout.
- **Errors/logging**: tools write diagnostics to stderr.
- **Exit codes**:
  - 0 = success
  - non-zero = failure

## `--describe` Contract

All tools must implement:

```bash
tool-name --describe
```

which prints a machine-readable model card describing:

- name and version
- capabilities
- input and output formats
- configuration options
- resource requirements

The expected schema is defined in `/specs/describe.schema.json`.
