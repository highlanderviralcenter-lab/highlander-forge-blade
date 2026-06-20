# ADR-004: Windows Credential Manager for AES Key Storage

## Status

Accepted

## Context

HFB encrypts the maintenance state file with AES-256-GCM. The encryption key must be:

1. Protected by the OS (not in plaintext on disk)
2. Persistent across application reinstalls
3. Not tied to hardware (survives motherboard replacement, VM cloning)
4. Recoverable if lost (with user impact)

## Decision

Store the AES-256-GCM key in the Windows Credential Manager as a `CRED_TYPE_GENERIC` credential with `CRED_PERSIST_LOCAL_MACHINE` persistence.

## Consequences

### Positive

- Protected by Windows DPAPI (encrypted at rest)
- Survives application reinstall
- Machine-scoped (not user-scoped), works for system tasks
- Standard Windows API, no third-party dependencies
- User can backup/restore via Credential Manager UI

### Negative

- Windows-only (acceptable for v3.0)
- Credential can be deleted by user or admin policy
- No automatic backup (user must manually export)
- Lost credential = lost access to encrypted states

## Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **Hardware-derived key** | No storage needed | HW changes = key lost; VMs share key | ❌ Rejected |
| **File-based key (encrypted)** | Simple | DPAPI user-scoped; fails for system tasks | ❌ Rejected |
| **TPM/Secure Enclave** | Hardware-backed | Complex; not universal | ⚠️ Future v3.2+ |
| **Credential Manager** | OS-protected, standard | Can be deleted | ✅ Accepted |

## Implementation

```rust
const CRED_TARGET_NAME: &str = "HighlanderForgeBlade:StateKey";

pub fn get_or_create_key() -> Result<[u8; 32], CredentialError> {
    if let Ok(key) = read_credential(CRED_TARGET_NAME) {
        return Ok(key);
    }

    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    write_credential(CRED_TARGET_NAME, &key)?;

    Ok(key)
}
```

## Recovery

If credential is deleted:
1. Detect on next run (`CredReadW` returns `ERROR_NOT_FOUND`)
2. Generate new key
3. Log warning: "Previous encrypted states are unrecoverable"
4. Continue operation (new states will use new key)

---

*Date: 2026-06-18 | Author: Security Team*
