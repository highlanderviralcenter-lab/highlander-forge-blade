# 🗡️ Highlander Forge Blade

Manutencao profissional do Windows — engine Rust, UI TUI/GUI.

> ⚠️ **Status (v3.0.0-alpha.2):** apenas as **Fases 1 e 2** estao funcionando de verdade
> (via TUI). As Fases 3–5 ainda estao em desenvolvimento — a limpeza exibida na TUI e
> simulada. Consulte [`docs/STATUS.md`](docs/STATUS.md) para a matriz completa do que
> funciona, o que e stub e o que esta pendente. Nao confie nesta secao de recursos ate
> que o changelog confirme cada item.

## Recursos

- **Fase 1**: ✅ Auditoria real (servicos, updates pendentes, disco) via PowerShell CIM
- **Fase 2**: ✅ Resumo e confirmacao na TUI
- **Fase 3**: 🚧 Em desenvolvimento (atualmente simulada na TUI)
- **Fase 4**: 🚧 Nao implementada (reinicializacao agendada)
- **Fase 5**: 🚧 Nao implementada (pos-reboot SFC/DISM/CHKDSK)
- ✅ Estado persistente versionado com checksum, config TOML/JSON, logs rotativos,
  manifest UAC (`requireAdministrator`) embutido no exe

## Modos de Execucao

```bash
# TUI interativo (unico modo disponivel hoje)
hfb            # ou: cargo run  (execute como Administrador)

# --- As flags abaixo AINDA NAO EXISTEM (planejadas para alpha.3) ---
# hfb --auto-phase 0 --format=json
# hfb --what-if
# hfb --check-update
```

## Compilacao

```bash
# TUI
cargo build --release --features tui

# GUI (futuro)
cargo build --release --features gui
```

## Arquitetura

- `app/`: Camada de aplicacao (estado, mensagens, comandos)
- `core/`: Engine de manutencao (traits para testabilidade)
- `ui/`: Interfaces (ratatui TUI / iced GUI)
- `platform/`: Codigo Windows-specific

## Licenca

MIT ou proprietaria — ver LICENSE.
