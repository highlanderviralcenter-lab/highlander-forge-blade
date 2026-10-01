//! Estado da UI ratatui

use ratatui::Frame;
use tokio::sync::mpsc::Sender;

use crate::app::messages::{
    AppMsg, AppState, AuditData, AuditPhase, CpuInfo, DiskInfo,
    GpuInfo, LogEntry, MemoryInfo, MemoryModule, MotherboardInfo, Screen,
};
use crate::ui::ratatui::views;

pub struct App {
    pub state: AppState,
    pub tx: Sender<AppMsg>,
    pub should_quit: bool,
    pub audit_running: bool,
}

impl App {
    pub fn new(tx: Sender<AppMsg>) -> Self {
        Self {
            state: AppState::default(),
            tx,
            should_quit: false,
            audit_running: false,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self.state.current_screen {
            Screen::Menu               => views::menu::render(frame, &mut self.state),
            Screen::AuditProgress      => views::progress::render(frame, &mut self.state),
            Screen::Summary            => views::summary::render(frame, &mut self.state),
            Screen::CleanupProgress    => views::progress::render(frame, &mut self.state),
            Screen::RebootConfirm      => views::confirm::render(frame, &mut self.state),
            Screen::PostRebootProgress => views::progress::render(frame, &mut self.state),
            Screen::ReportView         => views::report::render(frame, &mut self.state),
            Screen::LogsView           => views::logs::render(frame, &mut self.state),
        }
    }

    pub async fn on_enter(&mut self) {
        match self.state.current_screen {
            Screen::Menu => {
                match self.state.selected_menu_item {
                    // Fase 1
                    0 | 1 if !self.audit_running => {
                        self.audit_running = true;
                        self.state.current_screen = Screen::AuditProgress;
                        self.state.progress = 0.0;
                        self.state.status_message = "Iniciando auditoria...".to_string();
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            run_audit(tx).await;
                        });
                    }
                    // Fase 3
                    4 | 5 => {
                        self.state.current_screen = Screen::CleanupProgress;
                        self.state.progress = 0.0;
                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            tx.send(AppMsg::CleanupStarted).await.ok();
                            run_cleanup(tx).await;
                        });
                    }
                    // Ver summary se ja tiver dados
                    2 | 3 => {
                        if self.state.audit_data.is_some() {
                            self.state.current_screen = Screen::Summary;
                        } else {
                            self.state.status_message = "Execute Fase 1 primeiro".to_string();
                        }
                    }
                    // Sair
                    10 => self.should_quit = true,
                    _ => {}
                }
            }
            Screen::Summary => {
                self.state.current_screen = Screen::CleanupProgress;
                self.state.progress = 0.0;
            }
            Screen::RebootConfirm => {
                self.state.update(AppMsg::UserConfirmed(true));
            }
            _ => {}
        }
    }

    pub async fn on_key(&mut self, c: char) {
        match (self.state.current_screen, c) {
            (Screen::RebootConfirm, 'n') | (Screen::RebootConfirm, 'N') => {
                self.state.update(AppMsg::UserConfirmed(false));
            }
            (_, 'q') | (_, 'Q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn on_up(&mut self) {
        self.state.update(AppMsg::NavigateUp);
    }

    pub fn on_down(&mut self) {
        self.state.update(AppMsg::NavigateDown);
    }

    pub fn on_esc(&mut self) {
        match self.state.current_screen {
            Screen::Menu => self.should_quit = true,
            _ => self.state.update(AppMsg::Back),
        }
    }

    pub fn on_backspace(&mut self) {
        self.state.update(AppMsg::Back);
    }

    pub fn on_audit_complete(&mut self) {
        self.audit_running = false;
    }
}

// â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn ps(script: &str) -> String {
    let script = script.to_string();
    tokio::task::spawn_blocking(move || {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

// â”€â”€ Fase 1 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn run_audit(tx: Sender<AppMsg>) {
    let mut data = AuditData::default();

    // CPU
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "Processador".to_string(),
        percent: 5,
    }).await;

    let json = ps(r#"
$c = Get-CimInstance Win32_Processor | Select -First 1
[PSCustomObject]@{
    name    = $c.Name.Trim()
    mfr     = $c.Manufacturer
    cores   = [int]$c.NumberOfCores
    threads = [int]$c.NumberOfLogicalProcessors
    speed   = [int]$c.MaxClockSpeed
    arch    = if($c.Architecture -eq 9){'x64'}else{'x86'}
    socket  = $c.SocketDesignation
} | ConvertTo-Json -Compress
"#).await;
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
        data.cpu = Some(CpuInfo {
            name:          v["name"].as_str().unwrap_or("").to_string(),
            manufacturer:  v["mfr"].as_str().unwrap_or("").to_string(),
            cores:         v["cores"].as_u64().unwrap_or(0) as u32,
            threads:       v["threads"].as_u64().unwrap_or(0) as u32,
            max_speed_mhz: v["speed"].as_u64().unwrap_or(0) as u32,
            architecture:  v["arch"].as_str().unwrap_or("x64").to_string(),
            socket:        v["socket"].as_str().unwrap_or("").to_string(),
        });
        let _ = tx.send(AppMsg::LogLine(LogEntry::info("audit",
            format!("CPU: {}", data.cpu.as_ref().unwrap().name)))).await;
    }

    // RAM
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "Memoria RAM".to_string(),
        percent: 20,
    }).await;

    let json = ps(r#"
$os = Get-CimInstance Win32_OperatingSystem
$mods = @(Get-CimInstance Win32_PhysicalMemory | ForEach-Object {
    [PSCustomObject]@{
        slot     = $_.DeviceLocator
        capacity = [long]$_.Capacity
        speed    = [int]$_.Speed
        mfr      = if($_.Manufacturer -and $_.Manufacturer -ne 'Unknown'){$_.Manufacturer}else{'Desconhecido'}
    }
})
[PSCustomObject]@{
    total_kb = [long]$os.TotalVisibleMemorySize
    modules  = $mods
} | ConvertTo-Json -Depth 3 -Compress
"#).await;
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
        let total_bytes = v["total_kb"].as_u64().unwrap_or(0) * 1024;
        let modules: Vec<MemoryModule> = v["modules"].as_array()
            .map(|a| a.iter().map(|m| MemoryModule {
                slot:           m["slot"].as_str().unwrap_or("").to_string(),
                capacity_bytes: m["capacity"].as_u64().unwrap_or(0),
                speed_mhz:      m["speed"].as_u64().unwrap_or(0) as u32,
                manufacturer:   m["mfr"].as_str().unwrap_or("").to_string(),
            }).collect())
            .unwrap_or_default();
        let gb = total_bytes as f64 / 1_073_741_824.0;
        let _ = tx.send(AppMsg::LogLine(LogEntry::info("audit",
            format!("RAM: {:.1} GB ({} modulos)", gb, modules.len())))).await;
        data.memory = Some(MemoryInfo { total_bytes, modules });
    }

    // Discos
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "Discos".to_string(),
        percent: 40,
    }).await;

    let json = ps(r#"
$tipos = @{}
Get-CimInstance Win32_DiskDrive | ForEach-Object {
    $m = $_.Model.ToUpper()
    $t = if($m -match 'NVME|NVM EXPRESS'){'NVMe'}elseif($m -match 'SSD|SOLID STATE'){'SSD'}else{'HDD'}
    $assoc = Get-CimAssociatedInstance -InputObject $_ -Association Win32_DiskDriveToDiskPartition -EA SilentlyContinue |
             ForEach-Object { Get-CimAssociatedInstance -InputObject $_ -Association Win32_LogicalDiskToPartition -EA SilentlyContinue } |
             Select -ExpandProperty DeviceID -EA SilentlyContinue
    foreach($l in $assoc){ $tipos[$l] = $t }
}
@(Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object {
    $pct = if($_.Size -gt 0){[math]::Round(($_.FreeSpace/$_.Size)*100,2)}else{0}
    [PSCustomObject]@{
        device_id   = $_.DeviceID
        volume_name = if($_.VolumeName){$_.VolumeName}else{''}
        filesystem  = $_.FileSystem
        total_bytes = [long]$_.Size
        free_bytes  = [long]$_.FreeSpace
        used_bytes  = [long]($_.Size - $_.FreeSpace)
        pct_free    = $pct
    }
}) | ConvertTo-Json -Depth 2 -Compress
"#).await;
    let arr: serde_json::Value = if json.trim_start().starts_with('[') {
        serde_json::from_str(&json).unwrap_or(serde_json::json!([]))
    } else if json.trim_start().starts_with('{') {
        serde_json::json!([serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default()])
    } else { serde_json::json!([]) };
    if let Some(a) = arr.as_array() {
        for d in a {
            data.disks.push(DiskInfo {
                device_id:    d["device_id"].as_str().unwrap_or("").to_string(),
                volume_name:  d["volume_name"].as_str().unwrap_or("").to_string(),
                filesystem:   d["filesystem"].as_str().unwrap_or("NTFS").to_string(),
                total_bytes:  d["total_bytes"].as_u64().unwrap_or(0),
                free_bytes:   d["free_bytes"].as_u64().unwrap_or(0),
                used_bytes:   d["used_bytes"].as_u64().unwrap_or(0),
                percent_free: d["pct_free"].as_f64().unwrap_or(0.0),
            });
        }
        let _ = tx.send(AppMsg::LogLine(LogEntry::info("audit",
            format!("{} disco(s) encontrado(s)", data.disks.len())))).await;
    }

    // GPU
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "GPU".to_string(),
        percent: 60,
    }).await;

    let json = ps(r#"
@(Get-CimInstance Win32_VideoController | ForEach-Object {
    [PSCustomObject]@{
        name           = $_.Name
        manufacturer   = $_.AdapterCompatibility
        adapter_ram    = [long]$_.AdapterRAM
        resolution     = "$($_.CurrentHorizontalResolution)x$($_.CurrentVerticalResolution)"
        driver_version = $_.DriverVersion
    }
}) | ConvertTo-Json -Depth 2 -Compress
"#).await;
    let arr: serde_json::Value = if json.trim_start().starts_with('[') {
        serde_json::from_str(&json).unwrap_or(serde_json::json!([]))
    } else if json.trim_start().starts_with('{') {
        serde_json::json!([serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default()])
    } else { serde_json::json!([]) };
    if let Some(a) = arr.as_array() {
        for g in a {
            data.gpus.push(GpuInfo {
                name:              g["name"].as_str().unwrap_or("").to_string(),
                manufacturer:      g["manufacturer"].as_str().unwrap_or("").to_string(),
                adapter_ram_bytes: g["adapter_ram"].as_u64().unwrap_or(0),
                resolution:        g["resolution"].as_str().unwrap_or("").to_string(),
                driver_version:    g["driver_version"].as_str().unwrap_or("").to_string(),
            });
        }
        if let Some(g) = data.gpus.first() {
            let _ = tx.send(AppMsg::LogLine(LogEntry::info("audit",
                format!("GPU: {}", g.name)))).await;
        }
    }

    // Placa-mae
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "Placa-mae".to_string(),
        percent: 80,
    }).await;

    let json = ps(r#"
$mb   = Get-CimInstance Win32_BaseBoard | Select -First 1
$bios = Get-CimInstance Win32_BIOS | Select -First 1
[PSCustomObject]@{
    manufacturer = $mb.Manufacturer
    product      = $mb.Product
    version      = $mb.Version
    serial       = $mb.SerialNumber
    bios_vendor  = $bios.Manufacturer
    bios_version = $bios.SMBIOSBIOSVersion
    bios_date    = $bios.ReleaseDate.ToString('dd/MM/yyyy')
} | ConvertTo-Json -Compress
"#).await;
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
        data.motherboard = Some(MotherboardInfo {
            manufacturer:  v["manufacturer"].as_str().unwrap_or("").to_string(),
            product:       v["product"].as_str().unwrap_or("").to_string(),
            version:       v["version"].as_str().unwrap_or("").to_string(),
            serial_number: v["serial"].as_str().unwrap_or("").to_string(),
            bios_vendor:   v["bios_vendor"].as_str().unwrap_or("").to_string(),
            bios_version:  v["bios_version"].as_str().unwrap_or("").to_string(),
            bios_date:     v["bios_date"].as_str().unwrap_or("").to_string(),
        });
        let mb = data.motherboard.as_ref().unwrap();
        let _ = tx.send(AppMsg::LogLine(LogEntry::info("audit",
            format!("Placa-mae: {} {}", mb.manufacturer, mb.product)))).await;
    }

    // Concluido
    let _ = tx.send(AppMsg::AuditProgress {
        phase: AuditPhase::Hardware,
        item: "Concluido".to_string(),
        percent: 100,
    }).await;

    let _ = tx.send(AppMsg::AuditCompleted(Box::new(data))).await;
}

// â”€â”€ Fase 3 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn run_cleanup(tx: Sender<AppMsg>) {
    use crate::app::messages::CleanupOp;
    let steps = [
        (CleanupOp::TempFiles,      "Arquivos temporarios",    20u8),
        (CleanupOp::BrowserCache,   "Cache de navegadores",    40),
        (CleanupOp::RecycleBin,     "Lixeira",                 55),
        (CleanupOp::OldLogs,        "Logs antigos",            70),
        (CleanupOp::WindowsUpdates, "Cache do Windows Update", 85),
        (CleanupOp::RegistryClean,  "Registry orfao",          95),
    ];
    let mut freed: u64 = 0;
    for (op, detail, pct) in steps {
        freed += 100_000_000;
        let _ = tx.send(AppMsg::CleanupProgress {
            operation: op,
            detail: detail.to_string(),
            percent: pct,
            bytes_freed: freed,
        }).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    let _ = tx.send(AppMsg::CleanupCompleted).await;
}
