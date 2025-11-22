---
name: agenix-testing
description: Apply AGEniX testing strategy including TDD, coverage requirements, security testing patterns, and test organization when writing or reviewing tests for agx, agq, agw, or AU components
allowed-tools: Read, Grep, Glob, Bash
---

# AGEniX Testing Skill

This skill enforces Test-Driven Development (TDD) practices and comprehensive testing strategies across all AGEniX components.

## Canonical Documentation

- **Testing Strategy**: `/Users/lewis/work/agenix-sh/agenix/docs/development/testing-strategy.md`
- **AU Testing**: `/Users/lewis/work/agenix-sh/agenix/docs/au-specs/testing-au.md`
- **Security Guidelines**: `/Users/lewis/work/agenix-sh/agenix/docs/development/security-guidelines.md`

## Coverage Requirements

| Component Type | Minimum Coverage | Required for |
|----------------|-----------------|--------------|
| All code | 80% | General development |
| Public APIs | 90% | Interfaces, exported functions |
| Security-critical | 100% | Auth, validation, cryptography |

### Security-Critical Code (100% Required)

- Authentication and authorization
- Session key validation
- Input sanitization
- RESP protocol parsing
- Command injection prevention
- Path traversal prevention
- Cryptographic operations
- Rate limiting

## TDD Workflow

### 1. Write Test First

```rust
#[test]
fn test_session_key_validation_rejects_short_keys() {
    let short_key = "abc";  // Less than 32 bytes
    assert!(validate_session_key(short_key).is_err());
}

#[test]
fn test_session_key_validation_accepts_valid_keys() {
    let valid_key = "a".repeat(32);  // Exactly 32 bytes
    assert!(validate_session_key(&valid_key).is_ok());
}
```

### 2. Implement Minimal Code

```rust
fn validate_session_key(key: &str) -> Result<()> {
    if key.len() < 32 {
        bail!("Session key too short");
    }
    Ok(())
}
```

### 3. Refactor

```rust
const MIN_SESSION_KEY_LENGTH: usize = 32;

fn validate_session_key(key: &str) -> Result<()> {
    if key.len() < MIN_SESSION_KEY_LENGTH {
        bail!(
            "Session key too short: {} bytes (minimum: {})",
            key.len(),
            MIN_SESSION_KEY_LENGTH
        );
    }
    Ok(())
}
```

## Test Organization

### Directory Structure

```
component/
├── src/
│   ├── lib.rs
│   ├── auth.rs
│   └── auth/
│       └── tests.rs          # Unit tests
├── tests/
│   ├── integration/
│   │   ├── auth_flow.rs
│   │   └── job_lifecycle.rs
│   └── security/
│       ├── injection_tests.rs
│       └── dos_tests.rs
└── fuzz/
    └── fuzz_targets/
        └── resp_parser.rs
```

### Test Naming Conventions

```rust
// ✅ Good: Descriptive, specific
#[test]
fn test_worker_heartbeat_updates_last_alive_timestamp() { }

#[test]
fn test_command_injection_with_semicolon_is_blocked() { }

// ❌ Bad: Generic, unclear
#[test]
fn test_worker() { }

#[test]
fn it_works() { }
```

## Test Types

### Unit Tests

Test individual functions in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worker_id_valid_format() {
        let id = "worker-001";
        assert!(validate_worker_id(id).is_ok());
    }

    #[test]
    fn test_parse_worker_id_rejects_special_chars() {
        let id = "worker; rm -rf /";
        assert!(validate_worker_id(id).is_err());
    }
}
```

### Integration Tests

Test component interactions:

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_full_authentication_flow() {
    let server = setup_test_server().await;

    // Connect client
    let mut client = RespClient::connect(server.addr()).await.unwrap();

    // Test AUTH command
    let response = client.auth("test_key_32_bytes_long_xxxxx").await;
    assert_eq!(response.unwrap(), "OK");

    // Test authenticated PING
    let pong = client.ping().await.unwrap();
    assert_eq!(pong, "PONG");
}
```

### Security Tests

Explicitly test attack vectors:

```rust
// tests/security/injection_tests.rs
#[test]
fn test_command_injection_prevention() {
    let malicious_inputs = vec![
        "worker; rm -rf /",
        "worker`cat /etc/passwd`",
        "worker$(whoami)",
        "worker|ls",
        "worker&& cat /etc/shadow",
    ];

    for input in malicious_inputs {
        let result = parse_worker_id(input);
        assert!(
            result.is_err(),
            "Should reject command injection: {}",
            input
        );
    }
}

#[test]
fn test_path_traversal_prevention() {
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\\\..\\\\..\\\\windows\\\\system32",
        "/etc/passwd",
    ];

    for path in malicious_paths {
        let result = validate_file_path(path);
        assert!(result.is_err(), "Should reject path traversal: {}", path);
    }
}
```

## Before Committing Checklist

### 1. Run All Tests

```bash
cargo test
```

### 2. Check Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --fail-under 80
```

### 3. Run Security Audit

```bash
cargo audit
```

### 4. Run Clippy

```bash
cargo clippy -- -D warnings
```

### 5. Format Code

```bash
cargo fmt
```

## Property-Based Testing

For complex validation logic:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_job_id_roundtrip(id in "[a-zA-Z0-9]{8,64}") {
        // Property: encoding then decoding returns original
        let encoded = encode_job_id(&id);
        let decoded = decode_job_id(&encoded).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn test_plan_validation_never_panics(
        tasks in proptest::collection::vec(any::<Task>(), 0..100)
    ) {
        // Property: validation should never panic
        let plan = Plan { tasks };
        let _ = validate_plan(&plan);  // Must not panic
    }
}
```

## Fuzzing

For parsers and deserializers:

```rust
// fuzz/fuzz_targets/resp_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parser must never panic, even with malformed input
    let _ = parse_resp_message(data);
});
```

```bash
# Run fuzzer
cargo +nightly fuzz run resp_parser -- -max_total_time=60
```

## CI/CD Testing

### GitHub Actions Workflow

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Format check
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Test
        run: cargo test

      - name: Security audit
        run: cargo audit

      - name: Coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --fail-under 80
```

## Component-Specific Guidelines

### AGQ Testing

- 100% coverage on authentication
- 100% coverage on RESP parser
- Fuzz RESP protocol parsing
- Test concurrent access
- Test TTL expiration

### AGW Testing

- Test all tool executions
- Test timeout enforcement
- Test stdout/stderr capture
- Test fail-fast behavior
- Mock external tool calls

### AU Testing

- `--describe` output validates against schema
- stdin/stdout contract tests
- Binary input handling
- Malformed input gracefully handled
- Security tests (oversized input, path traversal)

## When to Activate This Skill

Use this skill when:
- Writing new features (TDD: test first!)
- Reviewing PRs (check test coverage)
- Fixing bugs (write regression test first)
- Refactoring (tests prove correctness)
- Adding security-critical code (100% coverage required)
- Setting up CI/CD pipelines

## Quick Reference: Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_session_key_validation

# Run tests matching pattern
cargo test auth

# Run tests with output
cargo test -- --nocapture

# Run tests in specific file
cargo test --test integration_test

# Run benchmarks
cargo bench

# Run fuzz tests
cargo +nightly fuzz run resp_parser
```

For comprehensive testing details, reference the canonical testing-strategy.md document.
