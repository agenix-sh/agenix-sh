---
name: rust-agenix-standards
description: Apply AGEniX Rust coding standards, idioms, error handling patterns, and best practices when writing code for agx, agq, agw, or AU components
allowed-tools: Read, Grep, Glob
---

# Rust AGEniX Standards Skill

This skill enforces consistent Rust coding standards and idioms across all AGEniX components.

## Canonical Documentation

- **Security Guidelines**: `/Users/lewis/work/agenix-sh/agenix/docs/development/security-guidelines.md`
- **Testing Strategy**: `/Users/lewis/work/agenix-sh/agenix/docs/development/testing-strategy.md`

## Error Handling

### Use `anyhow` for Application Code

```rust
use anyhow::{Context, Result, bail};

// ✅ Good: Descriptive context
fn load_config(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .context(format!("Failed to read config from {:?}", path))?;

    serde_json::from_str(&contents)
        .context("Failed to parse config JSON")
}

// ✅ Good: Use bail! for explicit errors
fn validate_worker_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("Worker ID cannot be empty");
    }
    Ok(())
}
```

### Use `thiserror` for Library Code

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid session key: {0}")]
    InvalidSessionKey(String),

    #[error("Worker not registered: {0}")]
    WorkerNotFound(String),

    #[error("Authentication timeout")]
    Timeout,
}
```

### Never Panic in Production

```rust
// ❌ BAD: Can panic
fn get_first_task(plan: &Plan) -> &Task {
    &plan.tasks[0]  // Panics if empty!
}

// ✅ GOOD: Returns Result
fn get_first_task(plan: &Plan) -> Result<&Task> {
    plan.tasks.first()
        .ok_or_else(|| anyhow!("Plan has no tasks"))
}

// ✅ GOOD: Returns Option
fn get_first_task(plan: &Plan) -> Option<&Task> {
    plan.tasks.first()
}
```

## Async/Await Patterns

### Use Tokio for Async Runtime

```rust
// ✅ Main function
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
    Ok(())
}

// ✅ Async tests
#[tokio::test]
async fn test_async_function() {
    let result = fetch_data().await;
    assert!(result.is_ok());
}
```

### Spawn Tasks Properly

```rust
// ✅ Spawn with proper error handling
tokio::spawn(async move {
    if let Err(e) = worker_heartbeat_loop().await {
        eprintln!("Heartbeat failed: {}", e);
    }
});

// ✅ Use JoinHandle for important tasks
let handle = tokio::spawn(async move {
    process_job(job).await
});

// Wait and handle result
let result = handle.await
    .context("Task panicked")??;  // Handle both join and task errors
```

## Ownership and Borrowing

### Prefer Borrowing

```rust
// ✅ Good: Borrows string
fn validate_worker_id(id: &str) -> Result<()> {
    // ...
}

// ❌ Avoid: Takes ownership unnecessarily
fn validate_worker_id(id: String) -> Result<()> {
    // ...
}
```

### Use `Cow` for Flexible Ownership

```rust
use std::borrow::Cow;

fn process_text(text: Cow<str>) -> String {
    if text.contains("AGX") {
        // Need to modify: convert to owned
        text.replace("AGX", "agx").into()
    } else {
        // No modification: use borrowed
        text.into_owned()
    }
}
```

## Type Safety

### Use Newtypes for Domain Concepts

```rust
// ✅ Type-safe IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new(id: String) -> Result<Self> {
        validate_worker_id(&id)?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Now you can't accidentally mix up worker IDs with job IDs
fn register_worker(id: WorkerId) { }  // Type-safe!
```

### Use Enums for State

```rust
#[derive(Debug, Clone)]
pub enum JobStatus {
    Pending,
    Running { worker_id: WorkerId, started_at: SystemTime },
    Completed { result: JobResult, finished_at: SystemTime },
    Failed { error: String, failed_at: SystemTime },
}

// Pattern matching ensures all cases handled
match job.status {
    JobStatus::Pending => { /* ... */ }
    JobStatus::Running { worker_id, .. } => { /* ... */ }
    JobStatus::Completed { result, .. } => { /* ... */ }
    JobStatus::Failed { error, .. } => { /* ... */ }
}
```

## Naming Conventions

### Follow Rust Conventions

```rust
// ✅ Types: PascalCase
struct SessionKey { }
enum JobStatus { }

// ✅ Functions/methods: snake_case
fn validate_session_key() { }
fn parse_resp_message() { }

// ✅ Constants: SCREAMING_SNAKE_CASE
const MAX_WORKERS: usize = 100;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ✅ Modules: snake_case
mod worker_pool { }
mod resp_protocol { }
```

### Use Descriptive Names

```rust
// ✅ Clear intent
let session_key = generate_session_key();
let worker_count = count_active_workers();

