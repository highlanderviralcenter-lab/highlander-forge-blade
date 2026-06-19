//! Persistencia de estado com versionamento

use crate::app::messages::{AuditData, StateError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const BASE_DIR: &str = r"C:\ManutencaoWindows";
pub const STATE_FILE: &str = "estado_manutencao.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFile {
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub machine_id: String,
    pub app_version: String,
    pub checksum: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_data: Option<Box<AuditData>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_data: Option<CleanupData>,

    pub phases_executed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanupData {
    pub bytes_freed: u64,
    pub services_disabled: Vec<String>,
    pub registry_keys_removed: Vec<String>,
    pub updates_installed: Vec<String>,
}

impl StateFile {
    pub fn new(machine_id: String) -> Self {
        let now = Utc::now();
        let mut s = Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            created_at: now,
            updated_at: now,
            machine_id,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            checksum: String::new(),
            audit_data: None,
            cleanup_data: None,
            phases_executed: Vec::new(),
        };
        s.recalculate_checksum();
        s
    }

    pub fn add_phase(&mut self, phase: &str) {
        if !self.phases_executed.contains(&phase.to_string()) {
            self.phases_executed.push(phase.to_string());
        }
        self.updated_at = Utc::now();
        self.recalculate_checksum();
    }

    fn recalculate_checksum(&mut self) {
        let old = self.checksum.clone();
        self.checksum = String::new();
        let json = serde_json::to_string(self).unwrap_or_default();
        self.checksum = format!("{:08x}", crc32(&json));
        let _ = old;
    }

    pub fn verify_checksum(&self) -> bool {
        let mut temp = self.clone();
        temp.checksum = String::new();
        let json = serde_json::to_string(&temp).unwrap_or_default();
        let calculated = format!("{:08x}", crc32(&json));
        calculated == self.checksum
    }
}

fn crc32(data: &str) -> u32 {
    let mut crc: u32 = 0xffffffff;
    for byte in data.bytes() {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

pub fn load_state() -> Result<StateFile, StateError> {
    let path = state_path();

    if !path.exists() {
        return Err(StateError::NotFound);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| StateError::Io(e.to_string()))?;

    let raw: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| StateError::Parse(e.to_string()))?;

    let version = raw
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    match version {
        0 => migrate_v0_to_v1(raw),
        1 => {
            let state: StateFile = serde_json::from_value(raw)
                .map_err(|e| StateError::Parse(e.to_string()))?;
            if !state.verify_checksum() {
                return Err(StateError::InvalidChecksum);
            }
            Ok(state)
        }
        _ => Err(StateError::UnsupportedVersion(version)),
    }
}

pub fn save_state(state: &StateFile) -> Result<(), StateError> {
    let path = state_path();
    std::fs::create_dir_all(BASE_DIR)
        .map_err(|e| StateError::Io(e.to_string()))?;

    if path.exists() {
        let backup = format!("{}.bak", path.display());
        let _ = std::fs::copy(&path, &backup);
    }

    let mut state = state.clone();
    state.recalculate_checksum();

    let json = serde_json::to_string_pretty(&state)
        .map_err(|e| StateError::Parse(e.to_string()))?;

    std::fs::write(&path, json)
        .map_err(|e| StateError::Io(e.to_string()))?;

    Ok(())
}

fn state_path() -> PathBuf {
    Path::new(BASE_DIR).join(STATE_FILE)
}

fn migrate_v0_to_v1(mut raw: serde_json::Value) -> Result<StateFile, StateError> {
    use crate::app::machine_id;

    raw["schema_version"] = serde_json::json!(1);
    raw["machine_id"] = serde_json::json!(machine_id::get_or_create_machine_id()
        .map_err(|e| StateError::Io(e.to_string()))?);
    raw["app_version"] = serde_json::json!(env!("CARGO_PKG_VERSION"));
    raw["checksum"] = serde_json::json!("");

    if raw.get("phases_executed").is_none() {
        raw["phases_executed"] = serde_json::json!(Vec::<String>::new());
    }

    serde_json::from_value(raw).map_err(|e| StateError::Parse(e.to_string()))
}
