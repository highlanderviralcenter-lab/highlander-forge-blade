//! machine_id persistente — separado do estado de manutencao

use std::path::Path;
use uuid::Uuid;

const MACHINE_ID_FILE: &str = "machine_id";
const BASE_DIR: &str = r"C:\ManutencaoWindows";

pub fn get_or_create_machine_id() -> Result<String, std::io::Error> {
    let path = Path::new(BASE_DIR).join(MACHINE_ID_FILE);

    if path.exists() {
        let id = std::fs::read_to_string(&path)?;
        let trimmed = id.trim();
        if Uuid::parse_str(trimmed).is_ok() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(BASE_DIR)?;

    let id = Uuid::new_v4().to_string();
    std::fs::write(&path, &id)?;

    Ok(id)
}

pub fn read_machine_id() -> Result<String, MachineIdError> {
    let path = Path::new(BASE_DIR).join(MACHINE_ID_FILE);

    if !path.exists() {
        return Err(MachineIdError::NotFound);
    }

    let id = std::fs::read_to_string(&path).map_err(MachineIdError::Io)?;
    let trimmed = id.trim();

    Uuid::parse_str(trimmed)
        .map_err(|_| MachineIdError::InvalidFormat)?;

    Ok(trimmed.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum MachineIdError {
    #[error("Arquivo machine_id nao encontrado")]
    NotFound,
    #[error("Erro de IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Formato invalido — nao e um UUID valido")]
    InvalidFormat,
}
