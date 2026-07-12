//! Fase 1: Auditoria completa via WMI/PowerShell

use crate::app::messages::Message;
use crate::core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::sync::mpsc::Sender;

// ── Estruturas de dados ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuData {
    pub name: String,
    pub cores: u32,
    pub threads: u32,
    pub frequency: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryData {
    pub total: String,
    pub total_bytes: u64,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskData {
    pub model: String,
    pub disk_type: String,
    pub size: String,
    pub free_percent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuData {
    pub name: String,
    pub driver: String,
    pub vram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MotherboardData {
    pub model: String,
    pub manufacturer: String,
    pub bios_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemperaturesData {
    pub cpu_package: Option<f64>,
    pub cpu_cores: Option<f64>,
    pub gpu: Option<f64>,
    pub ssd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerData {
    pub package: Option<f64>,
    pub ia_cores: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClocksData {
    pub bclk: Option<f64>,
    pub core_max: Option<u32>,
    pub memory: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditData {
    pub machine_id: String,
    pub timestamp: String,
    pub cpu: CpuData,
    pub memory: MemoryData,
    pub disks: Vec<DiskData>,
    pub gpu: GpuData,
    pub motherboard: MotherboardData,
    pub temperatures: TemperaturesData,
    pub power: PowerData,
    pub clocks: ClocksData,
}

// ── Auditor ─────────────────────────────────────────────────

pub struct Auditor;

impl Auditor {
    pub fn new() -> Self { Self }

    fn run_ps(script: &str) -> String {
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    pub async fn run_full_audit(&self, tx: Sender<Message>) -> Result<AuditData, CoreError> {
        let mut data = AuditData::default();

        // machine_id
        data.machine_id = crate::app::machine_id::get_or_create_machine_id()
            .unwrap_or_else(|_| "unknown".to_string());
        data.timestamp = chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string();

        // ── CPU ──────────────────────────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 5,
            log: "Coletando informacoes do processador...".to_string(),
        }).await;

        let cpu_json = Self::run_ps(r#"
$c = Get-CimInstance Win32_Processor | Select -First 1
[PSCustomObject]@{
    name    = $c.Name.Trim()
    cores   = [int]$c.NumberOfCores
    threads = [int]$c.NumberOfLogicalProcessors
    speed   = [int]$c.MaxClockSpeed
    arch    = if($c.Architecture -eq 9){'x64'}else{'x86'}
} | ConvertTo-Json -Compress
"#);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&cpu_json) {
            data.cpu = CpuData {
                name: v["name"].as_str().unwrap_or("").to_string(),
                cores: v["cores"].as_u64().unwrap_or(0) as u32,
                threads: v["threads"].as_u64().unwrap_or(0) as u32,
                frequency: format!("{} MHz", v["speed"].as_u64().unwrap_or(0)),
                architecture: v["arch"].as_str().unwrap_or("x64").to_string(),
            };
        }
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 20,
            log: format!("CPU: {}", data.cpu.name),
        }).await;

        // ── RAM ──────────────────────────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 25,
            log: "Coletando informacoes de memoria...".to_string(),
        }).await;

        let ram_json = Self::run_ps(r#"
$os  = Get-CimInstance Win32_OperatingSystem
$mods = @(Get-CimInstance Win32_PhysicalMemory | ForEach-Object {
    "$([math]::Round($_.Capacity/1GB,0))GB $($_.Speed)MHz - $($_.Manufacturer)"
})
[PSCustomObject]@{
    total_kb = [long]$os.TotalVisibleMemorySize
    modules  = $mods
} | ConvertTo-Json -Depth 3 -Compress
"#);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&ram_json) {
            let total_kb = v["total_kb"].as_u64().unwrap_or(0);
            let total_gb = total_kb as f64 / (1024.0 * 1024.0);
            let mods: Vec<String> = v["modules"].as_array()
                .map(|a| a.iter().map(|m| m.as_str().unwrap_or("").to_string()).collect())
                .unwrap_or_default();
            data.memory = MemoryData {
                total: format!("{:.1} GB", total_gb),
                total_bytes: total_kb * 1024,
                modules: mods,
            };
        }
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 40,
            log: format!("RAM: {}", data.memory.total),
        }).await;

        // ── DISCOS ───────────────────────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 45,
            log: "Coletando informacoes de disco...".to_string(),
        }).await;

        // Detecta tipo via modelo do disco fisico
        let disk_types_raw = Self::run_ps(r#"
$types = @{}
Get-CimInstance Win32_DiskDrive | ForEach-Object {
    $m = $_.Model.ToUpper()
    $t = if($m -match 'NVME|NVM') {'NVMe'}
         elseif($m -match 'SSD|SOLID') {'SSD'}
         else {'HDD'}
    # Pega letra do volume associado
    $assoc = Get-CimAssociatedInstance -InputObject $_ -Association Win32_DiskDriveToDiskPartition |
             ForEach-Object { Get-CimAssociatedInstance -InputObject $_ -Association Win32_LogicalDiskToPartition } |
             Select -ExpandProperty DeviceID -ErrorAction SilentlyContinue
    foreach($letter in $assoc) { $types[$letter] = $t }
}
$types | ConvertTo-Json -Compress
"#);
        let disk_types: serde_json::Value = serde_json::from_str(&disk_types_raw)
            .unwrap_or(serde_json::json!({}));

        let disks_raw = Self::run_ps(r#"
@(Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object {
    $free_pct = if($_.Size -gt 0){ [math]::Round(($_.FreeSpace/$_.Size)*100,1) } else { 0 }
    [PSCustomObject]@{
        id    = $_.DeviceID
        size  = "$([math]::Round($_.Size/1GB,0))GB"
        free  = "$($free_pct)%"
        vol   = $_.VolumeName
    }
}) | ConvertTo-Json -Depth 2 -Compress
"#);
        let disks_val: serde_json::Value = if disks_raw.trim_start().starts_with('[') {
            serde_json::from_str(&disks_raw).unwrap_or(serde_json::json!([]))
        } else if disks_raw.trim_start().starts_with('{') {
            serde_json::json!([serde_json::from_str::<serde_json::Value>(&disks_raw).unwrap_or_default()])
        } else {
            serde_json::json!([])
        };

        if let Some(arr) = disks_val.as_array() {
            for d in arr {
                let id = d["id"].as_str().unwrap_or("").to_string();
                let dtype = disk_types.get(&id)
                    .and_then(|v| v.as_str())
                    .unwrap_or("HDD")
                    .to_string();
                data.disks.push(DiskData {
                    model: format!("{}", id),
                    disk_type: dtype,
                    size: d["size"].as_str().unwrap_or("").to_string(),
                    free_percent: d["free"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 60,
            log: format!("{} disco(s) encontrado(s)", data.disks.len()),
        }).await;

        // ── GPU ──────────────────────────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 65,
            log: "Coletando informacoes da GPU...".to_string(),
        }).await;

        let gpu_raw = Self::run_ps(r#"
$g = Get-CimInstance Win32_VideoController | Select -First 1
[PSCustomObject]@{
    name   = $g.Name
    driver = $g.DriverVersion
    vram   = "$([math]::Round($g.AdapterRAM/1GB,0))GB"
} | ConvertTo-Json -Compress
"#);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&gpu_raw) {
            data.gpu = GpuData {
                name: v["name"].as_str().unwrap_or("").to_string(),
                driver: v["driver"].as_str().unwrap_or("").to_string(),
                vram: v["vram"].as_str().unwrap_or("").to_string(),
            };
        }
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 75,
            log: format!("GPU: {}", data.gpu.name),
        }).await;

        // ── PLACA-MAE ─────────────────────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 80,
            log: "Coletando informacoes da placa-mae...".to_string(),
        }).await;

        let mobo_raw = Self::run_ps(r#"
$mb   = Get-CimInstance Win32_BaseBoard | Select -First 1
$bios = Get-CimInstance Win32_BIOS | Select -First 1
[PSCustomObject]@{
    model    = "$($mb.Manufacturer) $($mb.Product)"
    mfr      = $mb.Manufacturer
    bios_ver = $bios.SMBIOSBIOSVersion
} | ConvertTo-Json -Compress
"#);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&mobo_raw) {
            data.motherboard = MotherboardData {
                model: v["model"].as_str().unwrap_or("").to_string(),
                manufacturer: v["mfr"].as_str().unwrap_or("").to_string(),
                bios_version: v["bios_ver"].as_str().unwrap_or("").to_string(),
            };
        }
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 90,
            log: format!("Placa-mae: {}", data.motherboard.model),
        }).await;

        // ── TEMPERATURAS (opcional) ───────────────────────────
        let _ = tx.send(Message::AuditProgress {
            phase: "Hardware".to_string(),
            percent: 95,
            log: "Tentando coletar temperaturas...".to_string(),
        }).await;

        let temp_raw = Self::run_ps(r#"
try {
    $temps = Get-CimInstance -Namespace root/WMI MSAcpi_ThermalZoneTemperature -EA Stop
    $first = ($temps | Select -First 1).CurrentTemperature
    $c = [math]::Round($first/10.0 - 273.15, 1)
    "{`"cpu`": $c}"
} catch { '{}' }
"#);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&temp_raw) {
            if let Some(t) = v["cpu"].as_f64() {
                data.temperatures.cpu_package = Some(t);
            }
        }

        let _ = tx.send(Message::AuditProgress {
            phase: "Concluindo".to_string(),
            percent: 100,
            log: "Auditoria concluida!".to_string(),
        }).await;

        Ok(data)
    }
}
