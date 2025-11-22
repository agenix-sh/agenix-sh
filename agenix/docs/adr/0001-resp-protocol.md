# 0001. Use RESP Protocol for Component Communication

**Date:** 2025-11-17
**Status:** Accepted
**Deciders:** AGX Core Team
**Tags:** architecture, protocol, infrastructure

---

## Context

AGEniX requires a communication protocol for inter-component messaging:
- AGX (planner) → AGQ (queue manager): Submit plans, query job status
- AGW (worker) → AGQ (queue manager): Register, pull jobs, report results

### Requirements

1. **Simple to implement**: Minimize complexity in Rust implementations
2. **Debuggable**: Human-readable for development and troubleshooting
3. **Efficient**: Low overhead for local communication
4. **Well-tested**: Battle-tested in production environments
5. **Binary-safe**: Support arbitrary binary data (future: model weights)
6. **Blocking operations**: Support for BRPOP-style blocking job pulls
7. **Ecosystem compatibility**: Existing client libraries for multiple languages

### Constraints

- Must work over TCP sockets (Phase 1) and Unix domain sockets (Phase 2+)
- Must support session-key based authentication
- Must handle both synchronous and asynchronous operations
- Must be implementable in pure Rust with minimal dependencies

---

## Decision

**We will use RESP (REdis Serialization Protocol) for all AGEniX component communication.**

RESP is the wire protocol used by Redis, a well-established text-based protocol that meets all our requirements.

---

## Alternatives Considered

### Option 1: Custom Binary Protocol

**Pros:**
- Optimal for specific use case
- Maximum performance (no overhead)
- Full control over features

**Cons:**
- Requires custom parser implementation
- No existing client libraries
- Higher risk of bugs in custom implementation
- Difficult to debug (binary format)
- No tooling support (can't use telnet/nc for testing)

### Option 2: gRPC / Protocol Buffers

**Pros:**
- Industry standard for RPC
- Schema validation via Protobuf
- HTTP/2 multiplexing
- Streaming support

**Cons:**
- Requires Protobuf compiler and code generation
- Heavyweight for local communication
- More complex to debug (binary + HTTP/2)
- Adds significant dependencies
- Overkill for simple request/response patterns

### Option 3: HTTP/REST + JSON

**Pros:**
- Familiar to most developers
- JSON is human-readable
- Existing HTTP client libraries

**Cons:**
- No native support for blocking operations (no HTTP/1.1 equivalent to BRPOP)
- Higher overhead (HTTP headers, JSON parsing)
- Less efficient for simple key-value operations
- Requires HTTP server implementation

### Option 4: MessagePack over TCP

**Pros:**
- Efficient binary format
- Smaller than JSON
- Language-agnostic

**Cons:**
- Binary format makes debugging harder
- Still requires custom framing protocol
- No built-in blocking operation support
- Less ecosystem support than RESP

### Decision Rationale

RESP chosen because:

1. **Proven at scale**: Powers Redis, one of the most widely deployed data stores
2. **Simple parser**: ~300 lines of Rust code for full implementation
3. **Human-readable**: Text-based, can use `telnet`/`nc` for manual testing
4. **Blocking support**: Native `BRPOP` command for efficient job pulling
5. **Ecosystem**: Redis client libraries in every language work with AGQ
6. **Binary-safe**: Bulk strings support arbitrary binary data
7. **Minimal dependencies**: No Protobuf, no code generation, no HTTP stack

---

## Consequences

### Positive

- **Fast development**: RESP parser implemented in &lt;1 day
- **Easy debugging**: Can manually test with `redis-cli`, `telnet`, or `nc`
- **Client compatibility**: Python/Node.js/Go/Rust Redis clients work out-of-box
- **Well-documented**: RESP specification is clear and complete
- **Efficient blocking**: `BRPOP` provides efficient job queue semantics
- **Future-proof**: Can add custom commands (e.g., `PLAN.SUBMIT`) while staying compatible

### Negative

- **Not a "standard" RPC protocol**: Not gRPC/HTTP, may confuse some developers
- **Text overhead**: Less compact than pure binary (acceptable for local communication)
- **Redis association**: Some may assume AGQ *is* Redis (documentation must clarify)
- **Limited type system**: No schema validation at protocol level (must validate in application)

### Neutral

- **Redis-like API**: Familiar to those who know Redis, learning curve for others
- **Custom commands**: Need to namespace custom commands (e.g., `PLAN.*`, `JOB.*`)

---

## Implementation Notes

### Phase 1 (Current)

- TCP sockets on `127.0.0.1:6380`
- Session-key authentication via `AUTH` command
- Implement core Redis commands: `PING`, `SET`, `GET`, `LPUSH`, `RPOP`, `BRPOP`, `ZADD`, `ZRANGEBYSCORE`
- Custom commands: `PLAN.SUBMIT`, `JOB.STATUS`, `WORKER.REGISTER`, `WORKER.HEARTBEAT`

### Phase 2 (Future)

- Unix domain sockets for localhost (eliminate network stack)
- TLS support for distributed deployments
- Additional custom commands as needed

### Key Files

- `agq/src/protocol/parser.rs`: RESP message parser
- `agq/src/protocol/serializer.rs`: RESP message serializer
- `agq/src/server.rs`: RESP server and command dispatcher

---

## Related Decisions

- [ADR-0002](./0002-embedded-redb.md): Embedded redb storage (implements Redis-like data structures)
- Future ADR: Network security and TLS configuration

---

## References

- [RESP Specification](https://redis.io/docs/reference/protocol-spec/)
- [AGEniX RESP Protocol Documentation](../api/resp-protocol.md)
- [AGQ Endpoints Documentation](../api/agq-endpoints.md)
- [Redis Protocol Design Philosophy](https://redis.io/topics/protocol)
