//! machine_id persistente — separado do estado de manutencao.
//!
//! Correcoes vs alpha.1:
//! - Caminho agora vem de Config (sem `r"C:\\ManutencaoWindows"` duplicado).
//! - Atributo oculto/sistema nao falha silenciosamente: erro logado via tracing.

use crate::config::Config;
use std::path::Path;
use uuid::Uuid;

pub fn get_or_create_machine_id() -> Result<String, MachineIdError> {
    get_or_create_machine_id_with(&Config::load())
}

pub fn get_or_create_machine_id_with(cfg: &Config) -> Result<String, MachineIdError> {
    let path = cfg.machine_id_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(MachineIdError::Io)?;
        let trimmed = content.trim();
        if !trimmed.is_empty() && Uuid::parse_str(trimmed).is_ok() {
            return Ok(trimmed.to_string());
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Tenta primeiro derivar um ID estavel do hardware; UUID v4 e o fallback.
    let id = match stable_hardware_uuid() {
        Some(id) => id,
        None => Uuid::new_v4().to_string(),
    };
    std::fs::write(&path, &id)?;
    #[cfg(windows)]
    {
        match set_hidden_system(&path) {
            Ok(()) => {}
            Err(e) => tracing::warn!("Nao foi possivel ocultar {}: {}", path.display(), e),
        }
    }
    Ok(id)
}

/// Deriva um UUIDv5 estavel a partir de identificadores de hardware (WMI),
/// para que reinstalacoes preservem o mesmo machine_id quando possivel.
fn stable_hardware_uuid() -> Option<String> {
    #[cfg(windows)]
    {
        let out = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-CimInstance Win32_ComputerSystemProduct).UUID",
            ])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if raw.is_empty() || raw.eq_ignore_ascii_case("ffffffff-ffff-ffff-ffff-ffffffffffff") {
            return None; // SMBIOS invalido em maquinas baratas/VMs antigas
        }
        // Namespace custom da aplicacao p/ derivação deterministica.
        let ns = uuid::Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").ok()?;
        Some(uuid::Uuid::new_v5(&ns, raw.as_bytes()).to_string())
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn set_hidden_system(path: &Path) -> Result<(), std::io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        SetFileAttributesW, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        SetFileAttributesW(
            PCWSTR(wide.as_ptr()),
            FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM,
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

pub fn read_machine_id() -> Result<String, MachineIdError> {
    read_machine_id_with(&Config::load())
}

pub fn read_machine_id_with(cfg: &Config) -> Result<String, MachineIdError> {
    let path = cfg.machine_id_path();
    if !path.exists() {
        return Err(MachineIdError::NotFound);
    }
    let content = std::fs::read_to_string(&path).map_err(MachineIdError::Io)?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(MachineIdError::InvalidFormat);
    }
    Uuid::parse_str(trimmed).map_err(|_| MachineIdError::InvalidFormat)?;
    Ok(trimmed.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum MachineIdError {
    #[error("Arquivo machine_id nao encontrado")]
    NotFound,
    #[error("Erro de IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Formato invalido")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("hfb-mid-test-{}", std::process::id()));
        let cfg = Config { base_dir: dir.to_string_lossy().to_string(), ..Default::default() };
        let id = get_or_create_machine_id_with(&cfg).unwrap();
        assert!(Uuid::parse_str(&id).is_ok());
        let again = get_or_create_machine_id_with(&cfg).unwrap();
        assert_eq!(id, again, "machine_id deve ser estavel entre chamadas");
        let read = read_machine_id_with(&cfg).unwrap();
        assert_eq!(id, read);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
