# 📊 HFB — Status Real do Projeto (2026-10-01)

> Este documento é a **fonte da verdade** sobre o que funciona.
> Diferente de `docs/README.md` (documento de visão/design), aqui não há otimismo de marketing.

## Legenda
- ✅ **Funcional** — código real, testável agora
- 🟡 **Parcial** — implementado mas incompleto ou não conectado
- ❌ **Stub/Fake** — retorna dados simulados ou vazio
- 🔗 **Órfão** — existe no código, mas nada o chama

---

## Por componente

| Componente | Arquivo | Status | Observação |
|---|---|---|---|
| TUI (menu, navegação, render) | `ui/ratatui/*` | ✅ | Loop ratatui+crossterm funcional; teclas ↑↓/Enter/Q |
| Fase 1 — Auditoria (TUI) | `ui/ratatui/app.rs::run_audit` | ✅ | PowerShell CIM real: serviços, updates pendentes, disco |
| Fase 2 — Resumo/Confirmação | `app/messages.rs`, UI | ✅ | Reducer trata AuditCompleted/CleanupConfirmed etc. |
| Motor `core::Auditor` + traits DI | `core/audit.rs`, `core/traits.rs` | 🟡🔗 | Robusto e com testes, mas a UI **não o usa** (lógica duplicada inline) |
| Provider WMI | `platform/windows/wmi.rs` | 🟡🔗 | Funciona, porém usa `Get-WmiObject` (deprecated) e múltiplos spawns |
| Provider Serviços | `platform/windows/services.rs` | 🟡🔗 | Parser de `sc query` frágil (locale-dependente) |
| Provider Registry | `platform/windows/registry.rs` | ❌ | Stub |
| Provider Windows Update (COM) | `platform/windows/updates.rs` | ❌ | Stub |
| Factory cross-platform | `core/traits.rs` / `platform/factory.rs` | ✅ | Compila em Linux e Windows (windows crate isolada por target) |
| Config TOML/JSON | `config.rs` | ✅ | Cascata: env `HFB_BASE_DIR` → arquivo → `%PROGRAMDATA%\HighlanderForgeBlade`; template gerável |
| Estado versionado + CRC32 | `app/state.rs` | ✅ | Save/load/migração v0→v1 com checksum corrigido; testes incluídos |
| machine_id estável | `app/machine_id.rs` | ✅ | UUID SMBIOS (UUIDv5) com fallback; persistido em `id.dat` oculto |
| Logging rotativo | `logging.rs` | ✅ | TUI → arquivo diário (`Logs/hfb.log.*`); headless → stdout JSON limpo |
| Manifest UAC (`requireAdministrator`) | `assets/*.manifest`, `build.rs` | ✅ | Embutido via winres em builds Windows |
| Headless engine | `app/headless.rs` | 🟡 | Execução REAL das fases 1/3/5 via ProviderFactory, exit codes honestos — **mas sem dispatcher CLI para invocá-lo** |
| CLI (`clap`) | `main.rs` | ❌ | `clap` está nas dependências, **nenhum parsing implementado**. Flags documentadas inexistem |
| Checagem de elevação em runtime | `main.rs` | ❌ | Depende só do manifest; sem flag `--no-elevate-check` |
| Fase 3 — Limpeza (TUI) | `ui/ratatui/app.rs::run_cleanup` | ❌ | **FAKE**: soma 100 MB/opção com `sleep(500ms)`; nenhuma exclusão real |
| Limpeza real (core) | `core/cleanup.rs` | 🟡🔗 | Implementações reais (temp/DISM/SFC) com bugs conhecidos; não conectada à UI |
| Fase 4 — Reboot agendado | — | ❌ | Não implementada |
| Fase 5 — Pós-reboot SFC/DISM/CHKDSK | — | ❌ | Não implementada (logs nunca parseados) |
| Relatórios HTML/TXT/JSON em disco | `core/reports*` | ❌ | Nunca gerados |
| Auto-update Ed25519 | — | ❌ | Chave embutida prevista, verificação não implementada |
| Testes unitários | `src/**` | ✅ | config(3), state(3), machine_id(1), audit(…) — todos passando |
| Testes de integração/benches | `tests/`, `benches/` | ❌ | Placeholders `assert!(true)` |
| Scripts release | `scripts/build-release.ps1` | ❌ | Sintaxe truncada/inválida |

## Resumo numérico
- **Fases prometidas:** 5 · **Fases realmente funcionais:** 2 (via TUI)
- **Correções aplicadas na auditoria:** itens P0 concluídos (caminhos, checksum, deps cross-platform, manifest, logging, panic=abort, supressões, higiene do repo)
- **Versão atual:** `3.0.0-alpha.2`

## Como testar hoje (Windows, como Administrador)
```powershell
cd C:\highlander-forge-blade
cargo build --release          # gera target\release\hfb.exe (com manifest UAC)
cargo run                      # abre a TUI → menu 1 = auditoria real, menu 2 = resumo
dir $env:ProgramData\HighlanderForgeBlade   # State\state.json, Logs\, Reports\
```
⚠️ O menu "Limpeza" mostra progresso **simulado** — nenhuma exclusão ocorre ainda.

## Próximo lote de trabalho (ordem planejada)
1. CLI `clap` em `main.rs` (--auto-phase, --what-if, --format, --config, --check-update) → conecta ao headless já pronto
2. Unificar auditoria da UI em `core::Auditor` (eliminar duplicação)
3. Substituir `run_cleanup` fake pelo provider real (medição pós-exclusão, dry-run, multi-volume)
4. Fases 4/5 reais (shutdown + RunOnce pós-reboot + parsing SFC/DISM/CHKDSK)
5. Relatórios físicos + scripts de release corrigidos
