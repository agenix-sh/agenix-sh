# Security Guidelines

**Version:** 1.0
**Status:** Canonical Development Guidelines
**Last Updated:** 2025-11-17

This document defines security guidelines for developing AGEniX components.

---

## Table of Contents

1. [Security-First Mindset](#security-first-mindset)
2. [OWASP Top 10 Considerations](#owasp-top-10-considerations)
3. [Rust-Specific Security](#rust-specific-security)
4. [Cryptography](#cryptography)
5. [Input Validation](#input-validation)
6. [Authentication & Authorization](#authentication--authorization)
7. [Dependency Security](#dependency-security)
8. [Security Checklist](#security-checklist)

---

## 1. Security-First Mindset

### Core Principle

**Every line of code in AGEniX is security-critical.**

AGQ handles authentication, job execution, and worker coordination. AGW executes user-provided plans. AGX processes potentially malicious user input. There is no "non-security-critical" code path.

### Threat Model

Assume:
- Users provide malicious input
- Network is hostile (even localhost can be compromised)
- Workers may be compromised
- Plans may contain injection attempts
- Time is adversarial (timing attacks)

### Defense in Depth

Layer multiple security controls:
1. **Input validation** - Reject bad data early
2. **Sandboxing** - Isolate execution
3. **Least privilege** - Minimum necessary permissions
4. **Fail secure** - Default to deny, not allow
5. **Audit logging** - Detect and investigate incidents

---

## 2. OWASP Top 10 Considerations

### 2.1 Injection Attacks

**Risk**: Command injection, SQL/NoSQL injection, RESP protocol injection

#### Command Injection

**❌ NEVER DO THIS**:
```rust
// DANGEROUS: Direct shell execution with user input
let cmd = format!("ls {}", user_input);
std::process::Command::new("sh").arg("-c").arg(&cmd).spawn();
```

**✅ DO THIS**:
```rust
// SAFE: Direct process spawn, no shell
use std::process::Command;

fn validate_path(path: &str) -> Result<PathBuf, Error> {
    // Validate path doesn't contain shell metacharacters
    if path.contains(&['|', ';', '&', '$', '`', '\n'][..]) {
        bail!("Invalid path: contains shell metacharacters");
    }

    // Canonicalize to prevent path traversal
    let canonical = PathBuf::from(path).canonicalize()?;

    // Ensure it's within allowed directory
    if !canonical.starts_with("/allowed/dir") {
        bail!("Path outside allowed directory");
    }

    Ok(canonical)
}

// Execute without shell
let validated_path = validate_path(user_input)?;
Command::new("ls").arg(validated_path).spawn()?;
```

#### RESP Protocol Injection

**❌ NEVER DO THIS**:
```rust
// DANGEROUS: Injecting user input into RESP message
let msg = format!("SET {} {}\r\n", user_key, user_value);
```

**✅ DO THIS**:
```rust
// SAFE: Use proper RESP serialization
use crate::protocol::RespValue;

let command = RespValue::Array(vec![
    RespValue::BulkString(b"SET".to_vec()),
    RespValue::BulkString(user_key.as_bytes().to_vec()),
    RespValue::BulkString(user_value.as_bytes().to_vec()),
]);
let serialized = command.serialize();
```

### 2.2 Authentication & Session Management

**Requirements**:
- Session keys must be cryptographically random (32+ bytes)
- Constant-time comparison to prevent timing attacks
- No session keys in logs
- Rate limiting on authentication attempts
- Automatic session expiry (via TTL)

**✅ CORRECT**:
```rust
use ring::rand::{SecureRandom, SystemRandom};
use subtle::ConstantTimeEq;

// Generate session key
fn generate_session_key() -> Result<String, Error> {
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key)?;
    Ok(hex::encode(key))
}

// Constant-time comparison
fn validate_session_key(provided: &[u8], expected: &[u8]) -> Result<(), Error> {
    if provided.len() != expected.len() {
        // Return error without revealing which check failed
        bail!("Invalid session key");
    }

    if provided.ct_eq(expected).into() {
        Ok(())
    } else {
        bail!("Invalid session key");
    }
}
```

### 2.3 Sensitive Data Exposure

**Never log**:
- Session keys
- Plan contents (may contain credentials)
- Worker registration tokens
- Job payloads
- Error details with sensitive data

**✅ CORRECT**:
```rust
use tracing::{info, error, instrument};

#[instrument(skip(session_key))]  // Don't log session_key
async fn authenticate(worker_id: &str, session_key: &[u8]) -> Result<(), Error> {
    info!(worker_id, "Authenticating worker");

    match validate_session_key(session_key) {
        Ok(_) => {
            info!(worker_id, "Authentication successful");
            Ok(())
        }
        Err(e) => {
            // Log error without exposing key
            error!(worker_id, "Authentication failed");
            Err(e)
        }
    }
}
```

### 2.4 Resource Exhaustion (DoS)

**Implement limits**:
- Connection limits per IP
- Maximum message size
- Job queue depth limits
- Worker registration rate limits
- Timeout all blocking operations

**✅ CORRECT**:
```rust
use tokio::time::timeout;
use std::time::Duration;

// Always set timeouts
async fn execute_with_timeout<F, T>(future: F) -> Result<T, Error>
where
    F: Future<Output = Result<T, Error>>,
{
    timeout(Duration::from_secs(30), future)
        .await
        .map_err(|_| Error::Timeout)?
}

// Rate limiting
use governor::{Quota, RateLimiter};

struct AuthRateLimiter {
    limiter: RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>,
}

impl AuthRateLimiter {
    fn new() -> Self {
        let quota = Quota::per_minute(nonzero!(5u32));
        Self {
            limiter: RateLimiter::keyed(quota),
        }
    }

    fn check(&self, ip: &str) -> Result<(), Error> {
        self.limiter
            .check_key(&ip.to_string())
            .map_err(|_| Error::RateLimitExceeded)?;
        Ok(())
    }
}
```

### 2.5 Security Misconfiguration

**Secure defaults**:
- Deny by default
- No debug endpoints in production
- Validate all configuration at startup
- Fail closed, not open

**✅ CORRECT**:
```rust
#[derive(Debug)]
pub struct Config {
    /// Listen address (default: localhost only)
    pub listen_addr: String,

    /// Enable debug endpoints (default: false)
    #[cfg(debug_assertions)]
    pub enable_debug: bool,

    /// Maximum connections (default: 100)
    pub max_connections: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:6380".into(),  // Localhost only
            #[cfg(debug_assertions)]
            enable_debug: false,  // Disabled by default
            max_connections: 100,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        // Ensure not listening on 0.0.0.0 in production
        if !cfg!(debug_assertions) && self.listen_addr.starts_with("0.0.0.0") {
            bail!("Cannot bind to 0.0.0.0 in production");
        }

        // Validate connection limit is reasonable
        if self.max_connections > 10000 {
            bail!("max_connections too high (>10000)");
        }

        Ok(())
    }
}
```

### 2.6 Deserialization Vulnerabilities

**Validate JSON strictly**:
- Set maximum nesting depth
- Limit array/object sizes
- Never use `unsafe` for deserialization
- Deny unknown fields

**✅ CORRECT**:
```rust
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]  // Reject unexpected fields
struct Plan {
    #[serde(deserialize_with = "validate_plan_id")]
    id: String,

    #[serde(deserialize_with = "validate_tasks")]
    tasks: Vec<Task>,
}

fn validate_plan_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let id = String::deserialize(deserializer)?;

    // Validate format
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(serde::de::Error::custom("Invalid plan ID format"));
    }

    // Validate length
    if id.len() > 64 {
        return Err(serde::de::Error::custom("Plan ID too long"));
    }

    Ok(id)
}

fn validate_tasks<'de, D>(deserializer: D) -> Result<Vec<Task>, D::Error>
where
    D: Deserializer<'de>,
{
    let tasks = Vec::<Task>::deserialize(deserializer)?;

    // Limit task count
    if tasks.len() > 100 {
        return Err(serde::de::Error::custom("Too many tasks (max 100)"));
    }

    Ok(tasks)
}
```

---

## 3. Rust-Specific Security

### 3.1 Memory Safety

**Minimize `unsafe` blocks**:
```rust
// ✅ SAFE: Use safe abstractions
fn process_data(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| b ^ 0xFF).collect()
}

