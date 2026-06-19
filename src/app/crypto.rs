//! Criptografia de estado via Windows Credential Manager
//!
//! DT-04: Chave AES-GCM armazenada no Credential Manager, nao derivada de HW ID.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

const CRED_TARGET_NAME: &str = "HighlanderForgeBlade:StateKey";

pub fn encrypt_state(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // Stub — implementar com windows::Win32::Security::Credentials
    let mut result = Vec::new();
    result.extend_from_slice(b"ENCRYPTED:");
    result.extend_from_slice(plaintext);
    Ok(result)
}

pub fn decrypt_state(ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.starts_with(b"ENCRYPTED:") {
        Ok(ciphertext[10..].to_vec())
    } else {
        Ok(ciphertext.to_vec())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Erro de criptografia: {0}")]
    Generic(String),
}
