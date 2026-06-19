//! Implementacao real de ServiceProvider

use crate::app::messages::ServiceInfo;
use crate::core::error::CoreError;
use crate::core::traits::ServiceProvider;

pub struct WinServiceProvider;

impl WinServiceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceProvider for WinServiceProvider {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError> {
        Ok(Vec::new())
    }

    fn get_status(&self, _name: &str) -> Result<String, CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn set_start_type(&self, _name: &str, _start_type: &str) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn stop(&self, _name: &str) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }

    fn start(&self, _name: &str) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("TODO".to_string()))
    }
}
