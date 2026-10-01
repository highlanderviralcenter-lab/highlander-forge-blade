# Changelog

## [3.0.0-alpha.2] - 2026-10-01

### Corrigido (auditoria profissional — itens P0)
- **Caminhos base**: `config.rs` reescrito; removido o caminho quebrado `r"C:\\ManutencaoWindows"`; resolucao em cascata (`HFB_BASE_DIR` -> config TOML/JSON -> `%PROGRAMDATA%\HighlanderForgeBlade`) com `ensure_dirs()`
- **Build cross-platform**: crate `windows` movida para `[target.'cfg(windows)'.dependencies]`; factory de providers compila em Linux/CI
- **Checksum do estado**: migracao v0->v1 agora recalcula o CRC32; falha de serializacao nao gera mais checksum valido para dados vazios; `StateLoaded(Ok)` restaura audit_data/fases
- **machine_id estavel**: derivado de UUID SMBIOS (UUIDv5, sobrevive a reinstalacao), fallback aleatorio v4 persistido; erro de `SetFileAttributesW` logado
- **Logging compativel com TUI**: logs vao para arquivo rotativo diario (`Logs/hfb.log.*`, tracing-appender); stdout reservado so para saida JSON headless
- **Reducer**: mensagens antes ignoradas implementadas (`CleanupFailed`, `RebootScheduled/Cancelled`, `PostRebootProgress/Failed`, `Shutdown`); navegacao limitada por `MENU_ITEM_COUNT` (sem indices magicos)
- **Release profile**: removido `panic = "abort"` (prejudicava diagnostico e testes)
- **Supressoes globais** `allow(dead_code)/allow(unused_imports)` removidas de `lib.rs`
- **Headless engine real**: fases 1/3/5 executam via `ProviderFactory` com exit codes honestos (NEEDS_REBOOT / PARTIAL_SUCCESS / SUCCESS_WITH_WARNINGS); `--what-if` read-only de verdade
- **Scripts**: `build-release.ps1` corrigido
- **Higiene do repo**: removidos backups/lixo (`alpha2_fix/`, `debug.txt`, `messages_backup.txt`, `hfb_mapeamento.ps1`); `target/` fora do versionamento

### Adicionado
- Manifest UAC `requireAdministrator` (`assets/hfb.exe.manifest` + `assets/hfb.rc`) embutido automaticamente via `build.rs` (winres)
- CLI com `clap` no `main.rs`: `--auto-phase`, `--what-if`, `--format`, `--config`, `--check-update`, `--version` (dispatcher conecta ao headless engine)
- Checagem de elevacao em runtime (`EXIT_NOT_ELEVATED`) alem do manifest
- Dependencias: `clap`, `dirs`, `tracing-appender`, `winreg` (target Windows)
- Testes unitarios reais: config (3), state roundtrip/tamper/migracao (3), machine_id (1)
- `docs/STATUS.md`: matriz honesta do que funciona / stub / pendente

### Conhecido (nao entregue ainda)
- Limpeza da TUI (`run_cleanup`) permanece **simulada** ate conexao com `core::cleanup`
- Fases 4/5 na TUI, relatorios fisicos, providers Registry/WU e auto-update Ed25519 seguem pendentes

## [3.0.0-alpha.1] - 2026-06-18

### Adicionado
- Esqueleto do projeto com arquitetura em camadas
- Menu TUI com ratatui 0.29
- Canal mpsc entre Tokio e loop de renderizacao (DT-01)
- Traits para injecao de dependencia (DT-09)
- State versionado com schema_version (DT-02)
- machine_id persistente separado do estado (DT-10)
- Logging dual-mode: humano e JSON (DT-13)
- Auto-update com verificacao Ed25519 (DT-11)
- Modo headless com exit codes padronizados (DT-12)

### Seguranca
- Chave publica de update embutida em compile-time
- Criptografia de estado via Credential Manager
