# Operation Modes

> TUI, headless, simulation, and auto-phase reference.

---

## Table of Contents

- [Mode Matrix](#mode-matrix)
- [Interactive TUI](#interactive-tui)
- [Headless / Automated](#headless--automated)
- [Simulation Mode](#simulation-mode)
- [Auto-Phase Reference](#auto-phase-reference)
- [Exit Codes](#exit-codes)
- [Logging Formats](#logging-formats)

---

## Mode Matrix

| Mode | Flag | UI | Output | Interactive | Reboot |
|------|------|-----|--------|:---------:|:------:|
| TUI (default) | none | ratatui | HTML/TXT | Yes | Prompts |
| Headless | --auto-phase <n> | None | JSON | No | Auto if needed |
| Simulation | --what-if | ratatui | HTML/TXT | Yes | No |
| Report Only | --generate-report | ratatui | HTML/TXT/JSON | Yes | No |
| Update Check | --check-update | ratatui/None | Text/JSON | Optional | No |

---

## Interactive TUI

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ↑ ↓ | Navigate menu / scroll logs |
| Enter | Select / Confirm |
| Esc | Back / Cancel |
| Q | Quit (from menu) |
| Tab | Switch focus (log panel vs controls) |
| Space | Toggle checkbox |
| P | Pause/resume auto-scroll (logs) |
| S | Save screenshot of current view |

### Color Legend

| Color | Meaning |
|-------|---------|
| 🟢 Green | Success / Completed |
| 🔴 Red | Error / Failed |
| 🟡 Yellow | Warning / Skipped |
| 🔵 Blue | Info / In Progress |
| 🟣 Magenta | Phase transition |
| ⚪ White | Debug |

---

## Headless / Automated

### Command Reference

```bash
# Full maintenance, JSON output
hfb --auto-phase 0 --format=json

# Audit only, save to network share
hfb --auto-phase 1 --format=json --output="\\server\share\%COMPUTERNAME%.json"

# Cleanup only, human logs
hfb --auto-phase 3 --format=human --log-dir="C:\Logs"

# Post-reboot repair (called by Task Scheduler)
hfb --auto-phase 5

# Full maintenance with custom state path
hfb --auto-phase 0 --state-path="D:\HFB\state.json"
```

### GPO Deployment Example

```powershell
# Deploy via Group Policy Startup Script
# Save as: \domain\SYSVOL\scripts\deploy-hfb.ps1

$installDir = "C:\ManutencaoWindows"
$binary = "$installDir\hfb.exe"

if (-not (Test-Path $binary)) {
    Copy-Item "\\server\share\hfb.exe" $binary
}

& $binary --auto-phase 0 --format=json --output="\\server\share\reports\$env:COMPUTERNAME.json" --log-dir="$installDir\Logs"

# Exit code handling
$exitCode = $LASTEXITCODE
if ($exitCode -eq 5) {
    # Reboot pending — schedule via shutdown.exe
    shutdown /r /t 300 /c "HFB maintenance reboot in 5 minutes"
}
```

---

## Simulation Mode

```bash
hfb --what-if
```

### Behavior

- No filesystem modifications
- No registry writes
- No service changes
- No Windows Update installations
- Simulated report generated showing projected changes

### Log Prefix

All log lines prefixed with [SIMULATION]:

```json
{"timestamp":"2026-06-20T22:30:00Z","level":"INFO","target":"hfb::core::cleanup","message":"[SIMULATION] Would delete 1,247 temp files (450 MB)","phase":"3"}
```

### Exit Code

3 — SIMULATION_COMPLETE

---

## Auto-Phase Reference

| Phase | Flag | Description | Reboot Required |
|-------|------|-------------|:---------------:|
| 0 | --auto-phase 0 | Full cycle (1 to 2 to 3 to 4 to 5) | Yes |
| 1 | --auto-phase 1 | Audit only | No |
| 2 | --auto-phase 2 | Summary display (TUI only) | No |
| 3 | --auto-phase 3 | Cleanup + optimization | Maybe |
| 4 | --auto-phase 4 | Reboot scheduling | Yes |
| 5 | --auto-phase 5 | Post-reboot repair | Maybe |

---

## Exit Codes

```rust
pub mod exit_codes {
    pub const SUCCESS: i32              = 0;  // All phases completed
    pub const FATAL_ERROR: i32           = 1;  // Critical error, check logs
    pub const SUCCESS_WITH_WARNINGS: i32 = 2;  // Done, but with warnings
    pub const SIMULATION_COMPLETE: i32   = 3;  // --what-if finished
    pub const UPDATE_AVAILABLE: i32      = 4;  // --check-update found new version
    pub const NEEDS_REBOOT: i32          = 5;  // Phase 3 done, reboot pending
    pub const PARTIAL_SUCCESS: i32       = 6;  // Some phases OK, others failed
}
```

### RMM Integration Example

```python
# Nagios / Zabbix / custom RMM check
import subprocess, json, sys

result = subprocess.run(["hfb", "--auto-phase", "0", "--format=json"], 
                       capture_output=True, text=True)

# Parse last line (HeadlessOutput JSON)
output = json.loads(result.stdout.strip().split('\n')[-1])

if result.returncode == 0:
    print(f"OK: Maintenance complete | bytes_freed={output['summary']['bytes_freed']}")
    sys.exit(0)
elif result.returncode == 5:
    print("WARNING: Reboot pending")
    sys.exit(1)
else:
    print(f"CRITICAL: Exit code {result.returncode} | {output['exit_reason']}")
    sys.exit(2)
```

---

## Logging Formats

### Human Format (TUI default)

```
2026-06-20 22:15:32 [INFO]  Starting Phase 1: System Audit
2026-06-20 22:15:33 [INFO]  Collecting hardware information...
2026-06-20 22:15:34 [SUCCESS] CPU: Intel i7-9700K (8 cores, 8 threads)
2026-06-20 22:15:35 [WARN]  WMI query Win32_TemperatureProbe returned empty, using fallback
```

### JSON Format (Headless default)

```json
{"timestamp":"2026-06-20T22:15:32.123Z","level":"INFO","target":"hfb::core::audit","fields":{"message":"Starting Phase 1: System Audit","phase":"1"},"span":{"name":"audit"}}
{"timestamp":"2026-06-20T22:15:34.456Z","level":"SUCCESS","target":"hfb::core::audit","fields":{"message":"CPU collected","cpu":"Intel i7-9700K","cores":8,"threads":8}}
```

### JSON Lines File

One JSON object per line, parseable with jq:

```bash
# Filter errors only
jq 'select(.level == "ERROR")' C:\ManutencaoWindows\Logs\hfb_20260620_221500.jsonl

# Count warnings by phase
jq -s 'group_by(.fields.phase) | map({phase: .[0].fields.phase, count: length})' logs.jsonl
```

---

*Last updated: 2026-06-20 | Document version: 1.0*
