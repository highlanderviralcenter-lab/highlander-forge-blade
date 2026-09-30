//! Highlander Forge Blade - Biblioteca principal
//!
//! Ferramenta profissional de manutencao do Windows: engine Rust, UI TUI/GUI,
//! modo headless para RMM/MSP.

pub mod app;
pub mod config;
pub mod core;
pub mod logging;
pub mod platform;
pub mod ui;
pub mod utils;

/// Re-exports comuns
pub use app::messages::{AppMsg, AuditPhase, CleanupOp, LogEntry, Screen};
pub use config::Config;
pub use core::audit::Auditor;
pub use core::error::CoreError;

/// Versao do schema de estado persistente
pub const STATE_SCHEMA_VERSION: u32 = 1;

/// Numero de itens do menu principal (fonte unica de verdade para navegacao).
/// Manter sincronizado com `MENU_ITEMS` em `ui::ratatui::views::menu`.
pub const MENU_ITEM_COUNT: usize = 11;

/// Codigo de saida usado quando o processo exige elevacao e nao foi elevado.
pub const EXIT_NOT_ELEVATED: i32 = 78; // EX_CONFIG (sysexits)
