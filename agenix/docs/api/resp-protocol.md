# RESP Protocol Specification

**Version:** 1.0
**Status:** Canonical API Specification
**Last Updated:** 2025-11-17

This document defines the RESP (REdis Serialization Protocol) API used for communication between AGEniX components.

---

## Table of Contents

1. [Overview](#overview)
2. [Protocol Choice](#protocol-choice)
3. [Transport](#transport)
4. [Authentication](#authentication)
5. [Command Reference](#command-reference)
6. [Error Handling](#error-handling)
7. [Data Types](#data-types)
8. [Security Considerations](#security-considerations)

---

## 1. Overview

All AGEniX components communicate using **RESP** (REdis Serialization Protocol), a simple, text-based protocol that is:

- **Human-readable**: Easy to debug with telnet or netcat
- **Language-agnostic**: Clients exist for all major languages
- **Well-tested**: Battle-tested in Redis for 15+ years
- **Simple to implement**: Straightforward parser/serializer

### Component Communication

```
AGX (Client) ←→ AGQ (Server) ←→ AGW (Client)
```

- **AGX** acts as a RESP client, submitting plans and querying status
- **AGQ** acts as a RESP server, managing the queue and worker coordination
- **AGW** acts as a RESP client, pulling jobs and reporting status

---

## 2. Protocol Choice

### Why RESP?

**Decision**: Use RESP instead of custom binary protocol, gRPC, or HTTP/REST

**Rationale**:
- **Simplicity**: Minimal parser implementation (&lt;500 LOC)
- **Debuggability**: Text-based, human-readable
- **Tooling**: Redis clients exist for all languages
- **Performance**: Efficient binary-safe encoding
- **No dependencies**: No Protobuf compiler, no HTTP server overhead

**Trade-offs**:
- Not as compact as binary protocols (acceptable for local communication)
- No built-in streaming (can layer on top if needed)
- No schema validation (handled at application layer with JSON schemas)

**See Also**: `docs/adr/0001-resp-protocol.md` (future)

---

## 3. Transport

### Phase 1: TCP Sockets

**Default**: AGQ listens on `127.0.0.1:6380` (configurable via `--port`)

**Format**: RESP messages over TCP

**Example**:
```bash
# Connect to AGQ
telnet 127.0.0.1 6380

# Or with nc
nc 127.0.0.1 6380
```

### Future: Unix Domain Sockets

For enhanced security and performance on single-machine deployments:

```bash
agq --socket /tmp/agq.sock
```

### Future: TLS/mTLS

For distributed deployments across networks:

```bash
agq --tls-cert server.crt --tls-key server.key
```

---

## 4. Authentication

### Session Key Authentication

**All commands** (except `AUTH` itself) require authentication via session key.

#### AUTH Command

**Syntax**: `AUTH <session_key>`

**Parameters**:
- `session_key` (string): 32+ byte cryptographic random key (hex-encoded recommended)

**Response**:
- Success: `+OK`
- Error: `-ERR AUTH requires exactly one argument`
- Error: `-ERR AUTH key cannot be empty`

**Example**:
```resp
Client: *2\r\n$4\r\nAUTH\r\n$64\r\n<64-char-hex-session-key>\r\n
Server: +OK\r\n
```

**Security**:
- Constant-time comparison to prevent timing attacks
- Session key must be 32+ bytes (64+ hex characters recommended)
- No session key should ever appear in logs
- Failed auth attempts should be rate-limited

#### Unauthenticated Access

**Error**: `-ERR NOAUTH Authentication required`

All commands except `AUTH` will return this error if client has not authenticated.

---

## 5. Command Reference

### 5.1 Core Commands

#### PING

**Syntax**: `PING [message]`

**Description**: Test server connectivity and authentication

**Response**:
- No message: `+PONG`
- With message: `<message>` (echoed back)

**Example**:
```resp
Client: *1\r\n$4\r\nPING\r\n
Server: +PONG\r\n

Client: *2\r\n$4\r\nPING\r\n$5\r\nhello\r\n
Server: $5\r\nhello\r\n
```

**Requires Auth**: Yes

---

### 5.2 Data Structure Commands

AGQ implements a subset of Redis data structure commands using embedded redb storage.

#### SET

**Syntax**: `SET <key> <value>`

**Description**: Set string value

**Response**:
- Success: `+OK`

**Example**:
```resp
Client: *3\r\n$3\r\nSET\r\n$6\r\nmykey\r\n$7\r\nmyvalue\r\n
Server: +OK\r\n
```

**Requires Auth**: Yes

---

#### GET

**Syntax**: `GET <key>`

**Description**: Get string value

**Response**:
- Found: `$<length>\r\n<value>\r\n`
- Not found: `$-1\r\n` (nil)

**Example**:
```resp
Client: *2\r\n$3\r\nGET\r\n$6\r\nmykey\r\n
Server: $7\r\nmyvalue\r\n

Client: *2\r\n$3\r\nGET\r\n$8\r\nnotfound\r\n
Server: $-1\r\n
```

**Requires Auth**: Yes

---

#### LPUSH

**Syntax**: `LPUSH <key> <value> [value ...]`

**Description**: Push one or more values to the head of a list

**Response**:
- Success: `:<new_length>\r\n` (integer)

**Example**:
```resp
Client: *3\r\n$5\r\nLPUSH\r\n$6\r\nmylist\r\n$5\r\nvalue\r\n
Server: :1\r\n

Client: *4\r\n$5\r\nLPUSH\r\n$6\r\nmylist\r\n$3\r\ntwo\r\n$5\r\nthree\r\n
Server: :3\r\n
```

**Requires Auth**: Yes

**Note**: Notifies waiting `BRPOP` calls

---

#### RPOP

**Syntax**: `RPOP <key>`

**Description**: Remove and return the tail element of a list

**Response**:
- Found: `$<length>\r\n<value>\r\n`
- Empty: `$-1\r\n` (nil)

**Example**:
```resp
Client: *2\r\n$4\r\nRPOP\r\n$6\r\nmylist\r\n
Server: $5\r\nvalue\r\n

Client: *2\r\n$4\r\nRPOP\r\n$9\r\nemptylist\r\n
Server: $-1\r\n
```

**Requires Auth**: Yes

---

#### BRPOP

**Syntax**: `BRPOP <key> <timeout_seconds>`

**Description**: Blocking pop from tail of list. Waits up to `timeout_seconds` for an element to become available.

**Parameters**:
- `key` (string): List key to pop from
- `timeout_seconds` (integer): Maximum seconds to wait (0 = wait indefinitely)

**Response**:
- Found (immediate or after wait): `$<length>\r\n<value>\r\n`
- Timeout: `$-1\r\n` (nil)

**Example**:
```resp
# Immediate return (list has data)
Client: *3\r\n$5\r\nBRPOP\r\n$6\r\nmylist\r\n$1\r\n5\r\n
Server: $5\r\nvalue\r\n

# Timeout after 2 seconds (list empty)
Client: *3\r\n$5\r\nBRPOP\r\n$9\r\nemptylist\r\n$1\r\n2\r\n
[... 2 seconds pass ...]
Server: $-1\r\n
```

**Requires Auth**: Yes

**Use Case**: AGW workers use `BRPOP queue:ready <timeout>` to pull jobs

**Implementation Notes**:
- Uses async notification mechanism (tokio broadcast channel)
- `LPUSH` to a key wakes all waiting `BRPOP` calls on that key
- Efficient: No polling, workers sleep until job available

---

#### ZADD

**Syntax**: `ZADD <key> <score> <member>`

**Description**: Add member to sorted set with given score

**Response**:
- Success: `:1\r\n` (1 if new, 0 if updated)

**Example**:
```resp
Client: *4\r\n$4\r\nZADD\r\n$9\r\nscheduled\r\n$10\r\n1700000000\r\n$10\r\njob-abc123\r\n
Server: :1\r\n
```

**Requires Auth**: Yes

**Use Case**: Scheduled jobs (score = Unix timestamp)

---

#### ZRANGEBYSCORE

**Syntax**: `ZRANGEBYSCORE <key> <min_score> <max_score>`

**Description**: Return members with scores in range [min, max]

**Response**:
- Array of members: `*<count>\r\n$<len1>\r\n<member1>\r\n$<len2>\r\n<member2>\r\n...`
- Empty: `*0\r\n`

**Example**:
```resp
Client: *4\r\n$13\r\nZRANGEBYSCORE\r\n$9\r\nscheduled\r\n$1\r\n0\r\n$10\r\n1700000000\r\n
Server: *2\r\n$10\r\njob-abc123\r\n$10\r\njob-def456\r\n
```

**Requires Auth**: Yes

**Use Case**: Retrieve jobs scheduled before a certain time

---

### 5.3 AGQ-Specific Commands (Phase 2+)

These commands are AGQ-specific extensions for plan and job management.

#### PLAN.SUBMIT

**Status**: Planned (not yet implemented)

**Syntax**: `PLAN.SUBMIT <plan_json>`

**Description**: Store a reusable Plan definition

**Parameters**:
- `plan_json` (string): JSON-encoded Plan per [job-schema.md](../architecture/job-schema.md)

**Response**:
- Success: `+OK plan_id=<uuid>`
- Error: `-ERR Invalid plan schema: <details>`

**Example**:
```resp
Client: *2\r\n$11\r\nPLAN.SUBMIT\r\n$123\r\n{"plan_id":"uuid-5678",...}\r\n
Server: +OK plan_id=uuid-5678\r\n
```

**Requires Auth**: Yes

**Validation**:
- JSON schema compliance
- Task numbering (1-based, contiguous)
- Valid `input_from_task` references
- Maximum task count (default 100)

---

#### ACTION.SUBMIT

**Status**: Planned (not yet implemented)

**Syntax**: `ACTION.SUBMIT <action_json>`

**Description**: Create multiple Jobs from one Plan with different inputs

**Parameters**:
- `action_json` (string): JSON with `plan_id` and array of `inputs`

**Response**:
- Success: `+OK action_id=<uuid> jobs_created=<count>`
- Error: `-ERR Plan not found: <plan_id>`

**Example**:
```json
{
  "action_id": "uuid-1234",
  "plan_id": "uuid-5678",
  "inputs": [
    {"file": "data1.txt"},
    {"file": "data2.txt"},
    {"file": "data3.txt"}
  ]
}
```

**Response**:
```resp
Server: +OK action_id=uuid-1234 jobs_created=3\r\n
```

**Requires Auth**: Yes

**Behavior**:
- Creates N Jobs (one per input)
- Each Job references the same `plan_id`
- All Jobs enqueued to `queue:ready`
- Returns Action ID for tracking

---

#### JOB.STATUS

**Status**: Planned (not yet implemented)

**Syntax**: `JOB.STATUS <job_id>`

**Description**: Query Job execution status

**Response**:
- Success: JSON with job metadata
- Not found: `$-1\r\n` (nil)

**Example**:
```resp
Client: *2\r\n$10\r\nJOB.STATUS\r\n$10\r\njob-abc123\r\n
Server: $87\r\n{"job_id":"job-abc123","status":"completed","started_at":"...","completed_at":"..."}\r\n
```

**Requires Auth**: Yes

**Returned Fields**:
- `job_id`: Unique job identifier
- `action_id`: Parent action (if part of Action)
- `status`: `pending` | `running` | `completed` | `failed` | `dead`
- `created_at`: ISO 8601 timestamp
- `started_at`: ISO 8601 timestamp (null if not started)
- `completed_at`: ISO 8601 timestamp (null if not completed)
- `stdout`: Task outputs (if completed)
- `stderr`: Task errors (if failed)
- `exit_code`: Final exit code

---

#### JOB.LIST

**Status**: Planned (not yet implemented)

**Syntax**: `JOB.LIST <action_id>`

**Description**: List all Jobs for a given Action

**Response**:
- Array of job IDs: `*<count>\r\n$<len1>\r\n<job_id1>\r\n...`
- Empty: `*0\r\n`

**Example**:
```resp
Client: *2\r\n$8\r\nJOB.LIST\r\n$10\r\naction-123\r\n
Server: *3\r\n$10\r\njob-abc123\r\n$10\r\njob-def456\r\n$10\r\njob-ghi789\r\n
```

**Requires Auth**: Yes

---

#### WORKER.REGISTER

**Status**: Planned (not yet implemented)

**Syntax**: `WORKER.REGISTER <worker_json>`

**Description**: Register worker with AGQ, providing capabilities

**Parameters**:
- `worker_json` (string): JSON with worker ID, capabilities, heartbeat interval

**Response**:
- Success: `+OK worker_id=<id> heartbeat_interval=<seconds>`
- Error: `-ERR Invalid worker registration: <details>`

**Example**:
```json
{
  "worker_id": "worker-local-001",
  "capabilities": ["sort", "uniq", "agx-ocr"],
  "max_concurrent_jobs": 4
}
```

**Response**:
```resp
Server: +OK worker_id=worker-local-001 heartbeat_interval=30\r\n
```

**Requires Auth**: Yes

**See Also**: [worker-registration.md](./worker-registration.md)

---

## 6. Error Handling

### Error Response Format

All errors use RESP error format: `-ERR <message>\r\n`

### Common Errors

| Error Code | Message | Cause |
|------------|---------|-------|
| `NOAUTH` | `Authentication required` | Client has not authenticated with `AUTH` |
| `ERR` | `Invalid arguments` | Command syntax error |
| `ERR` | `Unknown command` | Command not recognized |
| `ERR` | `Message too large` | Command exceeds max size (default 10MB) |
| `ERR` | `Invalid plan schema` | Plan JSON validation failed |
| `ERR` | `Plan not found` | Referenced `plan_id` doesn't exist |
| `ERR` | `Job not found` | Referenced `job_id` doesn't exist |

### Error Examples

```resp
# Not authenticated
Client: *1\r\n$4\r\nPING\r\n
Server: -ERR NOAUTH Authentication required\r\n

# Invalid command
Client: *1\r\n$7\r\nINVALID\r\n
Server: -ERR Unknown command 'INVALID'\r\n

# Invalid arguments
Client: *1\r\n$4\r\nAUTH\r\n
Server: -ERR AUTH requires exactly one argument\r\n
```

---

## 7. Data Types

### RESP Data Type Encoding

AGQ implements the following RESP data types:

#### Simple String

**Format**: `+<string>\r\n`

**Example**: `+OK\r\n`

**Use**: Status responses

---

#### Error

**Format**: `-<error_message>\r\n`

**Example**: `-ERR NOAUTH Authentication required\r\n`

**Use**: Error responses

---

#### Integer

**Format**: `:<number>\r\n`

**Example**: `:42\r\n`

**Use**: Counts, lengths, scores

---

#### Bulk String

**Format**: `$<length>\r\n<data>\r\n`

**Example**: `$5\r\nhello\r\n`

**Special**: `$-1\r\n` = nil (null)

**Use**: String values, JSON payloads

---

#### Array

**Format**: `*<count>\r\n<element1><element2>...`

**Example**:
```resp
*2\r\n
$3\r\nfoo\r\n
$3\r\nbar\r\n
```

**Special**: `*0\r\n` = empty array

**Use**: Command arguments, multi-value responses

---

## 8. Security Considerations

### 8.1 Session Key Management

**Generation**:
```rust
use ring::rand::{SecureRandom, SystemRandom};

let rng = SystemRandom::new();
let mut key = [0u8; 32];
rng.fill(&mut key)?;
let key_hex = hex::encode(key); // 64 hex chars
```

**Storage**:
- AGQ: Store hex-encoded key in memory only (no disk persistence)
- AGX/AGW: Read from environment variable or secure config file
- Never log session keys

**Comparison**:
```rust
use subtle::ConstantTimeEq;

if provided_key.ct_eq(&stored_key).into() {
    // Authenticated
}
```

### 8.2 Rate Limiting

**AUTH Command**:
- Limit failed attempts per IP: 5 per minute
- Exponential backoff on repeated failures
- Log suspicious activity

**All Commands**:
- Connection limit per IP: 100 concurrent connections
- Command rate limit: 10,000 commands/second per connection

### 8.3 Input Validation

**Command Parsing**:
- Maximum command size: 10 MB (configurable)
- Maximum array elements: 1,000,000
- Maximum nesting depth: 7 (RESP spec limit)

**Plan Validation**:
- Maximum plan size: 1 MB
- Maximum tasks per plan: 100 (configurable)
- JSON schema enforcement

### 8.4 Network Security

**Phase 1** (current):
- Listen on `127.0.0.1` only (localhost)
- No encryption (local-only communication)

**Phase 2** (future):
- Unix domain sockets (eliminate network exposure)
- TLS for distributed deployments
- mTLS for mutual authentication

---

## 9. Implementation Notes

### AGQ Server

**Language**: Rust
**Framework**: Tokio (async runtime)
**Parser**: Custom RESP parser (see `agq/src/protocol/`)
**Storage**: Embedded redb (see ADR 0002-embedded-redb.md)

**Key Files**:
- `src/server.rs` - RESP server and command dispatcher
- `src/protocol/parser.rs` - RESP message parser
- `src/storage/db.rs` - Embedded redb wrapper

### Client Libraries

**Recommended**:
- **Rust**: `redis` crate (compatible with AGQ's RESP implementation)
- **Python**: `redis-py`
- **Node.js**: `ioredis`
- **Go**: `go-redis`

**Example** (Python):
```python
import redis

client = redis.Redis(host='127.0.0.1', port=6380, decode_responses=True)
client.execute_command('AUTH', session_key)
client.ping()  # 'PONG'
```

---

## 10. Future Extensions

### Batch Commands

Support pipelining for reduced round trips:

```resp
Client: *2\r\n$4\r\nPING\r\n$5\r\nhello\r\n*2\r\n$4\r\nPING\r\n$5\r\nworld\r\n
Server: $5\r\nhello\r\n$5\r\nworld\r\n
```

### Pub/Sub

For real-time job status updates:

```resp
Client: *2\r\n$9\r\nSUBSCRIBE\r\n$11\r\njob:updates\r\n
Server: *3\r\n$9\r\nsubscribe\r\n$11\r\njob:updates\r\n:1\r\n
```

### Transactions

For atomic multi-command operations:

```resp
Client: *1\r\n$5\r\nMULTI\r\n
Server: +OK\r\n
Client: *3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
Server: +QUEUED\r\n
Client: *1\r\n$4\r\nEXEC\r\n
Server: *1\r\n+OK\r\n
```

---

## Related Documentation

- [AGQ Endpoints](./agq-endpoints.md) - Detailed AGQ-specific command reference
- [Worker Registration](./worker-registration.md) - AGW registration protocol
- [Job Schema](../architecture/job-schema.md) - Plan/Job structure
- [Zero-Trust Execution](../zero-trust/zero-trust-execution.md) - Security model

---

**Maintained by:** AGX Core Team
**Review cycle:** Quarterly or on protocol changes
**Questions?** Consult RESP specification at https://redis.io/docs/reference/protocol-spec/
