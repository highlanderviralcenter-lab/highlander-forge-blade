use serde::{Deserialize, Serialize};
use crate::core::audit::AuditData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Navegação
    NavigateTo(Screen),
    GoBack,
    Quit,

    // Auditoria
    StartAudit,
    AuditProgress { phase: String, percent: u16, log: String },
    AuditComplete(AuditData),
    AuditError(String),

    // Limpeza
    StartCleanup,
    CleanupProgress { item: String, percent: u16 },
    CleanupComplete,

    // Reboot
    ConfirmReboot(bool),
    ScheduleReboot,

    // UI
    Tick,
    KeyPressed(char),
    ScrollUp,
    ScrollDown,
    ToggleDetailedView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Screen {
    Menu,
    Progress,
    Summary,
    Confirm,
    Report,
    Logs,
    Detailed,  // Modo HWMonitor-style
}

impl Default for Screen {
    fn default() -> Self { Screen::Menu }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub timestamp: String,
    pub message: String,
}

impl LogEntry {
    pub fn info(msg: impl Into<String>) -> Self {
        Self { level: LogLevel::Info, timestamp: chrono::Local::now().format("%H:%M:%S").to_string(), message: msg.into() }
    }
    pub fn warn(msg: impl Into<String>) -> Self {
        Self { level: LogLevel::Warn, timestamp: chrono::Local::now().format("%H:%M:%S").to_string(), message: msg.into() }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self { level: LogLevel::Error, timestamp: chrono::Local::now().format("%H:%M:%S").to_string(), message: msg.into() }
    }
    pub fn success(msg: impl Into<String>) -> Self {
        Self { level: LogLevel::Success, timestamp: chrono::Local::now().format("%H:%M:%S").to_string(), message: msg.into() }
    }
}