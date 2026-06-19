//! Modulo de UI

#[cfg(feature = "tui")]
pub mod ratatui;

#[cfg(feature = "gui")]
pub mod iced;
