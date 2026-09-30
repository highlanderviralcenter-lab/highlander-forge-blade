//! Mensagens do sistema

use crate::core::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AppMsg {
    Tick, Shutdown, NavigateUp, NavigateDown, Select, Back,
    AuditStarted,
    AuditProgress { phase: AuditPhase, item: String, percent: u8 },
    AuditCompleted(Box<AuditData>),
    AuditFailed(CoreError),
    SummaryDisplayed,
    UserConfirmed(bool),
    CleanupStarted,
    CleanupProgress { operation: CleanupOp, detail: String, percent: u8, bytes_freed: u64 },
    CleanupCompleted,
    CleanupFailed(CoreError),
    RebootScheduled, RebootCancelled,
    PostRebootStarted,
    PostRebootProgress { tool: RepairTool, percent: u8, detail: String },
    PostRebootCompleted,
    PostRebootFailed(CoreError),
    LogLine(LogEntry),
    Error(CoreError),
    StateSaved,
    StateLoaded(Result<crate::app::state::StateFile, StateError>),
    ReportGenerated(ReportFormat),
    UpdateCheckStarted,
    UpdateAvailable(String),
    UpdateNotAvailable,
    UpdateFailed(CoreError),
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

impl LogEntry {
    pub fn info(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self { timestamp: Utc::now(), level: LogLevel::Info, source: source.into(), message: message.into() }
    }
    pub fn warn(message: impl Into<String>) -> Self {
        Self { timestamp: Utc::now(), level: LogLevel::Warn, source: "system".to_string(), message: message.into() }
    }
    pub fn success(message: impl Into<String>) -> Self {
        Self { timestamp: Utc::now(), level: LogLevel::Success, source: "system".to_string(), message: message.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel { Debug, Info, Warn, Error, Success, Phase }

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Success => write!(f, "SUCCESS"),
            LogLevel::Phase => write!(f, "PHASE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditPhase { Hardware, Software, Updates, Services, Registry, Environment }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupOp { TempFiles, BrowserCache, RecycleBin, OldLogs, WindowsUpdates, ServicesOptimize, RegistryClean }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairTool { Sfc, Dism, Chkdsk }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat { Html, Txt, Json }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditData {
    pub cpu: Option<CpuInfo>, pub memory: Option<MemoryInfo>, pub disks: Vec<DiskInfo>,
    pub gpus: Vec<GpuInfo>, pub motherboard: Option<MotherboardInfo>,
    pub temperatures: Vec<TemperatureReading>, pub software: Vec<SoftwareInfo>,
    pub services: Vec<ServiceInfo>, pub registry_run_keys: Vec<RunKey>,
    pub environment: EnvironmentVars,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String, pub manufacturer: String, pub cores: u32, pub threads: u32,
    pub max_speed_mhz: u32, pub architecture: String, pub socket: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64, pub modules: Vec<MemoryModule>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryModule {
    pub slot: String, pub capacity_bytes: u64, pub speed_mhz: u32, pub manufacturer: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device_id: String, pub volume_name: String, pub filesystem: String,
    pub total_bytes: u64, pub free_bytes: u64, pub used_bytes: u64, pub percent_free: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String, pub manufacturer: String, pub adapter_ram_bytes: u64,
    pub resolution: String, pub driver_version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String, pub product: String, pub version: String, pub serial_number: String,
    pub bios_vendor: String, pub bios_version: String, pub bios_date: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub zone: String, pub celsius: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub display_name: String, pub display_version: String, pub publisher: String,
    pub install_date: String, pub install_location: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String, pub display_name: String, pub state: String, pub start_mode: String,
    pub is_third_party: bool, pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunKey {
    pub hive: String, pub name: String, pub value: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentVars {
    pub system: Vec<(String, String)>, pub user: Vec<(String, String)>,
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum StateError {
    #[error("Arquivo de estado nao encontrado")] NotFound,
    #[error("Erro de IO: {0}")] Io(String),
    #[error("Erro de parse JSON: {0}")] Parse(String),
    #[error("Versao de schema nao suportada: {0}")] UnsupportedVersion(u32),
    #[error("Checksum invalido")] InvalidChecksum,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub current_screen: Screen,
    pub selected_menu_item: usize,
    pub audit_data: Option<Box<AuditData>>,
    pub progress: f32,
    pub logs: Vec<LogEntry>,
    pub status_message: String,
    pub is_simulation: bool,
    pub phases_completed: Vec<String>,
    /// Bytes efetivamente liberados acumulados na sessao (Fase 3).
    pub bytes_freed_total: u64,
}

impl AppState {
    pub fn update(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::Tick => {}
            // Limite real do menu (MENU_ITEM_COUNT itens) — antes hardcoded `< 10`.
            AppMsg::NavigateUp => { if self.selected_menu_item > 0 { self.selected_menu_item -= 1; } }
            AppMsg::NavigateDown => {
                if self.selected_menu_item + 1 < crate::MENU_ITEM_COUNT {
                    self.selected_menu_item += 1;
                }
            }
            AppMsg::Select => self.handle_menu_select(),
            AppMsg::Back => { self.current_screen = Screen::Menu; }
            AppMsg::AuditStarted => {
                self.current_screen = Screen::AuditProgress;
                self.progress = 0.0;
                self.status_message = "Iniciando auditoria...".to_string();
            }
            AppMsg::AuditProgress { percent, ref item, .. } => {
                tracing::debug!(target: "hfb_state", "AuditProgress handler: item={} percent={}", item, percent);
                self.progress = percent as f32;
                self.status_message = format!("Coletando: {}", item);
                self.logs.push(LogEntry::info("audit", format!("{} - {}%", item, percent)));
            }
            AppMsg::AuditCompleted(data) => {
                self.audit_data = Some(data);
                self.current_screen = Screen::Summary;
                self.progress = 100.0;
                self.status_message = "Auditoria concluida!".to_string();
                if !self.phases_completed.iter().any(|p| p == "1") {
                    self.phases_completed.push("1".to_string());
                }
            }
            AppMsg::AuditFailed(ref err) => {
                self.current_screen = Screen::Menu;
                self.status_message = format!("Erro: {}", err);
                self.logs.push(LogEntry::warn(format!("Auditoria falhou: {}", err)));
            }
            AppMsg::CleanupStarted => {
                self.current_screen = Screen::CleanupProgress;
                self.progress = 0.0;
                self.bytes_freed_total = 0;
            }
            AppMsg::CleanupProgress { percent, ref detail, bytes_freed, .. } => {
                self.progress = percent as f32;
                self.bytes_freed_total = self.bytes_freed_total.max(bytes_freed);
                self.status_message = format!("{} ({} MB liberados)", detail, bytes_freed / 1_048_576);
            }
            AppMsg::CleanupCompleted => {
                self.current_screen = Screen::RebootConfirm;
                self.progress = 100.0;
                self.status_message = format!("Limpeza concluida: {} MB liberados", self.bytes_freed_total / 1_048_576);
                if !self.phases_completed.iter().any(|p| p == "3") {
                    self.phases_completed.push("3".to_string());
                }
            }
            AppMsg::CleanupFailed(ref err) => {
                self.current_screen = Screen::Menu;
                self.status_message = format!("Limpeza falhou: {}", err);
                self.logs.push(LogEntry::warn(format!("Limpeza falhou: {}", err)));
            }
            AppMsg::UserConfirmed(true) => { self.current_screen = Screen::PostRebootProgress; }
            AppMsg::UserConfirmed(false) => { self.current_screen = Screen::Menu; }
            AppMsg::RebootScheduled => {
                self.status_message = "Reinicializacao agendada".to_string();
                self.logs.push(LogEntry::success("Reinicializacao agendada para 15s"));
                if !self.phases_completed.iter().any(|p| p == "4") {
                    self.phases_completed.push("4".to_string());
                }
            }
            AppMsg::RebootCancelled => {
                self.status_message = "Reinicializacao cancelada".to_string();
                self.logs.push(LogEntry::warn("Reinicializacao cancelada pelo usuario"));
            }
            AppMsg::PostRebootStarted => {
                self.current_screen = Screen::PostRebootProgress;
                self.progress = 0.0;
            }
            AppMsg::PostRebootProgress { percent, ref detail, .. } => {
                self.progress = percent as f32;
                self.status_message = detail.clone();
            }
            AppMsg::PostRebootCompleted => {
                self.current_screen = Screen::ReportView;
                self.progress = 100.0;
                if !self.phases_completed.iter().any(|p| p == "5") {
                    self.phases_completed.push("5".to_string());
                }
            }
            AppMsg::PostRebootFailed(ref err) => {
                self.status_message = format!("Pos-reboot falhou: {}", err);
                self.logs.push(LogEntry::warn(format!("Pos-reboot falhou: {}", err)));
            }
            AppMsg::ReportGenerated(_) => { self.current_screen = Screen::ReportView; }
            AppMsg::LogLine(entry) => {
                self.logs.push(entry);
                if self.logs.len() > 500 { self.logs.remove(0); }
            }
            AppMsg::Error(ref err) => {
                self.status_message = format!("Erro: {}", err);
                self.logs.push(LogEntry::warn(format!("Erro: {}", err)));
            }
            AppMsg::StateSaved => { self.status_message = "Estado salvo".to_string(); }
            AppMsg::StateLoaded(Ok(state)) => {
                self.status_message = "Estado carregado".to_string();
                if state.audit_data.is_some() { self.audit_data = state.audit_data; }
                self.phases_completed = state.phases_executed;
            }
            AppMsg::StateLoaded(Err(ref e)) => { self.status_message = format!("Erro ao carregar: {}", e); }
            AppMsg::UpdateAvailable(ref v) => { self.status_message = format!("Update {} disponivel", v); }
            AppMsg::UpdateNotAvailable => { self.status_message = "Nenhum update".to_string(); }
            AppMsg::UpdateFailed(ref err) => { self.status_message = format!("Falha no update: {}", err); }
            AppMsg::Shutdown => {}
        }
    }

    fn handle_menu_select(&mut self) {
        match self.selected_menu_item {
            0 | 1 => { /* Audit iniciada pelo app.rs */ }
            2 | 3 => {
                if self.audit_data.is_some() { self.current_screen = Screen::Summary; }
                else { self.status_message = "Execute Fase 1 primeiro".to_string(); self.logs.push(LogEntry::warn("Execute Fase 1 primeiro")); }
            }
            4 | 5 => { self.current_screen = Screen::CleanupProgress; self.progress = 0.0; }
            6 | 7 => { self.current_screen = Screen::RebootConfirm; }
            8 => { self.current_screen = Screen::PostRebootProgress; self.progress = 0.0; }
            9 => {
                if self.audit_data.is_some() { self.current_screen = Screen::ReportView; }
                else { self.status_message = "Nenhum dado de auditoria".to_string(); self.logs.push(LogEntry::warn("Execute Fase 1 primeiro")); }
            }
            10 => { self.logs.push(LogEntry::info("menu", "Saindo...")); }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default] Menu,
    AuditProgress, Summary, CleanupProgress, RebootConfirm,
    PostRebootProgress, ReportView, LogsView,
}

