# Getting Started

> From zero to first maintenance in 5 minutes.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [First Run Walkthrough](#first-run-walkthrough)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| Windows | 10 1903 (build 18362) | 11 23H2 |
| PowerShell | 5.1 | 7.4+ |
| RAM | 4 GB | 8 GB |
| Disk Space | 100 MB (binary) + 500 MB (logs/state) | 2 GB |
| Internet | Optional | Required for updates & DISM |
| Permissions | Administrator | Administrator |

> ⚠️ **HFB requires Administrator privileges** for WMI, registry modifications, and service control.

---

## Installation

### Option A: Pre-built Binary (Recommended)

```powershell
# Download latest release
Invoke-WebRequest -Uri "https://github.com/highlanderviralcenter-lab/highlander-forge-blade/releases/latest/download/hfb-x86_64-pc-windows-msvc.exe" -OutFile "hfb.exe"

# Verify signature (requires signtool or Get-AuthenticodeSignature)
Get-AuthenticodeSignature -FilePath ".\hfb.exe"

# Move to permanent location
New-Item -ItemType Directory -Path "C:\ManutencaoWindows" -Force
Move-Item -Path ".\hfb.exe" -Destination "C:\ManutencaoWindows\hfb.exe"
```

### Option B: Build from Source

```bash
# Clone repository
git clone https://github.com/highlanderviralcenter-lab/highlander-forge-blade.git
cd highlander-forge-blade

# Install Rust (if not present)
# https://rustup.rs/

# Build release binary
cargo build --release --features tui

# Binary located at:
# target\release\hfb.exe
```

### Option C: Winget (Future)

```powershell
# Coming in v3.0.0 stable
winget install HighlanderForge.Blade
```

---

## Quick Start

### Interactive TUI (Default)

```powershell
# Launch interactive mode
C:\ManutencaoWindows\hfb.exe

# Or simply (if in PATH)
hfb
```

### Headless / Automated

```powershell
# Full maintenance, JSON output, no interaction
hfb --auto-phase 0 --format=json --output="C:\Temp\result.json"

# Audit only
hfb --auto-phase 1 --format=json

# Simulation (preview without changes)
hfb --what-if

# Check for updates
hfb --check-update
```

---

## First Run Walkthrough

### Step 1: Launch

```powershell
PS C:\> hfb
```

You will see the main menu:

```
╔══════════════════════════════════════════════════════════════╗
║  🗡️  HIGHLANDER FORGE BLADE v3.0.0-alpha.1                ║
║  Professional Windows Maintenance Engine                     ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  [▶] Run All Phases (1 to 5)                               ║
║  [ ] Phase 1: Audit Only                                   ║
║  [ ] Phase 3: Cleanup Only                                 ║
║  [ ] Phase 5: Post-Reboot Repair Only                      ║
║  [ ] Generate Report from Existing State                   ║
║  [ ] Settings                                              ║
║  [Q] Quit                                                  ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

↑↓ Navigate  •  Enter Select  •  Q Quit
```

### Step 2: Select "Run All Phases"

Use ↑ ↓ to highlight, Enter to select.

### Step 3: Phase 1 — Audit

The screen transitions to a progress view:

```
Phase 1: System Audit
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  100%

Hardware    ✅  CPU: Intel i7-9700K | RAM: 16 GB
Software    ✅  142 programs detected
Updates     ✅  5 pending
Services    ✅  187 total, 12 third-party
Registry    ✅  23 Run keys found
Environment ✅  47 variables collected

[Logs] 2026-06-20T22:15:32Z INFO  Audit completed in 34s
```

### Step 4: Review Summary

After audit, a summary screen displays findings:

```
┌─────────────────────────────────────────────────────────────┐
│  Summary: 5 Updates Pending, 2.1 GB Temp Files Found          │
├─────────────────────────────────────────────────────────────┤
│  Recommended Actions:                                         │
│  • Install 5 Windows updates                                  │
│  • Clean 2.1 GB temporary files                               │
│  • Disable services: Fax, MapsBroker                          │
│  • Remove 3 orphaned registry keys                            │
├─────────────────────────────────────────────────────────────┤
│  [Enter] Proceed with Maintenance  [Esc] Cancel               │
└─────────────────────────────────────────────────────────────┘
```

Press Enter to proceed or Esc to abort.

### Step 5: Phase 3 — Cleanup

Progress updates in real-time:

```
Phase 3: System Cleanup
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   67%

Operation: Cleaning browser cache (Chrome)
Bytes Freed: 1.4 GB / 2.1 GB

[Logs] 2026-06-20T22:16:45Z WARN  Firefox cache locked, skipped
```

### Step 6: Reboot Confirmation

```
┌─────────────────────────────────────────────────────────────┐
│  Reboot Required                                              │
├─────────────────────────────────────────────────────────────┤
│  Phase 5 (SFC/DISM/CHKDSK) requires reboot.                 │
│  Task 'HFB_PostReboot' will be created.                     │
│  Reboot in 60 seconds...                                    │
├─────────────────────────────────────────────────────────────┤
│  [Enter] Reboot Now  [S] Skip Reboot  [A] Abort             │
└─────────────────────────────────────────────────────────────┘
```

Press Enter to reboot immediately. The machine will restart and Phase 5 will run automatically.

### Step 7: Post-Reboot (Automatic)

After Windows restarts, HFB runs automatically (no login required):

```
[Task Scheduler] HFB_PostReboot triggered
Phase 5: Post-Reboot Repair
SFC     ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  100%  ExitCode: 0
DISM    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  100%  ExitCode: 0
CHKDSK  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  100%  C: ExitCode: 0

Maintenance Complete. Reports saved to:
C:\ManutencaoWindows\Relatorios\Relatorio_20260620_221832\
```

---

## Troubleshooting

### "Access Denied" during cleanup

Run as Administrator:
```powershell
Start-Process -FilePath "hfb.exe" -Verb RunAs
```

### WMI queries return empty

Corporate GPO may restrict WMI. HFB will automatically fall back to registry. Check logs:
```powershell
Get-Content "C:\ManutencaoWindows\Logs\hfb_*.jsonl" | ConvertFrom-Json | Where-Object { $_.level -eq "WARN" }
```

### Phase 5 didn't run after reboot

Check Task Scheduler:
```powershell
Get-ScheduledTask -TaskName "HFB_PostReboot*"
```

If missing, manually trigger:
```powershell
hfb --auto-phase 5
```

### Antivirus blocks HFB

Submit to your AV vendor as false positive. HFB is signed with code signing certificate (v3.0.0 stable+).

---

*Last updated: 2026-06-20 | Document version: 1.0*
