# Testing Strategy

**Version:** 1.0
**Status:** Canonical Development Guidelines
**Last Updated:** 2025-11-17

This document defines the testing strategy for the AGEniX ecosystem.

---

## Table of Contents

1. [Philosophy](#philosophy)
2. [Test Coverage Requirements](#test-coverage-requirements)
3. [Test Types](#test-types)
4. [Test Organization](#test-organization)
5. [Testing Workflow](#testing-workflow)
6. [Continuous Integration](#continuous-integration)
7. [Best Practices](#best-practices)

---

## 1. Philosophy

### Test-Driven Development (TDD)

AGEniX follows strict TDD principles:

1. **Write tests first** - Before implementing functionality
2. **Implement minimal code** - Just enough to pass tests
3. **Refactor** - Improve code while maintaining tests
4. **Repeat** - For every feature and bug fix

### Why TDD?

- **Correctness**: Tests define expected behavior before implementation
- **Safety**: Regression protection for security-critical code
- **Documentation**: Tests serve as executable specifications
- **Confidence**: Refactor fearlessly with comprehensive test coverage
- **Design**: Writing tests first leads to better API design

---

## 2. Test Coverage Requirements

### Minimum Coverage

| Component | Minimum Coverage | Target Coverage |
|-----------|------------------|-----------------|
| All code | 80% | 90% |
| Public APIs | 90% | 100% |
| Security-critical code | 100% | 100% |
| Authentication | 100% | 100% |
| Input validation | 100% | 100% |
| Cryptographic operations | 100% | 100% |

### Security-Critical Code

The following **must have 100% test coverage**:

- Authentication and authorization
- Session key validation
- Input sanitization and validation
- RESP protocol parsing
- Command injection prevention
- Path traversal prevention
- Cryptographic operations
- Rate limiting

### Measuring Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html

# CI: Require minimum coverage
cargo tarpaulin --fail-under 80
```

---

## 3. Test Types

### 3.1 Unit Tests

**Purpose**: Test individual functions and methods in isolation

**Location**: Co-located with source code in `#[cfg(test)]` modules

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_validation() {
        // Test valid key
        let valid_key = "a".repeat(32);
        assert!(validate_session_key(&valid_key).is_ok());

        // Test invalid keys
        assert!(validate_session_key("").is_err());
        assert!(validate_session_key("short").is_err());
        assert!(validate_session_key("../etc/passwd").is_err());
    }

    #[test]
    fn test_worker_id_format() {
        assert!(validate_worker_id("worker-001").is_ok());
        assert!(validate_worker_id("worker_test_123").is_ok());

        // Invalid characters
        assert!(validate_worker_id("worker; rm -rf /").is_err());
        assert!(validate_worker_id("worker/../etc").is_err());
    }
}
```

**Best Practices**:
- One assertion concept per test
- Descriptive test names
- Test both success and failure cases
- Test edge cases and boundaries

---

### 3.2 Integration Tests

**Purpose**: Test component interactions and full workflows

**Location**: `tests/` directory at repository root

**Example**:
```rust
// tests/integration_test.rs
use agq::server::Server;
use agq::client::RespClient;

#[tokio::test]
async fn test_full_authentication_flow() {
    // Start test server
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    tokio::spawn(server.run());

    // Connect client
    let mut client = RespClient::connect(addr).await.unwrap();

    // Test AUTH command
    let response = client.auth("test_session_key_32_bytes_long!!").await;
    assert_eq!(response.unwrap(), "OK");

    // Test authenticated PING
    let pong = client.ping().await.unwrap();
    assert_eq!(pong, "PONG");
}

#[tokio::test]
async fn test_job_lifecycle() {
    // Test: plan submit → job create → worker pull → execute → complete
    let server = setup_test_server().await;

    // Submit plan
    let plan_id = submit_test_plan(&server).await;

    // Create job
    let job_id = create_job(&server, plan_id).await;

    // Worker pulls job
    let job = worker_pull_job(&server).await;
    assert_eq!(job.job_id, job_id);

    // Execute and report completion
    execute_job(&server, &job).await;

    // Verify job status
    let status = get_job_status(&server, job_id).await;
    assert_eq!(status, "completed");
}
```

**Best Practices**:
- Set up and tear down test infrastructure
- Test realistic workflows
- Use helper functions to reduce boilerplate
- Clean up resources (connections, temp files)

---

### 3.3 Security Tests

**Purpose**: Verify protection against common attack vectors

**Location**: `tests/security/` directory

**Example**:
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
        "..\\..\\..\\windows\\system32",
        "file:///etc/passwd",
        "/etc/passwd",
        "~/.ssh/id_rsa",
    ];

    for path in malicious_paths {
        let result = validate_file_path(path);
        assert!(result.is_err(), "Should reject path traversal: {}", path);
    }
}

#[test]
fn test_session_key_constant_time_comparison() {
    use std::time::Instant;

    let key1 = "a".repeat(32);
    let key2 = "b".repeat(32);
    let key3 = "a".repeat(32);

    // Measure timing for matching keys
    let start = Instant::now();
    let _ = compare_session_keys(&key1, &key3);
    let match_duration = start.elapsed();

    // Measure timing for non-matching keys
    let start = Instant::now();
    let _ = compare_session_keys(&key1, &key2);
    let nomatch_duration = start.elapsed();

    // Timing should be similar (within 10%)
    let ratio = match_duration.as_nanos() as f64 / nomatch_duration.as_nanos() as f64;
    assert!(
        (0.9..=1.1).contains(&ratio),
        "Timing attack vulnerability: ratio = {}",
        ratio
    );
}

#[test]
fn test_dos_protection_rate_limiting() {
    let mut limiter = RateLimiter::new(5, Duration::from_secs(60));

    // First 5 requests should succeed
    for i in 0..5 {
        assert!(limiter.check("192.168.1.1").is_ok(), "Request {} should succeed", i);
    }

    // 6th request should fail (rate limited)
    assert!(limiter.check("192.168.1.1").is_err(), "Should be rate limited");
}
```

**Best Practices**:
- Test all OWASP Top 10 attack vectors
- Test boundary conditions
- Test timing attack resistance
- Test resource exhaustion scenarios

---

### 3.4 Fuzzing Tests

**Purpose**: Discover edge cases through randomized input

**Location**: `fuzz/fuzz_targets/` directory

**Setup**:
```bash
cargo install cargo-fuzz
cargo fuzz init
```

**Example**:
```rust
// fuzz/fuzz_targets/resp_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use agq::protocol::parse_resp_message;

fuzz_target!(|data: &[u8]| {
    // Parser should never panic, even with malformed input
    let _ = parse_resp_message(data);
});
```

**Running Fuzz Tests**:
```bash
# Run fuzzer for 60 seconds
cargo +nightly fuzz run resp_parser -- -max_total_time=60

# Run with specific corpus
cargo +nightly fuzz run resp_parser fuzz/corpus/resp_parser

# Check coverage
cargo +nightly fuzz coverage resp_parser
```

**Best Practices**:
- Fuzz all parsers and deserializers
- Run fuzz tests in CI for at least 60 seconds
- Maintain corpus of interesting inputs
- Fix all panics and crashes discovered

---

### 3.5 Property-Based Tests

**Purpose**: Test properties that should hold for all inputs

**Dependencies**:
```toml
[dev-dependencies]
proptest = "1.0"
```

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_job_id_roundtrip(id in "[a-zA-Z0-9]{8,64}") {
        // Property: encoding then decoding should return original
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
        let _ = validate_plan(&plan);  // Should not panic
    }

    #[test]
    fn test_session_key_validation_consistent(
        key in "[a-zA-Z0-9]{32,128}"
    ) {
        // Property: validation should be consistent
        let result1 = validate_session_key(&key);
        let result2 = validate_session_key(&key);
        assert_eq!(result1.is_ok(), result2.is_ok());
    }
}
```

**Best Practices**:
- Define invariants as properties
- Test serialization round-trips
- Test mathematical properties
- Use reasonable input generators

---

### 3.6 Chaos/Failure Tests

**Purpose**: Test resilience to failures and unexpected conditions

**Example**:
```rust
#[tokio::test]
async fn test_network_partition_recovery() {
    let server = setup_test_server().await;
    let mut worker = Worker::connect(&server).await.unwrap();

    // Simulate network partition
    server.drop_all_connections().await;

    // Worker should detect failure and reconnect
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Worker should have reconnected
    assert!(worker.is_connected().await);
}

#[tokio::test]
async fn test_job_timeout_handling() {
    let server = setup_test_server().await;

    // Create job with very short timeout
    let job = Job {
        tasks: vec![Task {
            command: "sleep 10".into(),
            timeout_secs: 1,
            ..Default::default()
        }],
        ..Default::default()
    };

    // Execute job
    let result = execute_job(&server, &job).await;

    // Should fail with timeout error
    assert!(matches!(result, Err(JobError::Timeout)));
}

#[test]
fn test_out_of_memory_handling() {
    // Attempt to allocate huge plan
    let huge_plan = Plan {
        tasks: vec![Task::default(); 1_000_000],
        ..Default::default()
    };

    // Should fail gracefully, not OOM
    let result = validate_plan(&huge_plan);
    assert!(result.is_err());
}
```

---

## 4. Test Organization

### Directory Structure

```
component/
├── src/
│   ├── lib.rs
│   ├── auth.rs
│   └── auth/
│       └── tests.rs          # Unit tests for auth module
├── tests/
│   ├── integration/
│   │   ├── auth_flow.rs
│   │   ├── job_lifecycle.rs
│   │   └── worker_heartbeat.rs
│   └── security/
│       ├── injection_tests.rs
│       ├── dos_tests.rs
│       └── timing_tests.rs
└── fuzz/
    └── fuzz_targets/
        ├── resp_parser.rs
        └── plan_deserializer.rs
```

### Test Modules

**Co-located unit tests**:
```rust
// src/auth.rs
pub fn validate_session_key(key: &str) -> Result<(), Error> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_session_key() {
        // Tests here
    }
}
```

**Separate test modules**:
```rust
// src/auth/tests.rs (for larger test suites)
use super::*;

#[test]
fn test_complex_auth_scenario() {
    // Complex test
}
```

---

## 5. Testing Workflow

### Before Committing

```bash
# 1. Format code
cargo fmt

# 2. Check for warnings
cargo clippy -- -D warnings

# 3. Run all tests
cargo test

# 4. Run security audit
cargo audit

# 5. Check test coverage
cargo tarpaulin --fail-under 80
```

### CI Pipeline

Tests run automatically on:
- Every push to PR
- Every commit to main
- Nightly (including fuzzing)

**CI must pass before merge.**

---

## 6. Continuous Integration

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

  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust nightly
        uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Fuzz test
        run: |
          cargo +nightly fuzz run resp_parser -- -max_total_time=60
```

---

## 7. Best Practices

### Test Naming

Use descriptive names:

**Good**:
```rust
#[test]
fn test_session_key_rejects_empty_string() { }

#[test]
fn test_worker_heartbeat_updates_alive_timestamp() { }

#[test]
fn test_command_injection_with_semicolon_is_blocked() { }
```

**Bad**:
```rust
#[test]
fn test1() { }

#[test]
fn test_key() { }

#[test]
fn it_works() { }
```

### Test Independence

Tests must be independent and order-agnostic:

**Good**:
```rust
#[test]
fn test_a() {
    let db = create_test_db();  // Fresh DB per test
    // Test A
}

#[test]
fn test_b() {
    let db = create_test_db();  // Fresh DB per test
    // Test B
}
```

**Bad**:
```rust
static mut SHARED_STATE: i32 = 0;

#[test]
fn test_a() {
    unsafe { SHARED_STATE = 1; }  // Tests depend on order
}

#[test]
fn test_b() {
    unsafe { assert_eq!(SHARED_STATE, 1); }  // Breaks if run alone
}
```

### Test Data

Use test fixtures and builders:

```rust
// tests/fixtures.rs
pub fn test_plan() -> Plan {
    Plan {
        plan_id: "test-plan-001".into(),
        tasks: vec![
            Task {
                task_number: 1,
                command: "echo".into(),
                args: vec!["hello".into()],
                timeout_secs: 10,
            }
        ],
    }
}

pub struct PlanBuilder {
    plan: Plan,
}

impl PlanBuilder {
    pub fn new() -> Self {
        Self { plan: Plan::default() }
    }

    pub fn with_task(mut self, task: Task) -> Self {
        self.plan.tasks.push(task);
        self
    }

    pub fn build(self) -> Plan {
        self.plan
    }
}
```

### Async Test Helpers

```rust
// tests/helpers.rs
use tokio::time::timeout;
use std::time::Duration;

pub async fn wait_for_condition<F>(mut condition: F, max_wait: Duration) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < max_wait {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}
```

---

## Component-Specific Guidelines

### AGQ Testing

- 100% coverage on authentication
- 100% coverage on RESP parser
- Fuzz RESP protocol parsing
- Test all data structure operations (lists, sorted sets)
- Test concurrent access scenarios
- Test TTL expiration

### AGW Testing

- Test all tool executions
- Test timeout enforcement
- Test stdout/stderr capture
- Test fail-fast behavior
- Test graceful shutdown
- Mock external tool calls in tests

### AGX Testing

- Test plan generation with different prompts
- Test schema validation
- Test tool registry integration
- Mock LLM calls in tests
- Test plan serialization/deserialization

---

## Related Documentation

- [Security Guidelines](./security-guidelines.md) - Security testing requirements
- [CONTRIBUTING.md](https://github.com/agenix-sh/agenix/blob/main/CONTRIBUTING.md) - Contribution workflow
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly
**Questions?** See component-specific CLAUDE.md files or open an issue