// ❌ UNSAFE: Only use when absolutely necessary
unsafe fn process_data_unsafe(data: *const u8, len: usize) -> Vec<u8> {
    // SAFETY: Caller must ensure data is valid for len bytes
    std::slice::from_raw_parts(data, len)
        .iter()
        .map(|&b| b ^ 0xFF)
        .collect()
}
```

**If `unsafe` is required**:
- Document safety invariants
- Add SAFETY comment explaining why it's safe
- Minimize unsafe scope
- Add tests specifically for unsafe code

### 3.2 Integer Overflow

**Use checked arithmetic** for security-critical calculations:

**✅ CORRECT**:
```rust
fn calculate_timeout(base: u64, multiplier: u32) -> Result<u64, Error> {
    let multiplier_u64 = u64::from(multiplier);
    base.checked_mul(multiplier_u64)
        .ok_or_else(|| Error::IntegerOverflow)
}
```

**Enable overflow checks in release**:
```toml
[profile.release]
overflow-checks = true
```

### 3.3 Panic Safety

**Never panic in production code paths**:

**❌ BAD**:
```rust
fn process_job(job_id: &str) -> JobStatus {
    let job = jobs.get(job_id).unwrap();  // PANIC if not found
    job.status
}
```

**✅ GOOD**:
```rust
fn process_job(job_id: &str) -> Result<JobStatus, Error> {
    let job = jobs.get(job_id)
        .ok_or_else(|| Error::JobNotFound(job_id.to_string()))?;
    Ok(job.status)
}
```

---

## 4. Cryptography

### 4.1 Use Established Libraries

**✅ USE**:
- `ring` - Cryptographic primitives
- `rustls` - TLS implementation
- `subtle` - Constant-time operations

**❌ NEVER**:
- Roll your own crypto
- Use deprecated algorithms (MD5, SHA-1, DES)
- Implement timing-sensitive code without constant-time guarantees

### 4.2 Random Number Generation

**✅ CORRECT**:
```rust
use ring::rand::{SecureRandom, SystemRandom};

