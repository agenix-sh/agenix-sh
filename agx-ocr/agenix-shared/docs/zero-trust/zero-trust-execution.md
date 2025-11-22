# Zero-Trust Execution Model

AGEnix is designed with a zero-trust mindset.

Core principles:

- Workers (`agw`) do not generate or modify plans; they only execute pre-approved, signed plans.
- Tools are treated as untrusted processes and are invoked via stdin/stdout/stderr.
- Plans are signed by `agx` and verified by `agw` before execution.
- Workers do not have direct access to LLM providers.
- The set of allowed tools is explicitly configured and auditable.

This document will evolve to include detailed threat models and mitigation strategies.
