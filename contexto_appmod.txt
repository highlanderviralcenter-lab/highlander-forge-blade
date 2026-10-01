pub mod commands;
pub mod crypto;
pub mod events;
pub mod headless;
pub mod machine_id;
pub mod messages;
pub mod state;
pub mod update;

pub use messages::{
    AppState, AppMsg, Screen, LogEntry, LogLevel, AuditData, AuditPhase,
    CleanupOp, RepairTool, ReportFormat, StateError,
    CpuInfo, MemoryInfo, MemoryModule, DiskInfo, GpuInfo, MotherboardInfo,
};
pub use state::{load_state, save_state, StateFile, CleanupData, BASE_DIR};
