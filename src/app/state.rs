//! Persistencia de estado com versionamento, checksum e migracao segura.
//!
//! Correcoes vs alpha.1:
//! - BASE_DIR hardcoded `r"C:\\ManutencaoWindows"` (barra dupla) removido; agora usa Config.
//! - migrate_v0_to_v1 recalcula o checksum apos injetar metadados (antes gerava checksum invalido).
//! - Falha de serializacao nao gera mais checksum "valido" para dados vazios (usa `?`).

use crate::app::messages::{AuditData, StateError};
use crate::config::Config;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
/// Nome do arquivo de estado (o diretorio vem de Config).
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

    /// Serializa sem checksum e calcula CRC32. Retorna erro se a serializacao falhar
    /// (bug antigo: `unwrap_or_default()` produzia checksum valido p/ string vazia).
    fn payload_json(&self) -> Result<String, StateError> {
        let mut temp = self.clone();
        temp.checksum = String::new();
        serde_json::to_string(&temp).map_err(|e| StateError::Parse(e.to_string()))
    }

    pub fn recalculate_checksum(&mut self) {
        match self.payload_json() {
            Ok(json) => self.checksum = format!("{:08x}", crc32(&json)),
            Err(_) => self.checksum = String::new(),
        }
    }

    pub fn verify_checksum(&self) -> bool {
        match self.payload_json() {
            Ok(json) => format!("{:08x}", crc32(&json)) == self.checksum,
            Err(_) => false,
        }
    }
}

fn crc32(data: &str) -> u32 {
    let mut crc: u32 = 0xffffffff;
    for byte in data.bytes() {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xedb88320 } else { crc >> 1 };
        }
    }
    !crc
}

pub fn state_path(cfg: &Config) -> PathBuf {
    cfg.state_path()
}

pub fn load_state() -> Result<StateFile, StateError> {
    load_state_with(&Config::load())
}

pub fn load_state_with(cfg: &Config) -> Result<StateFile, StateError> {
    let path = cfg.state_path();
    if !path.exists() {
        return Err(StateError::NotFound);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| StateError::Io(e.to_string()))?;
    let raw: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| StateError::Parse(e.to_string()))?;
    let version = raw.get("schema_version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    match version {
        0 => migrate_v0_to_v1(raw),
        1 => {
            let state: StateFile =
                serde_json::from_value(raw).map_err(|e| StateError::Parse(e.to_string()))?;
            if !state.verify_checksum() {
                return Err(StateError::InvalidChecksum);
            }
            Ok(state)
        }
        _ => Err(StateError::UnsupportedVersion(version)),
    }
}

pub fn save_state(state: &StateFile) -> Result<(), StateError> {
    save_state_with(state, &Config::load())
}

pub fn save_state_with(state: &StateFile, cfg: &Config) -> Result<(), StateError> {
    let base = cfg.resolved_base_dir();
    std::fs::create_dir_all(&base).map_err(|e| StateError::Io(e.to_string()))?;
    let path = cfg.state_path();
    // Backup rotativo simples: .bak guarda a versao anterior.
    if path.exists() {
        let _ = std::fs::copy(&path, format!("{}.bak", path.display()));
    }
    let mut state = state.clone();
    state.recalculate_checksum();
    let json = serde_json::to_string_pretty(&state).map_err(|e| StateError::Parse(e.to_string()))?;
    std::fs::write(&path, json).map_err(|e| StateError::Io(e.to_string()))?;
    Ok(())
}

/// Migra v0 -> v1 e RECALCULA o checksum (correcao P1: antes devolvia checksum zerado/invalido).
fn migrate_v0_to_v1(mut raw: serde_json::Value) -> Result<StateFile, StateError> {
    use crate::app::machine_id;
    let mid = machine_id::get_or_create_machine_id()
        .map_err(|e| StateError::Io(e.to_string()))?;
    raw["schema_version"] = serde_json::json!(CURRENT_SCHEMA_VERSION);
    raw["machine_id"] = serde_json::json!(mid);
    raw["app_version"] = serde_json::json!(env!("CARGO_PKG_VERSION"));
    raw["checksum"] = serde_json::json!("");
    let now = serde_json::json!(Utc::now().to_rfc3339());
    if raw.get("created_at").is_none() {
        raw["created_at"] = now.clone();
    }
    raw["updated_at"] = now;
    if raw.get("phases_executed").is_none() {
        raw["phases_executed"] = serde_json::json!(Vec::<String>::new());
    }
    let mut state: StateFile =
        serde_json::from_value(raw).map_err(|e| StateError::Parse(e.to_string()))?;
    state.recalculate_checksum();
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_roundtrip_valid() {
        let mut s = StateFile::new("test-id".into());
        assert!(s.verify_checksum());
        s.add_phase("1");
        assert!(s.verify_checksum());
        s.audit_data = Some(Box::new(AuditData::default()));
        s.recalculate_checksum();
        assert!(s.verify_checksum());
    }

    #[test]
    fn tampered_state_fails_checksum() {
        let mut s = StateFile::new("test-id".into());
        s.add_phase("1");
        s.phases_executed.push("9".into()); // alterado sem recalcular
        assert!(!s.verify_checksum());
    }

    #[test]
    fn migration_recalculates_checksum() {
        let raw = serde_json::json!({
            "audit_data": null,
            "cleanup_data": null,
            "phases_executed": ["1"],
        });
        let migrated = migrate_v0_to_v1(raw).unwrap();
        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(migrated.verify_checksum(), "migracao deve gerar checksum valido");
    }
}
