# 0002. Use Embedded redb for AGQ Storage

**Date:** 2025-11-17
**Status:** Accepted
**Deciders:** AGX Core Team
**Tags:** architecture, storage, dependencies, infrastructure

---

## Context

AGQ (queue manager) requires persistent storage for:
- Plan definitions (reusable JSON structures)
- Job metadata and status
- Worker registrations and heartbeats
- Queue state (pending/scheduled jobs)
- Sorted sets for scheduling (jobs by timestamp)

### Requirements

1. **Embedded**: No external database server to install/manage
2. **Single-file**: Simple deployment and backup
3. **ACID transactions**: Ensure data consistency
4. **Rust-native**: First-class Rust support
5. **Zero external dependencies**: No runtime dependencies
6. **Concurrent access**: Multiple readers, single writer
7. **Efficient key-value operations**: Fast lookups, range scans
8. **Small footprint**: Minimal memory and disk usage

### Constraints

- Must support Linux and macOS (cross-platform)
- Must work on both x86_64 and ARM64 architectures
- Must be actively maintained (security updates)
- Must have clear licensing (compatible with MIT/Apache-2.0)
- Must support data structures: strings, hashes, lists, sorted sets

---

## Decision

**We will use redb (https://github.com/cberner/redb) as the embedded storage engine for AGQ.**

redb is a pure Rust embedded key-value database inspired by LMDB, providing ACID transactions in a zero-dependency, single-file format.

---

## Alternatives Considered

### Option 1: SQLite

**Pros:**
- Industry standard, battle-tested
- SQL query language (complex queries)
- Excellent documentation and tooling
- Wide ecosystem support
- Great debugging tools (sqlite3 CLI)

**Cons:**
- Requires C library (FFI overhead)
- SQL is overkill for key-value operations
- Larger binary size
- More complex API for simple KV needs
- Schema migrations more complex

### Option 2: sled

**Pros:**
- Pure Rust, zero dependencies
- Fast embedded database
- ACID transactions
- Active development (at time of initial consideration)

**Cons:**
- Project is in maintenance mode (no new features)
- Has had stability issues in past
- More complex API than redb
- Larger memory footprint

### Option 3: RocksDB (via rust-rocksdb)

**Pros:**
- Used by major distributed systems (Kafka, CockroachDB)
- Highly optimized for write-heavy workloads
- Column families for data organization

**Cons:**
- C++ library (requires linking)
- Large binary size (~10MB+)
- Overkill for single-node use case
- More complex configuration
- External dependency (C++ build chain)

### Option 4: In-Memory HashMap + Periodic Snapshots

**Pros:**
- Simplest implementation
- Fastest reads/writes (no disk I/O)
- No dependencies

**Cons:**
- Data loss on crash (unless frequent snapshots)
- No ACID guarantees
- Memory-limited capacity
- Snapshot serialization adds complexity

### Option 5: Actual Redis

**Pros:**
- Full Redis feature set
- Well-known and trusted
- Excellent tooling

**Cons:**
- Separate process to manage
- External dependency (violates zero-dependency goal)
- Adds operational complexity
- Requires network communication even for local use

### Decision Rationale

redb chosen because:

1. **Pure Rust**: No C/C++ dependencies, easier cross-compilation
2. **Single file**: Entire database in one `.redb` file
3. **ACID**: Full transactional guarantees
4. **Zero dependencies**: No runtime requirements
5. **Simple API**: Straightforward key-value operations
6. **Small footprint**: Adds minimal binary size (~200KB)
7. **Active maintenance**: Regular updates and security patches
8. **MIT licensed**: Compatible with our dual MIT/Apache-2.0 license

Compared to alternatives:
- **vs SQLite**: No FFI overhead, simpler for KV use case
- **vs sled**: More actively maintained, simpler API
- **vs RocksDB**: No C++ dependency, smaller binary
- **vs in-memory**: Persistence and ACID guarantees
- **vs actual Redis**: No external process, embedded

---

## Consequences

### Positive

- **Zero-dependency deployment**: Single AGQ binary, no database installation
- **Simple backup**: Copy single `.redb` file
- **Fast development**: Simple API, easy to implement Redis-like commands
- **Cross-platform**: Works on macOS/Linux, x86_64/ARM64 out-of-box
- **Type-safe**: Rust's type system prevents many DB bugs
- **Small binary**: ~200KB added to AGQ binary size
- **Predictable performance**: No network latency, no external process overhead

### Negative

- **Limited ecosystem**: Smaller community than SQLite/RocksDB
- **No SQL**: Can't do complex joins/queries (acceptable for our use case)
- **Single-writer**: One writer at a time (acceptable for single AGQ instance)
- **Younger project**: Less battle-tested than SQLite (but actively maintained)

### Neutral

- **Custom data structures**: Must implement Redis-like structures (lists, sorted sets) on top of KV
- **Migration path**: If we outgrow redb, can migrate to distributed DB later

---

## Implementation Notes

### redb Wrapper

Create `agq/src/storage/db.rs` that:
- Wraps redb with Redis-compatible API
- Implements: strings, lists, sorted sets, hashes
- Handles async notification for `BRPOP` (using tokio broadcast channels)

### Data Layout

```
# Strings
key:<name> → value

# Lists
list:<name>:len → count
list:<name>:0 → first_element
list:<name>:1 → second_element

# Sorted Sets
zset:<name>:score:<score>:<member> → ""  (for range scans)
zset:<name>:member:<member> → score       (for membership checks)

# Jobs
job:<job_id>:metadata → JSON
job:<job_id>:status → "pending"|"running"|"completed"|"failed"
job:<job_id>:worker → worker_id

# Plans
plan:<plan_id> → JSON

# Workers
worker:<worker_id>:metadata → JSON
worker:<worker_id>:alive → timestamp (with TTL via separate cleanup)
```

### TTL Implementation

redb doesn't have built-in TTL, so implement via:
- Separate background task that scans expired keys every 30 seconds
- Store expiry timestamps: `expiry:<timestamp>:<key> → ""`
- Scan `expiry` prefix for timestamps < now, delete associated keys

### File Location

- Default: `~/.agq/data.redb`
- Configurable via: `agq --data-dir /custom/path`

---

## Related Decisions

- [ADR-0001](./0001-resp-protocol.md): RESP protocol (redb implements Redis-like data structures)
- Future ADR: Distributed AGQ (when to migrate to distributed DB)

---

## References

- [redb GitHub Repository](https://github.com/cberner/redb)
- [redb Documentation](https://docs.rs/redb/)
- [Comparison: sled vs redb](https://github.com/cberner/redb/blob/master/docs/comparison_vs_sled.md)
- [LMDB Design](http://www.lmdb.tech/doc/) - redb is inspired by LMDB architecture
