pub mod app;
pub mod core;
pub mod config;
pub mod logging;
pub mod update;
pub mod utils;

#[cfg(feature = "tui")]
pub mod ui;

#[cfg(all(feature = "gui", not(feature = "tui")))]
pub mod ui;

pub use app::state::AppState;
pub use app::messages::AppMsg;
