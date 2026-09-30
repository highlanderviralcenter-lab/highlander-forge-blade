//! Logging dual-mode: humano (TUI/desenvolvimento) e JSON (headless/RMM).
//!
//! Correcoes vs alpha.1:
//! - Em TUI, saida humana vai para ARQUIVO (stdout poluiria a tela do ratatui).
//! - Log rotativo diario em <base_dir>/Logs via tracing-appender.
//! - Nivel controlado por Config.log_level e/ou RUST_LOG (RUST_LOG tem prioridade).

use std::path::Path;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, Registry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Human,
    Json,
}

/// Estado do appender de arquivo — mantido vivo enquanto o programa roda.
pub struct LoggingGuard {
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

fn build_filter(level: &str) -> EnvFilter {
    // RUST_LOG vence; senao usa nivel da Config com default conservador.
    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let lvl = level.parse().unwrap_or(tracing::level_filters::LevelFilter::INFO);
        EnvFilter::default().add_directive(lvl.into())
    })
}

/// Inicializa logging com arquivo rotativo no diretorio indicado.
pub fn init_with_dir(format: LogFormat, level: &str, logs_dir: Option<&Path>) -> LoggingGuard {
    let filter = build_filter(level);
    let timer = fmt::time::ChronoLocal::rfc_3339();

    // Camada de arquivo (rotacao diaria) quando o diretorio esta disponivel.
    let mut guard = None;
    let file_layer: Option<Box<dyn Layer<Registry> + Send + Sync>> = match logs_dir {
        Some(dir) => {
            match std::fs::create_dir_all(dir).map(|_| tracing_appender::rolling::daily(dir, "hfb.log")) {
                Ok(file_appender) => {
                    let (nb, g) = tracing_appender::non_blocking(file_appender);
                    guard = Some(g);
                    let layer = match format {
                        LogFormat::Json => fmt::layer()
                            .json()
                            .with_writer(nb)
                            .with_timer(timer)
                            .with_target(true)
                            .with_level(true)
                            .boxed(),
                        LogFormat::Human => fmt::layer()
                            .with_writer(nb)
                            .with_timer(timer)
                            .with_target(false)
                            .with_level(true)
                            .with_ansi(false)
                            .boxed(),
                    };
                    Some(layer.boxed())
                }
                Err(e) => {
                    eprintln!("aviso: nao foi possivel criar log em {:?}: {}", dir, e);
                    None
                }
            }
        }
        None => None,
    };

    // Camada de console: JSON sempre (stdout parseavel p/ RMM);
    // humana apenas fora da TUI (na TUI stdout eh o terminal do ratatui).
    let console_layer: Option<Box<dyn Layer<Registry> + Send + Sync>> = match format {
        LogFormat::Json => Some(
            fmt::layer()
                .json()
                .with_writer(std::io::stdout)
                .with_timer(fmt::time::ChronoLocal::rfc_3339())
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_current_span(true)
                .boxed(),
        ),
        LogFormat::Human if logs_dir.is_none() => Some(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_timer(fmt::time::ChronoLocal::rfc_3339())
                .with_target(false)
                .with_level(true)
                .with_ansi(true)
                .boxed(),
        ),
        LogFormat::Human => None, // TUI: somente arquivo
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    LoggingGuard { _guard: guard }
}

/// API legada (sem arquivo) — mantem compatibilidade de chamadas antigas.
pub fn init_logging(format: LogFormat) {
    init_with_dir(format, "info", None);
}
