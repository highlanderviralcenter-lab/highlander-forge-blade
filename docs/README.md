# 🗡️ Highlander Forge Blade

> **Professional Windows Maintenance Engine — Rust-powered, TUI-first, Enterprise-ready**

[![Build Status](https://img.shields.io/github/actions/workflow/status/highlanderviralcenter-lab/highlander-forge-blade/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/highlanderviralcenter-lab/highlander-forge-blade/actions)
[![Crates.io](https://img.shields.io/badge/crates.io-v3.0.0--alpha.2-orange?style=flat-square&logo=rust)](https://crates.io/crates/highlander-forge-blade)
[![License](https://img.shields.io/badge/license-MIT%2FProprietary-blue?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.78+-purple?style=flat-square&logo=rust)](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0.html)
[![Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?style=flat-square&logo=windows)](https://www.microsoft.com/windows)

---

## What is HFB?

**Highlander Forge Blade** is a next-generation Windows maintenance and optimization platform built in Rust. It bridges the gap between ad-hoc PowerShell scripts and bloated, closed-source "PC optimizers" by providing:

- 🎯 **A deterministic, auditable maintenance pipeline** — 5 phases from audit to post-reboot repair
- ⚡ **Zero-cost abstractions** — bare-metal performance via Rust's ownership model
- 🖥️ **Dual-mode operation** — interactive TUI for technicians, headless JSON for RMM/MSP integration
- 🔐 **Cryptographic integrity** — Ed25519-signed auto-updates, AES-256-GCM encrypted state
- 📊 **Fleet visibility** — SaaS dashboard for multi-machine management (v4.0+)

> *"There can be only one tool on the technician's USB stick."*

---

## Features

| Feature | Status | Version |
|---------|--------|---------|
| Interactive TUI with real-time progress (menu, navigation) | ✅ Stable | v3.0.0-alpha.2 |
| Phase 1 — Audit via PowerShell CIM (services, updates, disk) | ✅ Real data | v3.0.0-alpha.2 |
| Phase 2 — Summary & confirmation | ✅ Stable | v3.0.0-alpha.2 |
| Config TOML/JSON + `%PROGRAMDATA%` resolution | ✅ Stable | v3.0.0-alpha.2 |
| State persistence with checksum + migration | ✅ Stable | v3.0.0-alpha.2 |
| Rotating file logging (TUI-safe) | ✅ Stable | v3.0.0-alpha.2 |
| UAC manifest (`requireAdministrator`) embedded in exe | ✅ Stable | v3.0.0-alpha.2 |
| Cross-platform build (providers behind `ProviderFactory`) | ✅ Stable | v3.0.0-alpha.2 |
| Headless engine (real phases 1/3/5, honest exit codes) | 🟡 Engine ready, CLI dispatcher pending | v3.0.x |
| Phase 3 — Cleanup | ❌ **Simulated in TUI** (no real deletions yet) | v3.0.x |
| Phase 4 — Scheduled reboot | ❌ Not implemented | v3.0.x |
| Phase 5 — Post-reboot SFC/DISM/CHKDSK | ❌ Not implemented | v3.0.x |
| Reports on disk (HTML/TXT/JSON) | ❌ Not implemented | v3.0.x |
| Auto-update with Ed25519 signature verification | ❌ Design only — not implemented | v3.1.0 |
| GUI with Iced (wizard + dashboard) | 🚧 In Progress | v3.1.0 |
| Blake3 file indexing + deduplication | 📅 Planned | v3.2.0 |
| SaaS fleet dashboard | 📅 Planned | v4.0.0 |

> ⚠️ **Honesty note:** previous revisions of this document marked the 5-phase cycle,
> headless JSON mode and auto-update as "✅ Stable". That was aspirational, not factual.
> See [`STATUS.md`](STATUS.md) for the full component-by-component matrix.

---

## Quick Start

### Installation

```powershell
# Download latest release
Invoke-WebRequest -Uri "https://github.com/highlanderviralcenter-lab/highlander-forge-blade/releases/latest/download/hfb-x86_64-pc-windows-msvc.exe" -OutFile "hfb.exe"

# Verify signature
Get-AuthenticodeSignature -FilePath ".\hfb.exe"

# Run
.\hfb.exe
```

### Interactive Mode (TUI) — ✅ disponível hoje

```powershell
# Build a partir do fonte (Windows 10/11, execute como Administrador)
cargo build --release
.\target\release\hfb.exe        # abre o menu TUI (Fase 1 e 2 reais)
```

### Headless / CLI — 🚧 planejado para alpha.3

As flags abaixo **ainda não existem** no binário atual (o motor headless está pronto
em `src/app/headless.rs`; falta apenas o dispatcher `clap` em `main.rs`):

```powershell
# hfb --auto-phase 1                        # Audit only
# hfb --auto-phase 0 --format=json          # Full maintenance, JSON output
# hfb --what-if                             # Simulation (preview sem alteracoes)
# hfb --check-update                        # Verificar atualizacao (Ed25519)
```

---

## Architecture

```mermaid
graph TB
    subgraph "Presentation"
        TUI[ratatui TUI]
        CLI[Headless CLI]
        GUI[iced GUI v3.1+]
    end

    subgraph "Application"
        APP[AppState]
        MSG[AppMsg Channel]
        CMD[Commands]
    end

    subgraph "Domain"
        AUDIT[Audit]
        CLEAN[Cleanup]
        REPAIR[Repair]
        STATE[State Management]
    end

    subgraph "Platform"
        WMI[WMI]
        REG[Registry]
        SVC[Services]
        TS[Task Scheduler]
    end

    TUI --> APP
    CLI --> APP
    GUI --> APP
    APP --> MSG
    MSG --> AUDIT
    MSG --> CLEAN
    MSG --> REPAIR
    AUDIT --> WMI
    CLEAN --> SVC
    REPAIR --> TS
    STATE --> REG
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [00-product/vision.md](00-product/vision.md) | Product vision and principles |
| [00-product/use-cases.md](00-product/use-cases.md) | Complete use case catalog |
| [00-product/requirements.md](00-product/requirements.md) | Functional & non-functional requirements |
| [00-product/roadmap.md](00-product/roadmap.md) | Milestone timeline |
| [01-user/getting-started.md](01-user/getting-started.md) | First-time user guide |
| [01-user/operation-modes.md](01-user/operation-modes.md) | TUI, headless, simulation reference |
| [01-user/reports.md](01-user/reports.md) | Output formats and customization |
| [02-engineering/workflow.md](02-engineering/workflow.md) | Development workflow |
| [02-engineering/state-machine.md](02-engineering/state-machine.md) | State persistence and recovery |
| [02-engineering/function-points.md](02-engineering/function-points.md) | Sizing analysis |
| [02-engineering/risks.md](02-engineering/risks.md) | Risk register |
| [03-architecture/architecture.md](03-architecture/architecture.md) | System architecture |
| [03-architecture/runtime.md](03-architecture/runtime.md) | Async runtime and MPSC channel |
| [03-architecture/ui.md](03-architecture/ui.md) | UI layer design |
| [03-architecture/platform.md](03-architecture/platform.md) | Windows platform abstractions |
| [04-security/security.md](04-security/security.md) | Security architecture |
| [04-security/updates.md](04-security/updates.md) | Auto-update system |
| [04-security/credential-manager.md](04-security/credential-manager.md) | Key storage |
| [05-development/versioning.md](05-development/versioning.md) | Versioning & releases |
| [05-development/branching.md](05-development/branching.md) | Git branching strategy |
| [05-development/contributing.md](05-development/contributing.md) | Contribution guide |
| [06-future/indexing.md](06-future/indexing.md) | Blake3 file indexing |
| [06-future/recovery.md](06-future/recovery.md) | File recovery & secure wipe |
| [06-future/api.md](06-future/api.md) | SaaS API & portal |
| [07-decisions/](07-decisions/) | Architecture Decision Records (ADRs) |

---

## Safety & Security

- 📝 **Structured audit logs** — arquivo rotativo diario (`Logs/hfb.log.*`), timestamped
- 🔐 **Integridade de estado** — checksum CRC32 no `state.json` com deteccao de tamper e migracao versionada
- 🛡️ **Elevacao explicita** — manifest `requireAdministrator` embutido; o exe nao roda sem UAC
- 🧪 **Testes unitarios reais** para config/estado/machine-id (CI em Linux possivel apos factory cross-platform)
- ⚠️ *Planejado, ainda nao implementado:* AES-256-GCM via Credential Manager e verificacao Ed25519 de updates

---

## Performance

| Metric | Target | Measured |
|--------|--------|----------|
| Full audit (Phase 1) | < 60s | — |
| Cleanup (Phase 3) | > 1 GB/min | — |
| TUI render | 60 FPS | — |
| Binary size (stripped) | < 50 MB | — |
| Memory footprint | < 128 MB | — |

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](05-development/contributing.md) for details.

- 🐛 [Report bugs](https://github.com/highlanderviralcenter-lab/highlander-forge-blade/issues)
- 💡 [Request features](https://github.com/highlanderviralcenter-lab/highlander-forge-blade/issues)
- 🔧 [Submit PRs](https://github.com/highlanderviralcenter-lab/highlander-forge-blade/pulls)

---

## License

This project is dual-licensed under:

- **MIT License** — for open source use
- **Proprietary License** — for commercial/enterprise use

See [LICENSE](LICENSE) for details.

---

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) — Systems programming with safety
- [Tokio](https://tokio.rs/) — Async runtime
- [ratatui](https://ratatui.rs/) — Terminal UI framework
- [iced](https://iced.rs/) — GUI framework (v3.1+)
- [Axum](https://github.com/tokio-rs/axum) — Web framework (v4.0+)

---

<p align="center">
  <strong>🗡️ Highlander Forge Blade</strong><br>
  <em>Professional Windows Maintenance</em><br>
  <a href="https://github.com/highlanderviralcenter-lab/highlander-forge-blade">GitHub</a> •
  <a href="https://github.com/highlanderviralcenter-lab/highlander-forge-blade/tree/main/docs">Documentation</a> •
  <a href="">Discord</a>
</p>
