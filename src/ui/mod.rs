//! Camadas de UI — TUI (ratatui) e GUI (iced futuro)

#[cfg(feature = "tui")]
pub mod ratatui;

#[cfg(feature = "gui")]
pub mod iced;
