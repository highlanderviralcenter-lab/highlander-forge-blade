# Changelog

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