fn generate_nonce() -> Result<Vec<u8>, Error> {
    let rng = SystemRandom::new();
    let mut nonce = vec![0u8; 32];
    rng.fill(&mut nonce)?;
    Ok(nonce)
}
```

**❌ WRONG**:
```rust
use rand::Rng;

fn generate_nonce() -> Vec<u8> {
    let mut rng = rand::thread_rng();  // NOT cryptographically secure
    (0..32).map(|_| rng.gen()).collect()
}
```

### 4.3 Constant-Time Operations

**Use for secret comparison**:
```rust
use subtle::ConstantTimeEq;

fn compare_secrets(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}
```

---

## 5. Input Validation

### 5.1 Validation Pattern

**Always validate at boundaries**:

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct JobRequest {
    #[validate(length(min = 1, max = 64))]
    #[validate(regex = "^[a-zA-Z0-9_-]+$")]
    job_id: String,

    #[validate(length(max = 1048576))]  // 1MB max
    payload: String,

    #[validate(range(min = 1, max = 100))]
    task_count: usize,
}

fn handle_job_request(req: JobRequest) -> Result<(), Error> {
    // Validate before processing
    req.validate()?;

    // Now safe to process
    process_job(&req)
}
```

### 5.2 Common Validation Functions

```rust
/// Validate worker ID (alphanumeric + hyphens/underscores only)
pub fn validate_worker_id(id: &str) -> Result<(), Error> {
    if id.is_empty() {
        bail!("Worker ID cannot be empty");
    }

    if id.len() > 64 {
        bail!("Worker ID too long (max 64 chars)");
    }

    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        bail!("Worker ID contains invalid characters");
    }

    Ok(())
}

/// Validate file path (prevent traversal)
pub fn validate_file_path(path: &str) -> Result<PathBuf, Error> {
    let path = PathBuf::from(path);

    // Reject absolute paths
    if path.is_absolute() {
        bail!("Absolute paths not allowed");
    }

    // Reject parent directory references
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            bail!("Path traversal not allowed");
        }
    }

    // Canonicalize and ensure within allowed base
    let canonical = path.canonicalize()?;
    let allowed_base = PathBuf::from("/allowed/base").canonicalize()?;

    if !canonical.starts_with(&allowed_base) {
        bail!("Path outside allowed directory");
    }

    Ok(canonical)
}
```

