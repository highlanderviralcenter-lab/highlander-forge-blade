//! Highlander Forge Blade - Biblioteca principal
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod app;
pub mod config;
pub mod core;
pub mod logging;
pub mod platform;
pub mod ui;
pub mod utils;

/// Re-exports comuns
pub use core::error::CoreError;
pub use core::audit::Auditor;
pub use app::messages::{AppMsg, Screen, LogEntry, AuditPhase, CleanupOp};

/// Versao do schema de estado persistente
pub const STATE_SCHEMA_VERSION: u32 = 1;
