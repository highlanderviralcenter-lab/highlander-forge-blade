# Requirements Specification

> Functional and non-functional requirements with traceability to use cases.

---

## Table of Contents

- [Functional Requirements](#functional-requirements)
- [Non-Functional Requirements](#non-functional-requirements)
- [Traceability Matrix](#traceability-matrix)

---

## Functional Requirements

### FR-01: Phase Execution

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-01.1 | Execute Phase 1 (Audit) independently | Must | UC-02 |
| FR-01.2 | Execute Phases 1 to 5 sequentially with user confirmation gates | Must | UC-01 |
| FR-01.3 | Resume from any phase after unexpected termination | Must | UC-03 |
| FR-01.4 | Skip already-completed phases on resume | Must | UC-03 |
| FR-01.5 | Execute Phase 5 automatically via Task Scheduler after reboot | Must | UC-03 |

### FR-02: Data Collection

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-02.1 | Collect CPU, RAM, disk, GPU, motherboard, temperature data | Must | UC-01, UC-02 |
| FR-02.2 | Enumerate installed software via Registry + Win32_Product (optional) | Must | UC-02 |
| FR-02.3 | Query Windows Update Agent for pending/installed updates | Must | UC-02 |
| FR-02.4 | List all services with third-party identification | Must | UC-02 |
| FR-02.5 | Read Run keys and service entries from Registry | Must | UC-02 |
| FR-02.6 | Collect system and user environment variables | Must | UC-02 |
| FR-02.7 | Fallback to registry when WMI is restricted | Should | UC-02 |

### FR-03: Maintenance Operations

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-03.1 | Clean temporary files (Windows Temp, User Temp, Prefetch) | Must | UC-01 |
| FR-03.2 | Clean browser caches (Chrome, Edge, Firefox) | Must | UC-01 |
| FR-03.3 | Empty Recycle Bin | Should | UC-01 |
| FR-03.4 | Install pending Windows updates | Must | UC-01 |
| FR-03.5 | Disable unnecessary services (Fax, MapsBroker, etc.) | Should | UC-01 |
| FR-03.6 | Remove orphaned registry keys | Should | UC-01 |
| FR-03.7 | Create system restore point before modifications | Should | UC-01 |

### FR-04: Post-Reboot Repair

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-04.1 | Execute sfc /scannow and capture exit code | Must | UC-03 |
| FR-04.2 | Execute DISM /Online /Cleanup-Image /RestoreHealth | Must | UC-03 |
| FR-04.3 | Execute chkdsk on all fixed drives | Must | UC-03 |
| FR-04.4 | Schedule second reboot if SFC requires it | Should | UC-03 |
| FR-04.5 | Detect if scheduled task was manually removed | Should | UC-03 |

### FR-05: Reporting

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-05.1 | Generate HTML report with styling | Must | UC-01, UC-04 |
| FR-05.2 | Generate plain-text report | Must | UC-01, UC-04 |
| FR-05.3 | Generate JSON report for machine parsing | Should | UC-06 |
| FR-05.4 | Generate PDF report (GUI mode, v3.1+) | Could | UC-08 |
| FR-05.5 | Include timestamps, machine ID, and app version in all reports | Must | UC-01 |

### FR-06: State Management

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-06.1 | Save state as JSON with mandatory schema_version | Must | UC-01 |
| FR-06.2 | Auto-migrate state from older schema versions on load | Must | UC-03 |
| FR-06.3 | Encrypt state with AES-256-GCM via Windows Credential Manager | Should | UC-01 |
| FR-06.4 | Maintain persistent machine_id separate from state | Must | UC-06 |
| FR-06.5 | Validate state integrity with CRC32 checksum | Should | UC-03 |

### FR-07: User Interface

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-07.1 | TUI with keyboard navigation (Arrow keys, Enter, Esc, Q) | Must | UC-01 |
| FR-07.2 | Real-time progress gauges during operations | Must | UC-01 |
| FR-07.3 | Scrollable log panel with color-coded severity | Must | UC-07 |
| FR-07.4 | Summary screen before maintenance with confirmation | Must | UC-01 |
| FR-07.5 | Reboot confirmation screen with countdown | Must | UC-01 |
| FR-07.6 | Report generation screen with file paths | Must | UC-01 |
| FR-07.7 | GUI mode via Iced (v3.1+) | Could | UC-08 |

### FR-08: Headless Operation

| ID | Requirement | Priority | UC |
|----|-------------|----------|-----|
| FR-08.1 | Accept --auto-phase <n> for non-interactive execution | Must | UC-06 |
| FR-08.2 | Accept --format=json for structured output | Must | UC-06 |
| FR-08.3 | Accept --what-if for simulation mode | Should | UC-05 |
| FR-08.4 | Return standardized exit codes | Must | UC-06 |
| FR-08.5 | Write JSON Lines logs to disk | Must | UC-06 |
| FR-08.6 | Suppress all TUI rendering in headless mode | Must | UC-06 |

---

## Non-Functional Requirements

### Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01.1 | Phase 1 (Audit) completes in < 60s on reference hardware | 60s |
| NFR-01.2 | Phase 3 (Cleanup) processes > 1GB/minute | 1GB/min |
| NFR-01.3 | TUI renders at >= 30 FPS during progress updates | 30 FPS |
| NFR-01.4 | Memory footprint < 128MB during normal operation | 128MB |
| NFR-01.5 | Binary size < 50MB (stripped release) | 50MB |

### Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-02.1 | 99.5% success rate in headless fleet deployments | 99.5% |
| NFR-02.2 | Graceful degradation when WMI unavailable | 100% fallback |
| NFR-02.3 | State recovery after power loss during any phase | 100% |
| NFR-02.4 | No false-positive AV detection after code signing | 0 |

### Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-03.1 | All update archives verified via Ed25519 signature | 100% |
| NFR-03.2 | State encryption keys never written to disk unencrypted | 100% |
| NFR-03.3 | No hardcoded secrets in source code | 100% |
| NFR-03.4 | Minimum TLS 1.3 for all network operations | TLS 1.3 |

### Compatibility

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-04.1 | Windows 10 1903+ and Windows 11 | Win 10+ |
| NFR-04.2 | PowerShell 5.1+ for spawned operations | PS 5.1+ |
| NFR-04.3 | Runs on x86_64; ARM64 planned for v3.2 | x86_64 |

### Maintainability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-05.1 | 100% trait coverage for Windows-specific code | 100% |
| NFR-05.2 | CI passes on Linux with mocks (unit tests) | Linux CI |
| NFR-05.3 | Weekly Windows integration tests | Weekly |
| NFR-05.4 | Documentation coverage > 80% public APIs | 80% |

---

## Traceability Matrix

| Requirement | UC-01 | UC-02 | UC-03 | UC-04 | UC-05 | UC-06 | UC-07 | UC-08 |
|-------------|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|
| FR-01.1 | Yes | Yes | | | | | | |
| FR-01.2 | Yes | | | | | | | |
| FR-01.3 | Yes | | Yes | | | | | |
| FR-01.4 | Yes | | Yes | | | | | |
| FR-01.5 | Yes | | Yes | | | | | |
| FR-02.1 | Yes | Yes | | | | | | |
| FR-02.7 | | Yes | | | | | | |
| FR-03.1 | Yes | | | | Yes | Yes | | |
| FR-04.1 | | | Yes | | | | | |
| FR-05.1 | Yes | | | Yes | Yes | | | |
| FR-06.1 | Yes | Yes | Yes | Yes | | Yes | | |
| FR-06.2 | | | Yes | Yes | | | | |
| FR-07.1 | Yes | Yes | | | | | Yes | |
| FR-08.1 | | | | | | Yes | | |
| FR-08.2 | | | | | | Yes | | |
| FR-08.4 | | | | | | Yes | | |

---

*Last updated: 2026-06-20 | Document version: 1.0*
