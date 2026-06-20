//! Traits para injecao de dependencia e testabilidade (DT-09)

use crate::core::error::CoreError;
use crate::app::messages::*;

#[cfg_attr(test, mockall::automock)]
pub trait SystemInfoProvider: Send + Sync {
    fn cpu(&self) -> Result<CpuInfo, CoreError>;
    fn memory(&self) -> Result<MemoryInfo, CoreError>;
    fn disks(&self) -> Result<Vec<DiskInfo>, CoreError>;
    fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError>;
    fn motherboard(&self) -> Result<MotherboardInfo, CoreError>;
    fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait RegistryProvider: Send + Sync {
    fn read_key(&self, path: &str, name: &str) -> Result<String, CoreError>;
    fn enum_values(&self, path: &str) -> Result<Vec<(String, String)>, CoreError>;
    fn delete_value(&self, path: &str, name: &str) -> Result<(), CoreError>;
    fn enum_subkeys(&self, path: &str) -> Result<Vec<String>, CoreError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait ServiceProvider: Send + Sync {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError>;
    fn get_status(&self, name: &str) -> Result<String, CoreError>;
    fn set_start_type(&self, name: &str, start_type: &str) -> Result<(), CoreError>;
    fn stop(&self, name: &str) -> Result<(), CoreError>;
    fn start(&self, name: &str) -> Result<(), CoreError>;
}

// NOVO: CleanupProvider — necessario para core/cleanup.rs
#[cfg_attr(test, mockall::automock)]
pub trait CleanupProvider: Send + Sync {
    fn clean_temp_files(&self) -> Result<u64, CoreError>;
    fn clean_recycle_bin(&self) -> Result<u64, CoreError>;
    fn clean_browser_cache(&self) -> Result<u64, CoreError>;
    fn run_dism(&self) -> Result<(), CoreError>;
    fn run_sfc(&self) -> Result<(), CoreError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait UpdateProvider: Send + Sync {
    fn search_pending(&self) -> Result<Vec<String>, CoreError>;
    fn install_updates(&self) -> Result<u32, CoreError>;
    fn get_history(&self, limit: u32) -> Result<Vec<String>, CoreError>;
}

/// Fabrica que cria implementacoes REAIS. Testes injetam mocks diretamente.
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn system_info() -> Box<dyn SystemInfoProvider> {
        Box::new(crate::platform::windows::wmi::WmiSystemInfoProvider::new())
    }
    pub fn registry() -> Box<dyn RegistryProvider> {
        Box::new(crate::platform::windows::registry::WinRegistryProvider::new())
    }
    pub fn services() -> Box<dyn ServiceProvider> {
        Box::new(crate::platform::windows::services::WinServiceProvider::new())
    }
    pub fn updates() -> Box<dyn UpdateProvider> {
        Box::new(crate::platform::windows::updates::WuaUpdateProvider::new())
    }
    // NOVO
    pub fn cleanup() -> Box<dyn CleanupProvider> {
        Box::new(crate::core::cleanup::WinCleanupProvider::new())
    }
}
