//! Traits para injecao de dependencia e testabilidade
//!
//! DT-09: mockall::automock gera mocks automaticamente em builds de teste.
//! CI Linux roda testes com mocks; CI Windows roda testes de integracao.

use crate::core::error::CoreError;
use crate::app::messages::*;

/// Provedor de informacoes do sistema — implementacao real usa WMI
#[cfg_attr(test, mockall::automock)]
pub trait SystemInfoProvider: Send + Sync {
    fn cpu(&self) -> Result<CpuInfo, CoreError>;
    fn memory(&self) -> Result<MemoryInfo, CoreError>;
    fn disks(&self) -> Result<Vec<DiskInfo>, CoreError>;
    fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError>;
    fn motherboard(&self) -> Result<MotherboardInfo, CoreError>;
    fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError>;
}

/// Provedor de acesso ao Registry Windows
#[cfg_attr(test, mockall::automock)]
pub trait RegistryProvider: Send + Sync {
    fn read_key(&self, path: &str, name: &str) -> Result<String, CoreError>;
    fn enum_values(&self, path: &str) -> Result<Vec<(String, String)>, CoreError>;
    fn delete_value(&self, path: &str, name: &str) -> Result<(), CoreError>;
    fn enum_subkeys(&self, path: &str) -> Result<Vec<String>, CoreError>;
}

/// Provedor de controle de servicos Windows
#[cfg_attr(test, mockall::automock)]
pub trait ServiceProvider: Send + Sync {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError>;
    fn get_status(&self, name: &str) -> Result<String, CoreError>;
    fn set_start_type(&self, name: &str, start_type: &str) -> Result<(), CoreError>;
    fn stop(&self, name: &str) -> Result<(), CoreError>;
    fn start(&self, name: &str) -> Result<(), CoreError>;
}

/// Provedor de atualizacoes Windows
#[cfg_attr(test, mockall::automock)]
pub trait UpdateProvider: Send + Sync {
    fn search_pending(&self) -> Result<Vec<String>, CoreError>;
    fn install_updates(&self) -> Result<u32, CoreError>;
    fn get_history(&self, limit: u32) -> Result<Vec<String>, CoreError>;
}

/// Fábrica que cria implementacoes reais (Windows) ou mocks (testes)
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn system_info() -> Box<dyn SystemInfoProvider> {
        #[cfg(test)]
        return Box::new(MockSystemInfoProvider::new());
        #[cfg(not(test))]
        return Box::new(crate::platform::windows::wmi::WmiSystemInfoProvider::new());
    }

    pub fn registry() -> Box<dyn RegistryProvider> {
        #[cfg(test)]
        return Box::new(MockRegistryProvider::new());
        #[cfg(not(test))]
        return Box::new(crate::platform::windows::registry::WinRegistryProvider::new());
    }

    pub fn services() -> Box<dyn ServiceProvider> {
        #[cfg(test)]
        return Box::new(MockServiceProvider::new());
        #[cfg(not(test))]
        return Box::new(crate::platform::windows::services::WinServiceProvider::new());
    }

    pub fn updates() -> Box<dyn UpdateProvider> {
        #[cfg(test)]
        return Box::new(MockUpdateProvider::new());
        #[cfg(not(test))]
        return Box::new(crate::platform::windows::updates::WuaUpdateProvider::new());
    }
}
