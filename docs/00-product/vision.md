# 🗡️ Highlander Forge Blade

> **Professional Windows Maintenance Engine — Rust-powered, TUI-first, Enterprise-ready**

[![Build Status](https://img.shields.io/github/actions/workflow/status/highlanderviralcenter-lab/highlander-forge-blade/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/highlanderviralcenter-lab/highlander-forge-blade/actions)
[![Crates.io](https://img.shields.io/badge/crates.io-v3.0.0--alpha.1-orange?style=flat-square&logo=rust)](https://crates.io/crates/highlander-forge-blade)
[![License](https://img.shields.io/badge/license-MIT%2FProprietary-blue?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.78+-purple?style=flat-square&logo=rust)](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0.html)
[![Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?style=flat-square&logo=windows)](https://www.microsoft.com/windows)

---

## Table of Contents

- [Vision](#vision)
- [Core Principles](#core-principles)
- [Target Audiences](#target-audiences)
- [Competitive Landscape](#competitive-landscape)
- [Success Metrics](#success-metrics)

---

## Vision

**Highlander Forge Blade (HFB)** is a next-generation Windows maintenance and optimization platform built in Rust. It bridges the gap between ad-hoc PowerShell scripts and bloated, closed-source "PC optimizers" by providing:

- **A deterministic, auditable maintenance pipeline** — 5 phases from audit to post-reboot repair
- **Zero-cost abstractions** — bare-metal performance via Rust's ownership model
- **Dual-mode operation** — interactive TUI for technicians, headless JSON for RMM/MSP integration
- **Cryptographic integrity** — Ed25519-signed auto-updates, AES-256-GCM encrypted state
- **Fleet visibility** — SaaS dashboard for multi-machine management (v4.0+)

> *"There can be only one tool on the technician's USB stick."*

---

## Core Principles

| Principle | Implementation |
|-----------|---------------|
| **Determinism** | Every phase is idempotent, state-versioned, and resumable |
| **Transparency** | All operations logged in structured JSON; `--what-if` simulation mode |
| **Resilience** | Fails gracefully: corrupted state -> retry; missing WMI -> registry fallback |
| **Security** | No secrets in code; Credential Manager for keys; compile-time embedded public keys |
| **Testability** | Traits abstract all Windows-specific code; mocks run on Linux CI |
| **Ergonomics** | TUI with real-time progress, keyboard navigation, and color-coded severity |

---

## Target Audiences

```mermaid
mindmap
  root((HFB Users))
    Technician
      Interactive TUI
      One-click maintenance
      HTML reports
    MSP / Admin
      Headless deployment
      GPO / RMM integration
      Structured JSON output
    Enterprise
      Fleet dashboard
      Policy enforcement
      Audit compliance
    End User
      GUI wizard
      Toast notifications
      PDF reports
```

---

## Competitive Landscape

| Tool | Open Source | Structured Output | Resume After Reboot | Crypto-Signed Updates | Rust |
|------|:-----------:|:-----------------:|:-------------------:|:---------------------:|:----:|
| CCleaner | No | No | No | No | No |
| BleachBit | Yes | No | No | No | No |
| TronScript | Yes | No | Partial | No | No |
| **HFB v3.0** | Yes | Yes | Yes | Yes | Yes |

---

## Success Metrics

- **Performance**: Full audit (Phase 1) completes in < 60s on i5-8400 / 8GB RAM
- **Reliability**: 99.5% success rate across 1000+ headless deployments
- **Security**: Zero false-positive AV detections after code signing
- **Adoption**: 100+ stars, 10+ contributors by v3.0 stable

---

*Last updated: 2026-06-20 | Document version: 1.0*
