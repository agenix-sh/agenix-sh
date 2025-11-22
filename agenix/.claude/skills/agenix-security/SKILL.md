---
name: agenix-security
description: Apply AGEniX security guidelines including OWASP Top 10, Rust security patterns, command injection prevention, RESP protocol security, and zero-trust principles when reviewing or writing code
allowed-tools: Read, Grep, Glob, Bash
---

# AGEniX Security Skill

This skill enforces security best practices across the AGEniX ecosystem, with focus on the zero-trust execution model and Rust-specific security patterns.

## Canonical Documentation

- **Security Guidelines**: `/Users/lewis/work/agenix-sh/agenix/docs/development/security-guidelines.md`
- **Zero-Trust Model**: `/Users/lewis/work/agenix-sh/agenix/docs/zero-trust/zero-trust-execution.md`
- **Testing Strategy**: `/Users/lewis/work/agenix-sh/agenix/docs/development/testing-strategy.md`

## Zero-Trust Principles

### Workers Don't Generate Plans
```rust
// ❌ NEVER do this in AGW
fn execute_job(job: &Job) {
    let plan = call_llm_to_generate_plan();  // Security violation!
}

// ✅ CORRECT: Workers only execute
fn execute_job(job: &Job) {
    for task in &job.tasks {
        execute_task_sandboxed(task)?;  // Pre-approved plan only
    }
}
```

### Tools Are Untrusted Processes
- Run each tool as separate process (no shell)
- Enforce timeouts on all executions
- Limit resource usage (memory, CPU)
- Sanitize stdout/stderr before logging

## Command Injection Prevention

### NEVER Use Shell Execution
```rust
// ❌ DANGEROUS: Shell execution
Command::new("sh")
    .arg("-c")
    .arg(format!("grep {} file.txt", user_input))  // Injection risk!

// ✅ SAFE: Direct process execution
Command::new("grep")
    .arg(user_input)  // Safely passed as argument
    .arg("file.txt")
    .output()
```

### Validate All Inputs
```rust
fn validate_worker_id(id: &str) -> Result<()> {
    // Check format
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("Invalid worker ID: contains unsafe characters");
    }

    // Check length
    if id.len() > 64 {
        bail!("Worker ID too long");
    }

    // Block shell metacharacters
    if id.contains(&[';', '|', '&', '$', '`', '\n', '\r'][..]) {
        bail!("Worker ID contains shell metacharacters");
    }

    Ok(())
}
```

## RESP Protocol Security

### Session-Key Authentication (Constant-Time)
```rust
use subtle::ConstantTimeEq;

fn authenticate(provided: &[u8], expected: &[u8]) -> bool {
    // ✅ Constant-time comparison prevents timing attacks
    provided.ct_eq(expected).into()

    // ❌ NEVER use == for secrets (timing attack vulnerable)
    // provided == expected
}
```

### Input Validation
```rust
// Validate RESP message size
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;  // 10MB

fn parse_resp_message(buf: &[u8]) -> Result<RespMessage> {
    if buf.len() > MAX_MESSAGE_SIZE {
        bail!("RESP message too large: {}MB", buf.len() / 1024 / 1024);
    }

    // Parse safely...
}
```

## Path Traversal Prevention

```rust
use std::path::{Path, PathBuf};

fn validate_file_path(path: &str) -> Result<PathBuf> {
    // Reject path traversal attempts
    if path.contains("..") {
        bail!("Path traversal detected: {}", path);
    }

    // Canonicalize path
    let canonical = PathBuf::from(path).canonicalize()
        .context("Failed to canonicalize path")?;

    // Ensure within allowed directory
    let allowed = PathBuf::from("/allowed/workspace").canonicalize()?;
    if !canonical.starts_with(&allowed) {
        bail!("Path outside allowed directory: {:?}", canonical);
    }

    Ok(canonical)
}
```

## Integer Overflow Protection

```rust
// ✅ Use checked arithmetic for security-critical calculations
let total = count.checked_mul(size)
    .ok_or_else(|| anyhow!("Integer overflow in size calculation"))?;

// ❌ NEVER use unchecked arithmetic with user input
// let total = count * size;  // Can overflow!
```

## Resource Limits

```rust
const MAX_INPUT_SIZE: usize = 50 * 1024 * 1024;  // 50MB
const MAX_PLAN_SIZE: usize = 1 * 1024 * 1024;    // 1MB
const MAX_TASKS_PER_PLAN: usize = 100;
const MAX_TASK_TIMEOUT: Duration = Duration::from_secs(3600);  // 1 hour

fn validate_plan(plan: &Plan) -> Result<()> {
    if plan.tasks.len() > MAX_TASKS_PER_PLAN {
        bail!("Too many tasks: {} (max {})", plan.tasks.len(), MAX_TASKS_PER_PLAN);
    }

    for task in &plan.tasks {
        if task.timeout_secs > MAX_TASK_TIMEOUT.as_secs() {
            bail!("Task timeout too long: {}s", task.timeout_secs);
        }
    }

    Ok(())
}
```

## Security Testing Checklist

### Before Committing Code

- [ ] No user input flows to shell commands
- [ ] All inputs validated (size, format, content)
- [ ] Paths validated against traversal
- [ ] Constant-time comparison for secrets
- [ ] Integer overflow checks on arithmetic
- [ ] Resource limits enforced
- [ ] Timeouts on all I/O operations
- [ ] No secrets in logs or error messages
- [ ] RESP command injection tests written
- [ ] Authentication checks present

### Running Security Checks

```bash
# Security audit
cargo audit

# Clippy with security warnings
cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used

# Check for unsafe blocks
rg "unsafe" src/ --type rust

# Check for TODO/FIXME in security code
rg "TODO|FIXME" src/auth.rs src/session.rs
```

## OWASP Top 10 for AGEniX

1. **Broken Access Control** → Session-key authentication, worker registration validation
2. **Cryptographic Failures** → Use `ring` crate, constant-time comparisons
3. **Injection** → Never use shell, validate all inputs, parameterized commands
4. **Insecure Design** → Zero-trust model, workers can't generate plans
5. **Security Misconfiguration** → Explicit allowlists, deny by default
6. **Vulnerable Components** → `cargo audit`, pin dependencies
7. **Authentication Failures** → Session keys, no default credentials
8. **Software Integrity** → Plan signing (Phase 2), no unsigned code execution
9. **Logging Failures** → Log security events, never log secrets
10. **SSRF** → No network access from workers (future: network namespaces)

## When to Activate This Skill

Use this skill when:
- Reviewing PRs with authentication changes
- Implementing new RESP commands
- Adding input validation
- Working with session keys or secrets
- Designing worker sandboxing
- Writing security tests
- Auditing existing code for vulnerabilities

## Common Pitfalls to Avoid

❌ Using `unwrap()` or `expect()` in production code paths
❌ Concatenating strings to build commands
❌ Logging user input verbatim (may contain secrets)
❌ Using `==` to compare secrets
❌ Trusting file paths from user input
❌ Missing size limits on inputs
❌ Panicking instead of returning errors
❌ Using `unsafe` without extensive SAFETY comments

For comprehensive security details, use the Read tool to access the canonical security-guidelines.md and zero-trust-execution.md documents.
