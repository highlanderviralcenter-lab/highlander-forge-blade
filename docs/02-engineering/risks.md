# Risk Matrix

> Identified risks with probability, impact, mitigation strategy, and sprint allocation.

---

## Table of Contents

- [Risk Register](#risk-register)
- [Risk Heat Map](#risk-heat-map)
- [Mitigation Details](#mitigation-details)
- [Contingency Plans](#contingency-plans)

---

## Risk Register

| ID | Risk | Probability | Impact | Strategy | Sprint | Owner |
|----|------|:-----------:|:------:|----------|--------|-------|
| **R01** | WMI inaccessible in corporate environments (GPO restrictions) | High | High | Mitigate | B1 | Core Team |
| **R02** | DISM fails without internet (source files unavailable) | Medium | Medium | Mitigate | D3 | Core Team |
| **R03** | Antivirus / Windows Defender false positive | High | High | Mitigate | E6 | Security |
| **R04** | State JSON corrupted between phases | Low | High | Mitigate | A6 | Core Team |
| **R05** | User closes terminal during execution | Medium | Medium | Accept | B7 | UX Team |
| **R06** | Blake3 scan interrupted (hours-long operation) | High | Low | Mitigate | G1 | Storage Team |
| **R07** | Task Scheduler fails to execute Phase 5 (permissions, policy) | Medium | High | Mitigate | D1 | Core Team |
| **R08** | App update breaks compatibility with old state | Medium | Medium | Mitigate | A6 | Core Team |
| **R09** | Credential Manager credential deleted | Low | High | Mitigate | E2 | Security |
| **R10** | iced_aw incompatible with iced 0.13 | Medium | Medium | Mitigate | F1 | GUI Team |
| **R11** | Update public key compromised | Low | Critical | Mitigate | E6 | Security |
| **R12** | CI Linux cannot test Windows-specific code | High | Medium | Mitigate | A3 | DevOps |
| **R13** | Windows API changes in future Windows versions | Low | Medium | Monitor | Ongoing | Core Team |
| **R14** | Dependency crate yanked or unmaintained | Medium | Medium | Mitigate | Ongoing | DevOps |
| **R15** | Memory leak in long-running TUI | Low | Medium | Prevent | B7 | Core Team |

---

## Risk Heat Map

```mermaid
quadrantChart
    title Risk Heat Map
    x-axis Low Impact --> High Impact
    y-axis Low Probability --> High Probability
    quadrant-1 "Monitor"
    quadrant-2 "Mitigate Urgently"
    quadrant-3 "Accept"
    quadrant-4 "Mitigate"

    "R01 WMI": [0.9, 0.8]
    "R02 DISM": [0.5, 0.5]
    "R03 AV": [0.9, 0.9]
    "R04 Corrupt": [0.2, 0.8]
    "R05 Close": [0.5, 0.5]
    "R06 Blake3": [0.8, 0.3]
    "R07 Task": [0.6, 0.7]
    "R08 State": [0.5, 0.5]
    "R09 Cred": [0.2, 0.8]
    "R10 iced": [0.5, 0.5]
    "R11 Key": [0.1, 1.0]
    "R12 CI": [0.7, 0.6]
    "R13 API": [0.4, 0.3]
    "R14 Dep": [0.5, 0.5]
    "R15 Memory": [0.4, 0.2]
```

---

## Mitigation Details

### R01: WMI Inaccessible

**Detection**: WMI query returns WBEM_E_ACCESS_DENIED or timeout after 30s

**Mitigation**:
1. Implement SystemInfoProvider trait with WMI implementation
2. Implement RegistrySystemInfoProvider fallback
3. Auto-detect WMI availability on startup (ping Win32_OperatingSystem)
4. Log fallback usage with WARN level

```rust
pub struct FallbackSystemInfoProvider {
    primary: Box<dyn SystemInfoProvider>,
    fallback: Box<dyn SystemInfoProvider>,
}

impl SystemInfoProvider for FallbackSystemInfoProvider {
    fn cpu(&self) -> Result<CpuInfo, CoreError> {
        self.primary.cpu().or_else(|e| {
            log::warn!("WMI failed ({}), using registry fallback", e);
            self.fallback.cpu()
        })
    }
}
```

---

### R03: Antivirus False Positive

**Timeline**:
- Week 1-2: Submit to Microsoft for SmartScreen reputation (requires code signing)
- Week 3-4: Submit to major AV vendors (Avast, AVG, BitDefender, Kaspersky, McAfee)
- Week 5-6: Monitor VirusTotal (target: 0/70 detections)

**Prevention**:
- Code signing with EV certificate (not just standard)
- Manifest with requestedExecutionLevel="requireAdministrator"
- Avoid packers/UPX (common false positive trigger)
- Clear, descriptive file properties (Company, Product, Version)

---

### R04: State JSON Corrupted

**Defense in Depth**:

```
Layer 1: Atomic writes (temp file + rename)
Layer 2: CRC32 checksum sidecar file
Layer 3: Automatic backup (estado_manutencao.json.bak)
Layer 4: Schema version validation
Layer 5: Graceful degradation (re-run Phase 1 if unrecoverable)
```

---

### R11: Update Public Key Compromised

**Scenario**: Attacker gains access to update private key, signs malicious update

**Mitigation**:
- Key embedded at compile-time (include_bytes!), never fetched from network
- Key rotation: new key pair for each major version (v3 to v4)
- Old key revocation list embedded in binary
- Manual verification: user can verify signature independently with signtool or openssl

---

## Contingency Plans

### If R03 (AV Block) Occurs After Release

1. Immediate: Publish signed hash on GitHub Releases for manual verification
2. Short-term: Contact AV vendors with false positive report + signed binary
3. Medium-term: Consider alternative distribution (Chocolatey, Scoop) with established reputation
4. Long-term: If persistent, pivot to enterprise-only distribution (MSI via GPO, no public download)

### If R07 (Task Scheduler Failure) Occurs

1. Detect on next run: check if Phase 5 should have run but didn't
2. Prompt user: "Phase 5 not detected. Run now? [Y/N]"
3. Alternative: Use Windows Service (requires install) instead of Task Scheduler
4. Fallback: Manual Phase 5 execution via hfb --auto-phase 5

---

*Last updated: 2026-06-20 | Document version: 1.0*
