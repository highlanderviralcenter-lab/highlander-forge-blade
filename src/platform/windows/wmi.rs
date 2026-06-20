//! Implementacao real de SystemInfoProvider via WMI (PowerShell fallback + COM futuro)

use crate::app::messages::*;
use crate::core::error::CoreError;
use crate::core::traits::SystemInfoProvider;

pub struct WmiSystemInfoProvider;

impl WmiSystemInfoProvider {
    pub fn new() -> Self { Self }

    fn run_ps(&self, script: &str) -> Result<String, CoreError> {
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .output()
            .map_err(|e| CoreError::WmiUnavailable(e.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::WmiUnavailable(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl SystemInfoProvider for WmiSystemInfoProvider {
    fn cpu(&self) -> Result<CpuInfo, CoreError> {
        let name = self.run_ps("(Get-WmiObject Win32_Processor).Name").unwrap_or_else(|_| "Desconhecido".to_string());
        let cores: u32 = self.run_ps("(Get-WmiObject Win32_Processor).NumberOfCores").unwrap_or_default().parse().unwrap_or(0);
        let threads: u32 = self.run_ps("(Get-WmiObject Win32_Processor).NumberOfLogicalProcessors").unwrap_or_default().parse().unwrap_or(0);
        let max_speed: u32 = self.run_ps("(Get-WmiObject Win32_Processor).MaxClockSpeed").unwrap_or_default().parse().unwrap_or(0);
        Ok(CpuInfo {
            name: name.trim().to_string(),
            manufacturer: self.run_ps("(Get-WmiObject Win32_Processor).Manufacturer").unwrap_or_default().trim().to_string(),
            cores, threads, max_speed_mhz: max_speed,
            architecture: self.run_ps("(Get-WmiObject Win32_Processor).Architecture").unwrap_or_default().trim().to_string(),
            socket: self.run_ps("(Get-WmiObject Win32_Processor).SocketDesignation").unwrap_or_default().trim().to_string(),
        })
    }

    fn memory(&self) -> Result<MemoryInfo, CoreError> {
        let total: u64 = self.run_ps("(Get-WmiObject Win32_ComputerSystem).TotalPhysicalMemory").unwrap_or_default().parse().unwrap_or(0);
        let mut modules = Vec::new();
        let ps_modules = self.run_ps("Get-WmiObject Win32_PhysicalMemory | Select-Object DeviceLocator, Capacity, Speed, Manufacturer | ConvertTo-Json").unwrap_or_default();
        if !ps_modules.is_empty() && ps_modules != "null" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&ps_modules) {
                if let Some(arr) = json.as_array() {
                    for m in arr {
                        modules.push(MemoryModule {
                            slot: m.get("DeviceLocator").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                            capacity_bytes: m.get("Capacity").and_then(|v| v.as_u64()).unwrap_or(0),
                            speed_mhz: m.get("Speed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            manufacturer: m.get("Manufacturer").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        });
                    }
                } else if let Some(obj) = json.as_object() {
                    modules.push(MemoryModule {
                        slot: obj.get("DeviceLocator").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        capacity_bytes: obj.get("Capacity").and_then(|v| v.as_u64()).unwrap_or(0),
                        speed_mhz: obj.get("Speed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        manufacturer: obj.get("Manufacturer").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                    });
                }
            }
        }
        Ok(MemoryInfo { total_bytes: total, modules })
    }

    fn disks(&self) -> Result<Vec<DiskInfo>, CoreError> {
        let mut disks = Vec::new();
        let ps_disks = self.run_ps("Get-WmiObject Win32_LogicalDisk | Select-Object DeviceID, VolumeName, FileSystem, Size, FreeSpace | ConvertTo-Json").unwrap_or_default();
        if !ps_disks.is_empty() && ps_disks != "null" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&ps_disks) {
                if let Some(arr) = json.as_array() {
                    for d in arr {
                        let total = d.get("Size").and_then(|v| v.as_u64()).unwrap_or(0);
                        let free = d.get("FreeSpace").and_then(|v| v.as_u64()).unwrap_or(0);
                        disks.push(DiskInfo {
                            device_id: d.get("DeviceID").and_then(|v| v.as_str()).unwrap_or("C:").to_string(),
                            volume_name: d.get("VolumeName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            filesystem: d.get("FileSystem").and_then(|v| v.as_str()).unwrap_or("NTFS").to_string(),
                            total_bytes: total,
                            free_bytes: free,
                            used_bytes: total.saturating_sub(free),
                            percent_free: if total > 0 { (free as f64 / total as f64) * 100.0 } else { 0.0 },
                        });
                    }
                }
            }
        }
        Ok(disks)
    }

    fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError> {
        let mut gpus = Vec::new();
        let ps_gpus = self.run_ps("Get-WmiObject Win32_VideoController | Select-Object Name, AdapterRAM, VideoModeDescription, DriverVersion | ConvertTo-Json").unwrap_or_default();
        if !ps_gpus.is_empty() && ps_gpus != "null" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&ps_gpus) {
                if let Some(arr) = json.as_array() {
                    for g in arr {
                        gpus.push(GpuInfo {
                            name: g.get("Name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                            manufacturer: "Desconhecido".to_string(),
                            adapter_ram_bytes: g.get("AdapterRAM").and_then(|v| v.as_u64()).unwrap_or(0),
                            resolution: g.get("VideoModeDescription").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            driver_version: g.get("DriverVersion").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        });
                    }
                }
            }
        }
        Ok(gpus)
    }

    fn motherboard(&self) -> Result<MotherboardInfo, CoreError> {
        Ok(MotherboardInfo {
            manufacturer: self.run_ps("(Get-WmiObject Win32_BaseBoard).Manufacturer").unwrap_or_default().trim().to_string(),
            product: self.run_ps("(Get-WmiObject Win32_BaseBoard).Product").unwrap_or_default().trim().to_string(),
            version: self.run_ps("(Get-WmiObject Win32_BaseBoard).Version").unwrap_or_default().trim().to_string(),
            serial_number: self.run_ps("(Get-WmiObject Win32_BaseBoard).SerialNumber").unwrap_or_default().trim().to_string(),
            bios_vendor: self.run_ps("(Get-WmiObject Win32_BIOS).Manufacturer").unwrap_or_default().trim().to_string(),
            bios_version: self.run_ps("(Get-WmiObject Win32_BIOS).Version").unwrap_or_default().trim().to_string(),
            bios_date: self.run_ps("(Get-WmiObject Win32_BIOS).ReleaseDate").unwrap_or_default().trim().to_string(),
        })
    }

    fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError> {
        // MSFT_WmiProvider classes são voláteis; retorna vazio por seguranca
        Ok(Vec::new())
    }
}
