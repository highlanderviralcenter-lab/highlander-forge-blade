//! Mensagens do sistema — canal unico entre async tasks e UI

use crate::core::error::CoreError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum AppMsg {
    Tick,
    Shutdown,
    NavigateUp,
    NavigateDown,
    Select,
    Back,
    AuditStarted,
    AuditProgress {
        phase: AuditPhase,
        item: String,
        percent: u8,
    },
    AuditCompleted(Box<AuditData>),
    AuditFailed(CoreError),
    SummaryDisplayed,
    UserConfirmed(bool),
    CleanupStarted,
    CleanupProgress {
        operation: CleanupOp,
        detail: String,
        percent: u8,
        bytes_freed: u64,
    },
    CleanupCompleted,
    CleanupFailed(CoreError),
    RebootScheduled,
    RebootCancelled,
    PostRebootStarted,
    PostRebootProgress {
        tool: RepairTool,
        percent: u8,
        detail: String,
    },
    PostRebootCompleted,
    PostRebootFailed(CoreError),
    LogLine(LogEntry),
    Error(CoreError),
    StateSaved,
    StateLoaded(Result<AppState, StateError>),
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
        Self {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            source: source.into(),
            message: message.into(),
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: LogLevel::Warn,
            source: "system".to_string(),
            message: message.into(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: LogLevel::Success,
            source: "system".to_string(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Success,
    Phase,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditPhase {
    Hardware,
    Software,
    Updates,
    Services,
    Registry,
    Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupOp {
    TempFiles,
    BrowserCache,
    RecycleBin,
    OldLogs,
    WindowsUpdates,
    ServicesOptimize,
    RegistryClean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairTool {
    Sfc,
    Dism,
    Chkdsk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Html,
    Txt,
    Json,
}

#[derive(Debug, Clone, Default)]
pub struct AuditData {
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskInfo>,
    pub gpus: Vec<GpuInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub temperatures: Vec<TemperatureReading>,
    pub software: Vec<SoftwareInfo>,
    pub services: Vec<ServiceInfo>,
    pub registry_run_keys: Vec<RunKey>,
    pub environment: EnvironmentVars,
}

#[derive(Debug, Clone, Default)]
pub struct CpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub cores: u32,
    pub threads: u32,
    pub max_speed_mhz: u32,
    pub architecture: String,
    pub socket: String,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub modules: Vec<MemoryModule>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryModule {
    pub slot: String,
    pub capacity_bytes: u64,
    pub speed_mhz: u32,
    pub manufacturer: String,
}

#[derive(Debug, Clone, Default)]
pub struct DiskInfo {
    pub device_id: String,
    pub volume_name: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub percent_free: f64,
}

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub adapter_ram_bytes: u64,
    pub resolution: String,
    pub driver_version: String,
}

#[derive(Debug, Clone, Default)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial_number: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_date: String,
}

#[derive(Debug, Clone, Default)]
pub struct TemperatureReading {
    pub zone: String,
    pub celsius: f64,
}

#[derive(Debug, Clone, Default)]
pub struct SoftwareInfo {
    pub display_name: String,
    pub display_version: String,
    pub publisher: String,
    pub install_date: String,
    pub install_location: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_mode: String,
    pub is_third_party: bool,
    pub path: String,
}

#[derive(Debug, Clone, Default)]
pub struct RunKey {
    pub hive: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct EnvironmentVars {
    pub system: Vec<(String, String)>,
    pub user: Vec<(String, String)>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum StateError {
    #[error("Arquivo de estado nao encontrado")]
    NotFound,
    #[error("Erro de IO: {0}")]
    Io(String),
    #[error("Erro de parse JSON: {0}")]
    Parse(String),
    #[error("Versao de schema nao suportada: {0}")]
    UnsupportedVersion(u32),
    #[error("Checksum invalido — estado pode estar corrompido")]
    InvalidChecksum,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Menu,
    AuditProgress,
    Summary,
    CleanupProgress,
    RebootConfirm,
    PostRebootProgress,
    ReportView,
    LogsView,
}
