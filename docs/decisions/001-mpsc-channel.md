# ADR 001: Canal MPSC entre Tokio e ratatui

## Status
Aceito

## Contexto
ratatui e sincrono. O loop terminal.draw() bloqueia. Tasks async do core (WMI, disco, Windows Update) nao podem bloquear a UI.

## Decisao
Usar tokio::sync::mpsc::channel<AppMsg>(256) — bounded para backpressure controlado.

## Consequencias
- Positivo: UI responsiva mesmo durante operacoes longas
- Positivo: Arquitetura limpa — UI consome, core produz
- Negativo: Overhead de serializacao das mensagens
