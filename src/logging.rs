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
    EnvFilter,
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

// Correcao E0277/E0599 (build): as camadas de file/console precisam ser tipos
// CONCRETOS combinaveis via .with(). Box<dyn Layer<Registry>> nao implementa
// Layer<Layered<...>>, e genericidade solta vaza no impl Trait. Solucao:
// cada variacao constroi seu proprio subscriber concreto abaixo.

/// Inicializa logging com arquivo rotativo no diretorio indicado.
pub fn init_with_dir(format: LogFormat, level: &str, logs_dir: Option<&Path>) -> LoggingGuard {
    let filter = build_filter(level);
    let timer = fmt::time::ChronoLocal::rfc_3339();

    // Monta o appender de arquivo (rotacao diaria) se possivel.
    let file_appender = match logs_dir {
        Some(dir) => match std::fs::create_dir_all(dir)
            .map(|_| tracing_appender::rolling::daily(dir, "hfb.log"))
        {
            Ok(a) => Some(a),
            Err(e) => {
                eprintln!("aviso: nao foi possivel criar log em {:?}: {}", dir, e);
                None
            }
        },
        None => None,
    };

    let (nb, guard) = match file_appender {
        Some(a) => {
            let (nb, g) = tracing_appender::non_blocking(a);
            (Some(nb), Some(g))
        }
        None => (None, None),
    };

    // Cada combinacao constrói um subscriber de tipo concreto (camadas homogeneas).
    match (format, nb) {
        (LogFormat::Json, Some(nb)) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .json()
                        .with_writer(std::io::stdout)
                        .with_timer(timer.clone())
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_level(true)
                        .with_current_span(true),
                )
                .with(
                    fmt::layer()
                        .json()
                        .with_writer(nb)
                        .with_timer(timer)
                        .with_target(true)
                        .with_level(true),
                )
                .init();
        }
        (LogFormat::Json, None) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .json()
                        .with_writer(std::io::stdout)
                        .with_timer(timer)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_level(true)
                        .with_current_span(true),
                )
                .init();
        }
        (LogFormat::Human, Some(nb)) => {
            // TUI: saida humana apenas em arquivo (stdout eh o terminal do ratatui).
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_writer(nb)
                        .with_timer(timer)
                        .with_target(false)
                        .with_level(true)
                        .with_ansi(false),
                )
                .init();
        }
        (LogFormat::Human, None) => {
            // Sem dir de log: fallback para stderr (nao polui stdout).
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_writer(std::io::stderr)
                        .with_timer(timer)
                        .with_target(false)
                        .with_level(true)
                        .with_ansi(true),
                )
                .init();
        }
    }

    LoggingGuard { _guard: guard }
}

/// API legada (sem arquivo) — mantem compatibilidade de chamadas antigas.
pub fn init_logging(format: LogFormat) {
    init_with_dir(format, "info", None);
}
