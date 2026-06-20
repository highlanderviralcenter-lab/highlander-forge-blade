# ADR-002: JSON with Mandatory Schema Versioning

## Status

Accepted

## Context

HFB saves execution state to disk (`estado_manutencao.json`) to survive crashes and reboots. As the application evolves, the state format may change. We need a strategy that:

1. Allows forward compatibility (new app reads old state)
2. Prevents silent data corruption
3. Is simple to implement and debug
4. Works on Windows without additional dependencies

## Decision

Use JSON with a mandatory `schema_version` field. Implement automatic migration on load. Use atomic writes (temp file + rename) for durability.

## Consequences

### Positive

- Human-readable for debugging
- Easy to inspect with any text editor
- Simple migration functions (v0→v1, v1→v2)
- No database dependency for core state
- Atomic writes prevent corruption during power loss

### Negative

- Larger file size than binary formats
- Slower than SQLite for large datasets (not an issue for state)
- No built-in query capability
- Schema evolution requires manual migration code

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **SQLite for all state** | ACID, queryable | Overkill for small state; adds dependency | ❌ Rejected for core state |
| **Protobuf / MessagePack** | Compact, fast | Binary, hard to debug; no schema migration | ❌ Rejected |
| **TOML** | Human-readable | Slower parsing; less tooling | ❌ Rejected |
| **JSON + schema_version** | Readable, simple, migratable | Larger size | ✅ Accepted |
| **SQLite for Blake3 index** | Handles millions of rows | See [ADR-003](003-blake3-sqlite.md) | ✅ Accepted for index |

## Implementation

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn load_state(path: &Path) -> Result<StateFile, StateError> {
    let content = std::fs::read_to_string(path)?;
    let raw: serde_json::Value = serde_json::from_str(&content)?;

    let version = raw.get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    match version {
        0 => migrate_v0_to_v1(raw),
        1 => serde_json::from_value(raw).map_err(StateError::Parse),
        _ => Err(StateError::UnsupportedVersion(version)),
    }
}
```

## Migration Rules

1. Always add `schema_version` to new formats
2. Never remove fields (deprecate instead)
3. Provide migration for N-2 versions (e.g., v3 supports v1, v2)
4. Test migrations with fixture files in `tests/fixtures/`

---

*Date: 2026-06-18 | Author: Core Team*
