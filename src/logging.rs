//! Logging dual-mode: humano (TUI) e JSON (headless)

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

pub fn init_logging(format: LogFormat) {
    match format {
        LogFormat::Human => init_human(),
        LogFormat::Json => init_json(),
    }
}

fn init_human() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive("hfb=info".parse().unwrap()))
        .with(fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_level(true)
            .with_ansi(true)
            .with_timer(fmt::time::ChronoLocal::rfc_3339()))
        .init();
}

fn init_json() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env()
            .add_directive("hfb=info".parse().unwrap()))
        .with(fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_level(true)
            .with_current_span(true)
            .with_timer(fmt::time::ChronoLocal::rfc_3339()))
        .init();
}
