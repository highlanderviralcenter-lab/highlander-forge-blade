use serde::{Deserialize, Serialize};
use crate::app::messages::{Screen, LogEntry};
use crate::core::audit::AuditData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_screen: Screen,
    pub menu_selected: usize,
    pub confirm_selected: bool, // true = Sim, false = Nao
    pub progress_percent: u16,
    pub current_phase: String,
    pub logs: Vec<LogEntry>,
    pub audit_data: Option<AuditData>,
    pub schema_version: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Menu,
            menu_selected: 0,
            confirm_selected: true,
            progress_percent: 0,
            current_phase: String::new(),
            logs: Vec::new(),
            audit_data: None,
            schema_version: 1,
        }
    }
}