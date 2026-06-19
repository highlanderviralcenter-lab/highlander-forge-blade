//! Auto-update com verificacao Ed25519 (DT-11)

use crate::core::error::CoreError;

pub async fn check_update() -> Result<Option<String>, CoreError> {
    tracing::info!("Verificando atualizacoes...");
    Ok(None)
}

pub async fn download_and_verify(_url: &str, _expected_sig: &str) -> Result<Vec<u8>, CoreError> {
    Err(CoreError::NotSupported("Auto-update ainda nao implementado".to_string()))
}
