# Zero-Trust Execution Model

**Version:** 1.0
**Status:** Canonical Security Architecture
**Last Updated:** 2025-11-17

AGEniX is designed with a zero-trust security model at its core. This document defines the threat model, security principles, and mitigation strategies.

---

## Table of Contents

1. [Zero-Trust Principles](#zero-trust-principles)
2. [Threat Model](#threat-model)
3. [Trust Boundaries](#trust-boundaries)
4. [Security Controls](#security-controls)
5. [Attack Scenarios & Mitigations](#attack-scenarios--mitigations)
6. [Future Enhancements](#future-enhancements)

---

## 1. Zero-Trust Principles

### Core Tenet

**Never trust, always verify.**

Every component, user, and process is assumed to be potentially compromised or malicious.

### Foundational Principles

#### 1.1 Workers Don't Generate Plans

**Principle**: AGW workers are pure execution engines, not planners.

**Implementation**:
- Workers **never** call LLMs
- Workers **never** generate or modify plans
- Workers only execute pre-approved JSON plans
- Workers cannot make planning decisions

**Rationale**: Prevents compromised workers from creating malicious execution plans.

#### 1.2 Tools Are Untrusted Processes

**Principle**: All tools (Unix commands, AUs) are treated as potentially malicious.

**Implementation**:
- Tools invoked via stdin/stdout/stderr (no shell)
- Each tool runs as separate process
- Timeout enforcement per task
- Resource limits (CPU, memory)
- No network access (future: network namespaces)
- Captured stdout/stderr sanitized before logging

**Rationale**: Limits blast radius if tool is compromised or malicious.

#### 1.3 Plans Are Signed and Verified

**Principle**: Plans carry cryptographic proof of origin.

**Implementation** (Phase 2+):
- AGX signs plans with private key
- AGW verifies signature before execution
- Invalid signatures → immediate rejection
- Signature includes plan hash (prevents tampering)

**Rationale**: Ensures plans come from trusted planner, not injected by attacker.

#### 1.4 No Direct LLM Access for Workers

**Principle**: Workers cannot make API calls to LLM providers.

**Implementation**:
- No API keys in worker environment
- Network policies block LLM endpoints (future)
- Workers operate in air-gapped mode

**Rationale**: Prevents compromised worker from using LLM to generate malicious plans.

#### 1.5 Explicit Allowlists

**Principle**: Everything is denied by default; only explicitly allowed items are permitted.

**Implementation**:
- Tool allowlist (only registered tools can execute)
- Command argument validation
- File path allowlist (only specific directories)
- Network allowlist (only specific endpoints, Phase 3+)

**Rationale**: Minimizes attack surface by denying unexpected operations.

---

## 2. Threat Model

### 2.1 Threat Actors

| Actor | Capability | Motivation |
|-------|-----------|------------|
| **Malicious User** | Can submit arbitrary plans via AGX | Exfiltrate data, gain persistence, DoS |
| **Compromised Worker** | Can execute jobs, read plans | Lateral movement, data exfiltration |
| **Compromised Tool/AU** | Can execute within worker sandbox | Escape sandbox, persist, exfiltrate |
| **Network Attacker** | MITM on AGX ↔ AGQ ↔ AGW communication | Intercept plans, inject malicious jobs |
| **Insider Threat** | Access to configuration, session keys | Full system compromise |

### 2.2 Assets to Protect

| Asset | Sensitivity | Threat |
|-------|------------|--------|
| **Session Keys** | Critical | Allows authentication as worker/planner |
| **Plans** | High | May contain credentials, sensitive commands |
| **Job Outputs** | High | May contain extracted data, secrets |
| **Worker Capabilities** | Medium | Reveals available tools, attack surface |
| **AGQ Database** | High | Contains all jobs, plans, worker metadata |

### 2.3 Attack Vectors

1. **Command Injection**: Malicious input in plan arguments
2. **Path Traversal**: Accessing files outside allowed directories
3. **Plan Tampering**: Modifying plans in transit or storage
4. **Unauthorized Execution**: Running jobs without proper authentication
5. **Resource Exhaustion**: DoS via large plans, infinite loops
6. **Data Exfiltration**: Stealing outputs via malicious tools
7. **Privilege Escalation**: Worker escaping sandbox
8. **Replay Attacks**: Re-submitting old jobs/plans

---

## 3. Trust Boundaries

### 3.1 Component Trust Levels

```
┌─────────────────────────────────────────────────────┐
│ Trusted Zone                                        │
│                                                     │
│  ┌─────────┐         ┌─────────┐                   │
│  │   AGX   │────────▶│   AGQ   │                   │
│  │(Planner)│         │ (Queue) │                   │
│  └─────────┘         └─────────┘                   │
│       │ Signed Plans       │                        │
│       │                    │ Jobs                   │
└───────┼────────────────────┼────────────────────────┘
        │                    │
        │                    ▼
┌───────┼────────────────────────────────────────────┐
│ Untrusted Zone             │                        │
│                            │                        │
│                    ┌───────▼──────┐                 │
│                    │     AGW      │                 │
│                    │  (Worker)    │                 │
│                    └──────┬───────┘                 │
│                           │ Executes                │
│                           │                         │
│                    ┌──────▼───────┐                 │
│                    │  Tools / AUs │                 │
│                    │  (Sandboxed) │                 │
│                    └──────────────┘                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Trusted Zone**:
- AGX: Generates plans, has LLM access
- AGQ: Stores plans, manages queue, authenticates workers

**Untrusted Zone**:
- AGW: Executes plans, no LLM access
- Tools/AUs: External processes, sandboxed

### 3.2 Data Flow Trust

```
User Input (Untrusted)
    ↓
AGX Validation ───────────────┐ (Sanitization)
    ↓                         │
Plan Generation (LLM)         │
    ↓                         │
Plan Validation               │
    ↓                         │
Sign Plan ←───────────────────┘
    ↓
AGQ Storage (Trusted after signature)
    ↓
AGW Pulls Job
    ↓
Verify Signature ──────────────┐ (Trust check)
    ↓                          │
Execute Tasks (Sandbox) ←──────┘
    ↓
Sanitize Outputs
    ↓
Return Results (Treated as untrusted until reviewed)
```

---

## 4. Security Controls

### 4.1 Authentication & Authorization

**Session-Key Authentication**:
```rust
// Constant-time comparison prevents timing attacks
use subtle::ConstantTimeEq;

fn authenticate(provided: &[u8], expected: &[u8]) -> bool {
    provided.ct_eq(expected).into()
}
```

**Authorization Matrix**:

| Actor | Can Submit Plans | Can Execute Jobs | Can Modify Plans |
|-------|-----------------|------------------|------------------|
| AGX | ✅ | ❌ | ✅ (before submission) |
| AGQ | ❌ | ❌ | ❌ (only stores) |
| AGW | ❌ | ✅ | ❌ (only executes) |
| Tool | ❌ | ❌ | ❌ |

### 4.2 Input Validation

**All inputs validated at boundaries**:

```rust
// Plan validation
fn validate_plan(plan: &Plan) -> Result<(), Error> {
    // 1. Schema compliance
    plan.validate_schema()?;

    // 2. Task limits
    if plan.tasks.len() > 100 {
        bail!("Too many tasks (max 100)");
    }

    // 3. Task references
    for task in &plan.tasks {
        if let Some(input_from) = task.input_from_task {
            if input_from >= task.task_number {
                bail!("Invalid task reference");
            }
        }
    }

    // 4. Command allowlist
    for task in &plan.tasks {
        if !is_allowed_command(&task.command) {
            bail!("Command not in allowlist: {}", task.command);
        }
    }

    Ok(())
}
```

### 4.3 Execution Sandboxing

**Process Isolation**:
```rust
use std::process::Command;

fn execute_task_sandboxed(task: &Task) -> Result<TaskResult, Error> {
    let mut cmd = Command::new(&task.command);

    // 1. Set arguments (no shell interpolation)
    cmd.args(&task.args);

    // 2. Set timeout
    let timeout = Duration::from_secs(task.timeout_secs.into());

    // 3. Limit resources (future: cgroups)
    // cmd.env("RLIMIT_CPU", "60");
    // cmd.env("RLIMIT_AS", "512M");

    // 4. Drop privileges (future: user namespaces)
    // cmd.uid(65534);  // nobody

    // 5. Restrict network (future: network namespaces)
    // cmd.unshare(CLONE_NEWNET);

    // 6. Execute with timeout
    tokio::time::timeout(timeout, cmd.output()).await??
}
```

### 4.4 Output Sanitization

**Sanitize before logging**:
```rust
fn sanitize_output(output: &[u8]) -> String {
    // 1. Limit size
    let truncated = if output.len() > 10_000 {
        &output[..10_000]
    } else {
        output
    };

    // 2. Convert to UTF-8, replacing invalid sequences
    String::from_utf8_lossy(truncated).to_string()

    // 3. Remove ANSI escape codes (future)
    // remove_ansi_codes(&s)

    // 4. Redact patterns that look like secrets (future)
    // redact_secrets(&s)
}
```

### 4.5 Rate Limiting & Resource Limits

**Prevent DoS**:
```rust
// Connection limits
const MAX_CONNECTIONS_PER_IP: usize = 10;

// Request rate limits
const MAX_REQUESTS_PER_MINUTE: u32 = 1000;

// Job limits
const MAX_JOBS_PER_WORKER: usize = 4;

// Plan limits
const MAX_PLAN_SIZE_BYTES: usize = 1_048_576;  // 1MB
const MAX_TASKS_PER_PLAN: usize = 100;
const MAX_TASK_TIMEOUT_SECS: u64 = 3600;  // 1 hour
```

---

## 5. Attack Scenarios & Mitigations

### 5.1 Command Injection

**Attack**: Malicious user submits plan with command injection in arguments.

**Example**:
```json
{
  "tasks": [
    {
      "command": "grep",
      "args": ["; rm -rf /", "file.txt"]
    }
  ]
}
```

**Mitigations**:
1. ✅ **No shell execution** - Use `Command::new()` directly, not `sh -c`
2. ✅ **Argument validation** - Reject arguments with shell metacharacters
3. ✅ **Command allowlist** - Only pre-registered commands allowed
4. ✅ **Process isolation** - Each task runs as separate process

**Code**:
```rust
// SAFE: Direct process execution
Command::new("grep").arg(user_arg).arg("file.txt").spawn();

// DANGEROUS: Shell execution (NEVER DO THIS)
// Command::new("sh").arg("-c").arg(format!("grep {} file.txt", user_arg));
```

### 5.2 Path Traversal

**Attack**: Malicious plan attempts to access files outside allowed directories.

**Example**:
```json
{
  "tasks": [
    {
      "command": "cat",
      "args": ["../../../etc/passwd"]
    }
  ]
}
```

**Mitigations**:
1. ✅ **Path validation** - Canonicalize paths, check against allowlist
2. ✅ **Working directory restriction** - Start in sandboxed directory
3. ✅ **Filesystem isolation** (future) - chroot or mount namespaces

**Code**:
```rust
fn validate_path(path: &str) -> Result<PathBuf, Error> {
    let canonical = PathBuf::from(path).canonicalize()?;
    let allowed = PathBuf::from("/allowed/workspace").canonicalize()?;

    if !canonical.starts_with(&allowed) {
        bail!("Path outside allowed directory");
    }

    Ok(canonical)
}
```

### 5.3 Plan Tampering (MITM)

**Attack**: Attacker intercepts AGX → AGQ communication and modifies plan.

**Mitigations** (Phase 2+):
1. 🔜 **Plan signing** - AGX signs plans with private key
2. 🔜 **Signature verification** - AGW verifies before execution
3. 🔜 **TLS encryption** - Encrypt AGX ↔ AGQ ↔ AGW communication
4. ✅ **Session-key authentication** - Prevents unauthorized submission

**Code** (future):
```rust
// Sign plan (AGX)
fn sign_plan(plan: &Plan, private_key: &PrivateKey) -> Signature {
    let plan_hash = sha256(serde_json::to_vec(plan));
    sign(private_key, &plan_hash)
}

// Verify plan (AGW)
fn verify_plan(plan: &Plan, signature: &Signature, public_key: &PublicKey) -> Result<(), Error> {
    let plan_hash = sha256(serde_json::to_vec(plan));
    verify(public_key, &plan_hash, signature)
}
```

### 5.4 Privilege Escalation

**Attack**: Compromised tool attempts to escape sandbox.

**Mitigations** (Phase 3):
1. 🔜 **User namespaces** - Run tools as unprivileged user
2. 🔜 **Seccomp filters** - Restrict syscalls
3. 🔜 **AppArmor/SELinux profiles** - Mandatory access control
4. 🔜 **Capabilities drop** - Remove unnecessary capabilities

**Code** (future):
```rust
use nix::unistd::{setuid, Uid};

fn execute_unprivileged(task: &Task) -> Result<Output, Error> {
    // Drop to nobody user
    setuid(Uid::from_raw(65534))?;

    // Now execute task with reduced privileges
    Command::new(&task.command).output()
}
```

### 5.5 Data Exfiltration

**Attack**: Malicious AU exfiltrates data to external server.

**Mitigations** (Phase 3):
1. 🔜 **Network isolation** - Block outbound network by default
2. 🔜 **Network allowlist** - Only specific endpoints permitted
3. ✅ **Output size limits** - Limit stdout/stderr size
4. ✅ **Output sanitization** - Redact secrets before logging

**Code** (future):
```rust
// Block network via network namespace
Command::new("unshare")
    .arg("--net")  // New network namespace (no internet)
    .arg("--")
    .arg(&task.command)
    .output()
```

### 5.6 Resource Exhaustion (DoS)

**Attack**: Malicious plan with infinite loop or huge memory allocation.

**Mitigations**:
1. ✅ **Timeout enforcement** - All tasks have max duration
2. ✅ **Connection limits** - Max connections per IP
3. ✅ **Plan size limits** - Max 100 tasks, 1MB plan size
4. 🔜 **Cgroup limits** - CPU, memory limits per task

**Code**:
```rust
// Timeout enforcement
tokio::time::timeout(
    Duration::from_secs(task.timeout_secs.into()),
    execute_task(task)
).await?
```

---

## 6. Future Enhancements

### Phase 2: Cryptographic Plan Signing

- AGX signs plans with Ed25519 private key
- AGW verifies signatures before execution
- Key rotation mechanism
- Revocation list for compromised keys

### Phase 3: Advanced Sandboxing

- **Linux namespaces**: PID, network, mount, user, IPC
- **Seccomp-BPF**: Syscall filtering
- **Cgroups v2**: Resource limits (CPU, memory, I/O)
- **AppArmor profiles**: Mandatory access control
- **gVisor/Firecracker**: Lightweight VM isolation

### Phase 4: Network Security

- **mTLS**: Mutual TLS for AGX ↔ AGQ ↔ AGW
- **Unix domain sockets**: Eliminate network for local communication
- **Network policies**: Kubernetes-style network allowlists
- **VPN/WireGuard**: Encrypted worker communication

### Phase 5: Observability & Forensics

- **Audit logging**: Immutable log of all actions
- **Anomaly detection**: ML-based detection of unusual patterns
- **Intrusion detection**: Alert on suspicious behavior
- **Forensics mode**: Detailed logging for incident response

---

## Related Documentation

- [Security Guidelines](../development/security-guidelines.md) - Development security practices
- [Testing Strategy](../development/testing-strategy.md) - Security testing requirements
- [System Overview](../architecture/system-overview.md) - Overall architecture
- [Worker Registration](../api/worker-registration.md) - Worker authentication flow

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly or on security incidents
**Threat model last reviewed:** 2025-11-17