---

## 6. Authentication & Authorization

### 6.1 Session Key Management

**Generation**:
```rust
fn generate_session_key() -> Result<String, Error> {
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key)?;
    Ok(hex::encode(key))
}
```

**Storage** (AGQ):
- Store hex-encoded in memory only
- Never persist to disk
- Clear on shutdown

**Distribution** (AGX/AGW):
- Read from environment variable or secure config file
- Never hardcode
- Never commit to git

**Comparison**:
```rust
use subtle::ConstantTimeEq;

fn authenticate(provided_key: &[u8], stored_key: &[u8]) -> Result<(), Error> {
    if provided_key.len() != stored_key.len() {
        bail!("Invalid session key");
    }

    if provided_key.ct_eq(stored_key).into() {
        Ok(())
    } else {
        bail!("Invalid session key");
    }
}
```

### 6.2 Authorization

**Job ownership**:
```rust
fn can_update_job(job_id: &str, worker_id: &str) -> Result<(), Error> {
    let claimed_by = db.get(format!("job:{}:worker", job_id))?;

    if claimed_by != worker_id {
        bail!("Worker {} cannot update job claimed by {}", worker_id, claimed_by);
    }

    Ok(())
}
```

---

## 7. Dependency Security

### 7.1 Audit Dependencies

**Before every commit**:
```bash
cargo audit
```

**In CI**:
```yaml
- name: Security audit
  run: cargo audit
```

### 7.2 Minimize Dependencies

**Principles**:
- Only add dependencies when necessary
- Prefer well-maintained crates
- Review dependency tree: `cargo tree`
- Avoid unmaintained crates
- Check for security advisories

### 7.3 Pin Versions

**Use `Cargo.lock`**:
- Commit `Cargo.lock` to git
- Ensures reproducible builds
- Prevents automatic updates to vulnerable versions

---

## 8. Security Checklist

### Before Every PR

- [ ] No user input flows to system commands without validation
- [ ] All input is validated and sanitized
- [ ] Authentication checks are in place where required
- [ ] No secrets in logs or error messages
- [ ] Timeouts set on all I/O operations
- [ ] Integer overflow cannot occur in critical paths
- [ ] Deserialization is safe and bounded
- [ ] No `unsafe` code (or justified with SAFETY comments)
- [ ] Cryptographic operations use constant-time comparisons
- [ ] Error messages don't leak sensitive data
- [ ] Resource limits enforced (memory, connections, etc.)
- [ ] All error paths tested
- [ ] Security tests added for new attack surfaces
- [ ] `cargo audit` passes
- [ ] `cargo clippy -- -W clippy::security` passes

### Security Review Questions

1. **What could go wrong if this input is malicious?**
2. **What happens if this operation times out?**
3. **Could this leak information to an attacker?**
4. **Is this comparison timing-safe?**
5. **Could an attacker exhaust resources here?**
6. **Is this error message too revealing?**
7. **Could this panic in production?**
8. **Is this the minimal privilege needed?**

---

## Component-Specific Guidelines

### AGQ Security

- 100% test coverage on authentication
- Fuzz RESP parser
- Rate limit all commands
- Validate all RESP inputs
- Constant-time session key comparison
- No plan contents in logs

### AGW Security

- Validate all tool names against allowlist
- Sanitize all arguments before execution
- Never use shell for command execution
- Enforce timeouts on all tasks
- Capture and sanitize stdout/stderr
- No network access during execution (future: network namespace)

### AGX Security

- Validate all user prompts (length limits)
- Sanitize plan JSON before submission
- Never execute plans locally without user confirmation
- Validate tool registry responses
- Rate limit LLM API calls

---

## Reporting Security Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

**Instead**:
1. Email: `security@agenix.sh`
2. Or create private security advisory on GitHub
3. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

**Response time**: Within 48 hours

---

## Related Documentation

- [Testing Strategy](./testing-strategy.md) - Security testing requirements
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Security model
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly or on security incidents
**Questions?** Open issue with `security` label or email security@agenix.sh
