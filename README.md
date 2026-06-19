# 🗡️ Highlander Forge Blade

Manutencao profissional do Windows — engine Rust, UI TUI/GUI.

## Recursos

- **Fase 1**: Auditoria completa (hardware, software, servicos, registry)
- **Fase 2**: Resumo e confirmacao
- **Fase 3**: Limpeza e otimizacao
- **Fase 4**: Reinicializacao agendada
- **Fase 5**: Pos-reboot (SFC, DISM, CHKDSK)

## Modos de Execucao

```bash
# TUI interativo (padrao)
hfb

# Headless — automatizacao via RMM/GPO
hfb --auto-phase 0 --format=json

# Simulacao (sem alteracoes reais)
hfb --what-if

# Verificar atualizacoes
hfb --check-update
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
