# Reports

> Output formats, structure, and customization.

---

## Table of Contents

- [Report Types](#report-types)
- [HTML Report](#html-report)
- [Text Report](#text-report)
- [JSON Report](#json-report)
- [Report Directory Structure](#report-directory-structure)
- [Customizing Reports](#customizing-reports)

---

## Report Types

| Format | Extension | Best For | Interactive | Machine-Readable |
|--------|-----------|----------|:-----------:|:----------------:|
| HTML | .html | Technician review, email | Yes | No |
| Text | .txt | Quick CLI viewing, grep | No | Partial |
| JSON | .json | RMM integration, archiving | No | Yes |
| PDF | .pdf | End-user delivery (v3.1+) | Yes | No |

---

## HTML Report

### Structure

```html
<!DOCTYPE html>
<html>
<head>
  <title>HFB Report — MACHINE-01 — 2026-06-20</title>
  <style>/* Embedded CSS, no external dependencies */</style>
</head>
<body>
  <header>
    <h1>🗡️ Highlander Forge Blade Report</h1>
    <p>Machine: MACHINE-01 | Date: 2026-06-20 22:18:32 UTC</p>
  </header>

  <section id="executive-summary">
    <h2>Executive Summary</h2>
    <div class="metrics">
      <div class="metric success">✅ All phases completed</div>
      <div class="metric">💾 2.1 GB freed</div>
      <div class="metric">🔧 5 updates installed</div>
      <div class="metric">⚡ 2 services optimized</div>
    </div>
  </section>

  <section id="hardware">
    <h2>Hardware Inventory</h2>
    <table>...</table>
  </section>

  <section id="software">
    <h2>Software Inventory</h2>
    <table>...</table>
  </section>

  <section id="updates">
    <h2>Windows Updates</h2>
    <table>...</table>
  </section>

  <section id="services">
    <h2>Services</h2>
    <table>...</table>
  </section>

  <section id="logs">
    <h2>Operation Log</h2>
    <pre>...</pre>
  </section>
</body>
</html>
```

### Features

- Self-contained (no CDN dependencies, works offline)
- Responsive design (readable on mobile)
- Dark mode support (prefers-color-scheme)
- Collapsible sections
- Sortable tables (vanilla JS)

---

## Text Report

```
================================================================================
HIGHLANDER FORGE BLADE v3.0.0 — MAINTENANCE REPORT
================================================================================
Machine ID:    a1b2c3d4-e5f6-7890-abcd-ef1234567890
Machine Name:  MACHINE-01
Date:          2026-06-20 22:18:32 UTC
App Version:   3.0.0-alpha.1

================================================================================
EXECUTIVE SUMMARY
================================================================================
Status:        ✅ COMPLETED SUCCESSFULLY
Phases:        1, 2, 3, 4, 5
Duration:      12m 34s

================================================================================
HARDWARE
================================================================================
CPU:           Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz
  Cores:       8
  Threads:     8
  Max Speed:   4900 MHz

RAM:           16 GB (1x 16 GB DDR4-3200 Corsair)

Disks:
  C:  512 GB SSD (NVMe) — 45% used
  D:  1 TB HDD (SATA)  — 72% used

GPU:           NVIDIA GeForce RTX 2070 SUPER (8 GB)

================================================================================
CLEANUP RESULTS
================================================================================
Temporary Files:    1,247 files deleted (450 MB)
Browser Cache:        892 files deleted (380 MB)
  Chrome:           534 files (210 MB)
  Edge:             358 files (170 MB)
  Firefox:          Skipped (locked)

Windows Updates:    5 installed
  KB1234567, KB1234568, KB1234569, KB1234570, KB1234571

Services Optimized: 2 disabled
  Fax, MapsBroker

Registry:           3 orphaned keys removed

================================================================================
POST-REBOOT REPAIR
================================================================================
SFC:       ExitCode 0 (No integrity violations)
DISM:      ExitCode 0 (RestoreHealth completed)
CHKDSK C:  ExitCode 0 (No errors found)

================================================================================
END OF REPORT
================================================================================
```

---

## JSON Report

```json
{
  "version": "3.0.0-alpha.1",
  "machine_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "machine_name": "MACHINE-01",
  "timestamp": "2026-06-20T22:18:32Z",
  "phases_executed": ["1", "2", "3", "4", "5"],
  "duration_seconds": 754,
  "hardware": {
    "cpu": { "name": "Intel i7-9700K", "cores": 8, "threads": 8, "max_speed_mhz": 4900 },
    "memory": { "total_bytes": 17179869184, "modules": [{"capacity": 17179869184, "speed_mhz": 3200, "manufacturer": "Corsair"}] },
    "disks": [{"letter": "C:", "type": "SSD", "total_bytes": 549755813888, "used_bytes": 247390116096, "interface": "NVMe"}]
  },
  "software": { "count": 142, "third_party_count": 89 },
  "updates": { "installed": ["KB1234567", "KB1234568", "KB1234569", "KB1234570", "KB1234571"], "pending": 0 },
  "cleanup": {
    "bytes_freed": 2147483648,
    "temp_files_deleted": 1247,
    "browser_cache_deleted": 892,
    "services_disabled": ["Fax", "MapsBroker"],
    "registry_keys_removed": 3
  },
  "repair": {
    "sfc": { "exit_code": 0, "result": "No integrity violations" },
    "dism": { "exit_code": 0, "result": "RestoreHealth completed" },
    "chkdsk": [{ "drive": "C:", "exit_code": 0, "result": "No errors found" }]
  }
}
```

---

## Report Directory Structure

```
C:\ManutencaoWindows\
├── Relatorios\
│   └── Relatorio_20260620_221832\
│       ├── relatorio.html      # Human-readable, styled
│       ├── relatorio.txt       # Plain text, grep-friendly
│       └── relatorio.json      # Machine-parseable
```

---

## Customizing Reports

### Themes

Place custom CSS in:
```
C:\ManutencaoWindows\themes\custom.css
```

Reference in config.toml:
```toml
[report]
theme = "custom"
include_logs = true
max_log_entries = 500
```

### Localization

Reports support i18n via lang setting:
```toml
[report]
lang = "pt-BR"  # Portuguese (Brazil)
# lang = "en-US"  # English (default)
```

---

*Last updated: 2026-06-20 | Document version: 1.0*
