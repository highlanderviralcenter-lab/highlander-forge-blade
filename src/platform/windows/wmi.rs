//! Implementacao WMI real — usa windows-rs

use crate::app::messages::*;
use crate::core::error::CoreError;
use crate::core::traits::SystemInfoProvider;

pub struct WmiSystemInfoProvider;

impl WmiSystemInfoProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemInfoProvider for WmiSystemInfoProvider {
    fn cpu(&self) -> Result<CpuInfo, CoreError> {
        // TODO: Implementar via WMI (Win32_Processor)
        // Stub para compilacao
        Ok(CpuInfo {
            name: "Desconhecido".to_string(),
            manufacturer: "Desconhecido".to_string(),
            cores: 0,
            threads: 0,
            max_speed_mhz: 0,
            architecture: "x64".to_string(),
            socket: "Desconhecido".to_string(),
        })
    }

    fn memory(&self) -> Result<MemoryInfo, CoreError> {
        Ok(MemoryInfo::default())
    }

    fn disks(&self) -> Result<Vec<DiskInfo>, CoreError> {
        Ok(Vec::new())
    }

    fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError> {
        Ok(Vec::new())
    }

    fn motherboard(&self) -> Result<MotherboardInfo, CoreError> {
        Ok(MotherboardInfo::default())
    }

    fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError> {
        Ok(Vec::new())
    }
}
