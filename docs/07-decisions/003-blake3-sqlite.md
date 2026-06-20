# ADR-003: SQLite for Blake3 File Index

## Status

Accepted (Future: v3.2.0)

## Context

The Storage Pro module (v3.2.0) will index millions of files with Blake3 hashes. The requirements are:

1. Store millions of file records (path, size, modified time, hash)
2. Query duplicates by hash + size
3. Resume interrupted scans
4. Atomic batch inserts
5. Complex filtering (by size, by path pattern, by date)

## Decision

Use SQLite with WAL (Write-Ahead Logging) mode for the Blake3 file index. Use JSON for the core maintenance state (see [ADR-002](002-state-versioning.md)).

## Consequences

### Positive

- Handles millions of rows efficiently
- SQL queries for complex analysis
- ACID transactions for batch operations
- WAL mode allows concurrent reads during writes
- Resume capability via checkpoint table
- No separate server process

### Negative

- Adds `sqlx` dependency (with `sqlite` feature)
- Requires migration management for schema changes
- File locking on network drives (use local disk only)
- Not human-readable (unlike JSON state)

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **JSON Lines** | Human-readable | Slow queries; no transactions | ❌ Rejected |
| **CSV** | Simple | No types; slow; no indexes | ❌ Rejected |
| **LevelDB/RocksDB** | Fast KV | No SQL; complex queries hard | ❌ Rejected |
| **PostgreSQL (embedded)** | Full SQL | Heavy; overkill for local tool | ❌ Rejected |
| **SQLite** | SQL, ACID, lightweight, WAL | Schema migrations | ✅ Accepted |

## Schema

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    size INTEGER NOT NULL,
    modified INTEGER NOT NULL,
    blake3_hash BLOB NOT NULL,
    indexed_at INTEGER NOT NULL
);

CREATE INDEX idx_hash ON files(blake3_hash);
CREATE INDEX idx_path ON files(path);
CREATE INDEX idx_size ON files(size);
```

## Implementation Notes

- Use `sqlx::sqlite::SqliteConnectOptions` with `journal_mode(Wal)`
- Implement `insert_or_update` with `ON CONFLICT(path) DO UPDATE`
- Checkpoint every 1000 files during scan
- Use `fd_lock` to prevent concurrent access from multiple HFB instances

---

*Date: 2026-06-18 | Author: Storage Team*
