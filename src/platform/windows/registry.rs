//! Implementacao real de RegistryProvider

use crate::core::error::CoreError;
use crate::core::traits::RegistryProvider;

pub struct WinRegistryProvider;

impl WinRegistryProvider {
    pub fn new() -> Self {
        Self
    }
}

impl RegistryProvider for WinRegistryProvider {
    fn read_key(&self, _path: &str, _name: &str) -> Result<String, CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn enum_values(&self, _path: &str) -> Result<Vec<(String, String)>, CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn delete_value(&self, _path: &str, _name: &str) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn enum_subkeys(&self, _path: &str) -> Result<Vec<String>, CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }
}
