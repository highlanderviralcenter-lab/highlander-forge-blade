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

// CleanupProvider — operacoes mutativas da Fase 3.
// As variantes *_detailed retornam (bytes_liberados, arquivos_removidos) medidos
// APOS a exclusao efetiva (correcao P0: contagem honesta, sem inflacao).
#[cfg_attr(test, mockall::automock)]
pub trait CleanupProvider: Send + Sync {
    fn clean_temp_files_detailed(&self, plan: &crate::core::cleanup::CleanupPlan) -> Result<(u64, u64), CoreError>;
    fn clean_recycle_bin_detailed(&self, all_volumes: bool) -> Result<(u64, u64), CoreError>;
    fn clean_old_logs_detailed(&self, min_age: std::time::Duration) -> Result<(u64, u64), CoreError>;
    fn clean_windows_update_cache(&self) -> Result<u64, CoreError>;
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

/// Fabrica que cria implementacoes REAIS da plataforma.
/// Em Windows, retorna providers nativos (WMI/CIM, registro, servicos, WUA).
/// Em outras plataformas retorna stubs inofensivos — permite testes de CI e
/// builds cross-platform sem quebrar (correcao P0: antes referenciava modulos
/// `platform::windows` incondicionalmente, quebrando o build Linux).
/// Testes unitarios injetam mocks diretamente nos consumidores (DI via traits).
pub struct ProviderFactory;

#[cfg(windows)]
mod real {
    use super::*;

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
    pub fn cleanup() -> Box<dyn CleanupProvider> {
        Box::new(crate::platform::windows::cleanup::WinCleanupProvider::new())
    }
}

#[cfg(not(windows))]
mod stub {
    use super::*;

    pub fn system_info() -> Box<dyn SystemInfoProvider> {
        Box::new(StubSys)
    }
    pub fn registry() -> Box<dyn RegistryProvider> {
        Box::new(StubReg)
    }
    pub fn services() -> Box<dyn ServiceProvider> {
        Box::new(StubSvc)
    }
    pub fn updates() -> Box<dyn UpdateProvider> {
        Box::new(StubUpd)
    }
    pub fn cleanup() -> Box<dyn CleanupProvider> {
        Box::new(StubClean)
    }

    fn not_supported() -> CoreError {
        CoreError::NotSupported("operacao indisponivel fora do Windows".to_string())
    }

    struct StubSys;
    impl SystemInfoProvider for StubSys {
        fn cpu(&self) -> Result<CpuInfo, CoreError> { Err(not_supported()) }
        fn memory(&self) -> Result<MemoryInfo, CoreError> { Err(not_supported()) }
        fn disks(&self) -> Result<Vec<DiskInfo>, CoreError> { Err(not_supported()) }
        fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError> { Err(not_supported()) }
        fn motherboard(&self) -> Result<MotherboardInfo, CoreError> { Err(not_supported()) }
        fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError> { Ok(vec![]) }
    }

    struct StubReg;
    impl RegistryProvider for StubReg {
        fn read_key(&self, _: &str, _: &str) -> Result<String, CoreError> { Err(not_supported()) }
        fn enum_values(&self, _: &str) -> Result<Vec<(String, String)>, CoreError> { Ok(vec![]) }
        fn delete_value(&self, _: &str, _: &str) -> Result<(), CoreError> { Err(not_supported()) }
        fn enum_subkeys(&self, _: &str) -> Result<Vec<String>, CoreError> { Ok(vec![]) }
    }

    struct StubSvc;
    impl ServiceProvider for StubSvc {
        fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError> { Ok(vec![]) }
        fn get_status(&self, _: &str) -> Result<String, CoreError> { Err(not_supported()) }
        fn set_start_type(&self, _: &str, _: &str) -> Result<(), CoreError> { Err(not_supported()) }
        fn stop(&self, _: &str) -> Result<(), CoreError> { Err(not_supported()) }
        fn start(&self, _: &str) -> Result<(), CoreError> { Err(not_supported()) }
    }

    struct StubUpd;
    impl UpdateProvider for StubUpd {
        fn search_pending(&self) -> Result<Vec<String>, CoreError> { Ok(vec![]) }
        fn install_updates(&self) -> Result<u32, CoreError> { Ok(0) }
        fn get_history(&self, _: u32) -> Result<Vec<String>, CoreError> { Ok(vec![]) }
    }

    struct StubClean;
    impl CleanupProvider for StubClean {
        fn clean_temp_files_detailed(&self, _: &crate::core::cleanup::CleanupPlan) -> Result<(u64, u64), CoreError> { Ok((0, 0)) }
        fn clean_recycle_bin_detailed(&self, _: bool) -> Result<(u64, u64), CoreError> { Ok((0, 0)) }
        fn clean_old_logs_detailed(&self, _: std::time::Duration) -> Result<(u64, u64), CoreError> { Ok((0, 0)) }
        fn clean_windows_update_cache(&self) -> Result<u64, CoreError> { Ok(0) }
        fn clean_browser_cache(&self) -> Result<u64, CoreError> { Ok(0) }
        fn run_dism(&self) -> Result<(), CoreError> { Err(not_supported()) }
        fn run_sfc(&self) -> Result<(), CoreError> { Err(not_supported()) }
    }
}

impl ProviderFactory {
    pub fn system_info() -> Box<dyn SystemInfoProvider> {
        #[cfg(windows)] { real::system_info() }
        #[cfg(not(windows))] { stub::system_info() }
    }
    pub fn registry() -> Box<dyn RegistryProvider> {
        #[cfg(windows)] { real::registry() }
        #[cfg(not(windows))] { stub::registry() }
    }
    pub fn services() -> Box<dyn ServiceProvider> {
        #[cfg(windows)] { real::services() }
        #[cfg(not(windows))] { stub::services() }
    }
    pub fn updates() -> Box<dyn UpdateProvider> {
        #[cfg(windows)] { real::updates() }
        #[cfg(not(windows))] { stub::updates() }
    }
    pub fn cleanup() -> Box<dyn CleanupProvider> {
        #[cfg(windows)] { real::cleanup() }
        #[cfg(not(windows))] { stub::cleanup() }
    }
}