// ❌ Unclear abbreviations
let sk = gen_key();
let cnt = count();
```

## Documentation

### Document Public APIs

```rust
/// Validates a worker ID according to AGEniX naming rules.
///
/// # Arguments
///
/// * `id` - The worker ID to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error describing why validation failed.
///
/// # Errors
///
/// Returns error if:
/// - ID is empty
/// - ID contains invalid characters
/// - ID is too long (> 64 characters)
///
/// # Examples
///
/// ```
/// # use agq::validate_worker_id;
/// assert!(validate_worker_id("worker-001").is_ok());
/// assert!(validate_worker_id("invalid; rm -rf /").is_err());
/// ```
pub fn validate_worker_id(id: &str) -> Result<()> {
    // Implementation
}
```

### Use `#[must_use]` for Important Returns

```rust
#[must_use = "Session key must be stored securely"]
pub fn generate_session_key() -> String {
    // ...
}
```

## Code Organization

### Module Structure

```
src/
├── lib.rs              # Public API, re-exports
├── server.rs           # AGQ server implementation
├── protocol/
│   ├── mod.rs          # RESP protocol public API
│   ├── parser.rs       # RESP parsing logic
│   └── types.rs        # RESP data types
├── storage/
│   ├── mod.rs
│   └── redb.rs         # redb backend
└── worker/
    ├── mod.rs
    └── heartbeat.rs
```

### Public API in lib.rs

```rust
// src/lib.rs
pub mod protocol;
pub mod server;

// Re-export commonly used types
pub use protocol::{RespMessage, RespCommand};
pub use server::Server;

// Private modules
mod storage;
mod worker;
```

## Performance Patterns

### Avoid Unnecessary Allocations

```rust
// ✅ Good: Reuse buffer
let mut buffer = Vec::with_capacity(1024);
for _ in 0..100 {
    buffer.clear();
    read_into_buffer(&mut buffer)?;
    process(&buffer)?;
}

// ❌ Wasteful: Allocates every iteration
for _ in 0..100 {
    let buffer = Vec::new();
    // ...
}
```

### Use `&str` Over `String` When Possible

```rust
// ✅ Good: No allocation needed
fn log_message(msg: &str) {
    eprintln!("[LOG] {}", msg);
}

// ❌ Forces caller to allocate
fn log_message(msg: String) {
    eprintln!("[LOG] {}", msg);
}
```

## Unsafe Code

### Minimize `unsafe` Usage

```rust
// Only use unsafe when absolutely necessary
// Always document with SAFETY comments

/// # Safety
///
/// Caller must ensure `ptr` is:
/// - Valid and properly aligned
/// - Points to initialized memory
/// - Not accessed concurrently from other threads
unsafe fn read_unaligned(ptr: *const u8) -> u8 {
    std::ptr::read_unaligned(ptr)
}
```

### Prefer Safe Abstractions

```rust
// ✅ Use safe abstractions when possible
use bytes::Bytes;

fn process_buffer(buf: &Bytes) {
    // Safe slice operations
}

// ❌ Avoid raw pointer manipulation
// unsafe fn process_buffer(ptr: *const u8, len: usize) { }
```

## Dependencies

### Prefer Well-Maintained Crates

**Approved dependencies:**
- `anyhow` - Error handling
- `thiserror` - Error types
- `tokio` - Async runtime
- `serde`, `serde_json` - Serialization
- `clap` - CLI parsing
- `tracing` - Logging
- `bytes` - Efficient byte buffers
- `redb` - Embedded database
- `subtle` - Constant-time operations

### Pin Major Versions

```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }  # Pin to 1.x
serde = { version = "1.0", features = ["derive"] }
```

## Formatting and Linting

### Always Run Before Committing

```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings

# No warnings allowed!
```

### Custom Clippy Configuration

```toml
# .clippy.toml or Cargo.toml [package.metadata.clippy]
```

Deny these lints:
- `unwrap_used` - Force proper error handling
- `expect_used` - Except in tests
- `panic` - No panics in production
- `todo` - No unfinished code in production

## Testing Patterns

### Use `#[cfg(test)]` Modules

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_registration() {
        // Test code
    }
}
```

### Helper Functions for Tests

```rust
#[cfg(test)]
mod test_helpers {
    pub fn create_test_server() -> Server {
        // Common test setup
    }

    pub fn create_test_plan() -> Plan {
        Plan {
            plan_id: "test-plan".into(),
            tasks: vec![/* ... */],
        }
    }
}
```

## When to Activate This Skill

Use this skill when:
- Writing new Rust code
- Reviewing Rust PRs
- Refactoring existing code
- Setting up new Rust projects
- Deciding on error handling approaches
- Choosing appropriate data structures

## Quick Checklist

Before committing Rust code:

- [ ] Runs `cargo fmt`
- [ ] Passes `cargo clippy -- -D warnings`
- [ ] No `unwrap()` or `expect()` in production code
- [ ] All public APIs documented
- [ ] Tests written (TDD)
- [ ] Error messages are descriptive
- [ ] No panics in production paths
- [ ] Async code uses Tokio properly
- [ ] Types follow naming conventions
- [ ] Security patterns applied (from agenix-security skill)

For comprehensive Rust security patterns, reference the agenix-security skill and security-guidelines.md documentation.
