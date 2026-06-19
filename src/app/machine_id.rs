//! machine_id persistente — separado do estado de manutencao

use std::path::Path;
use uuid::Uuid;

const MACHINE_ID_FILE: &str = "machine_id";
const BASE_DIR: &str = r"C:\\ManutencaoWindows";

pub fn get_or_create_machine_id() -> Result<String, MachineIdError> {
    let path = Path::new(BASE_DIR).join(MACHINE_ID_FILE);
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(MachineIdError::Io)?;
        let trimmed = content.trim();
        if !trimmed.is_empty() && Uuid::parse_str(trimmed).is_ok() {
            return Ok(trimmed.to_string());
        }
    }
    std::fs::create_dir_all(BASE_DIR)?;
    let id = Uuid::new_v4().to_string();
    std::fs::write(&path, &id)?;
    #[cfg(windows)]
    unsafe {
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Storage::FileSystem::SetFileAttributesW;
        use windows::core::PCWSTR;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let _ = SetFileAttributesW(PCWSTR(wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN
            | windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SYSTEM);
    }
    Ok(id)
}

pub fn read_machine_id() -> Result<String, MachineIdError> {
    let path = Path::new(BASE_DIR).join(MACHINE_ID_FILE);
    if !path.exists() { return Err(MachineIdError::NotFound); }
    let content = std::fs::read_to_string(&path).map_err(MachineIdError::Io)?;
    let trimmed = content.trim();
    if trimmed.is_empty() { return Err(MachineIdError::InvalidFormat); }
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
