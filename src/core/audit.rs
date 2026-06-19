//! Fase 1: Auditoria completa do sistema

use crate::app::messages::*;
use crate::core::error::CoreError;
use crate::core::traits::{SystemInfoProvider, RegistryProvider, ServiceProvider};
use tokio::sync::mpsc::Sender;
use tracing::{info, warn};

pub struct Auditor<'a> {
    sys: &'a dyn SystemInfoProvider,
    reg: &'a dyn RegistryProvider,
    svc: &'a dyn ServiceProvider,
}

impl<'a> Auditor<'a> {
    pub fn new(sys: &'a dyn SystemInfoProvider, reg: &'a dyn RegistryProvider, svc: &'a dyn ServiceProvider) -> Self {
        Self { sys, reg, svc }
    }

    pub async fn run_full(&self, tx: &Sender<AppMsg>) -> Result<AuditData, CoreError> {
        let mut data = AuditData::default();

        self.report_progress(tx, AuditPhase::Hardware, "CPU", 0).await;
        data.cpu = Some(self.sys.cpu()?);
        self.report_progress(tx, AuditPhase::Hardware, "Memoria", 10).await;
        data.memory = Some(self.sys.memory()?);
        self.report_progress(tx, AuditPhase::Hardware, "Discos", 20).await;
        data.disks = self.sys.disks()?;
        self.report_progress(tx, AuditPhase::Hardware, "GPU", 30).await;
        data.gpus = self.sys.gpu()?;
        self.report_progress(tx, AuditPhase::Hardware, "Placa-mae", 35).await;
        data.motherboard = Some(self.sys.motherboard()?);
        self.report_progress(tx, AuditPhase::Hardware, "Temperaturas", 38).await;
        data.temperatures = self.sys.temperatures().unwrap_or_default();

        self.report_progress(tx, AuditPhase::Software, "Programas instalados", 40).await;
        data.software = self.collect_software()?;

        self.report_progress(tx, AuditPhase::Services, "Servicos do sistema", 60).await;
        data.services = self.svc.list_all()?;

        self.report_progress(tx, AuditPhase::Registry, "Chaves de inicializacao", 75).await;
        data.registry_run_keys = self.collect_run_keys()?;

        self.report_progress(tx, AuditPhase::Environment, "Variaveis de ambiente", 90).await;
        data.environment = self.collect_environment()?;

        self.report_progress(tx, AuditPhase::Hardware, "Concluido", 100).await;
        info!("Auditoria completa finalizada");
        Ok(data)
    }

    async fn report_progress(&self, tx: &Sender<AppMsg>, phase: AuditPhase, item: &str, percent: u8) {
        let _ = tx.send(AppMsg::AuditProgress { phase, item: item.to_string(), percent }).await;
    }

    fn collect_software(&self) -> Result<Vec<SoftwareInfo>, CoreError> {
        let mut software = Vec::new();
        let paths = [
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ];
        for path in &paths {
            match self.reg.enum_subkeys(path) {
                Ok(subkeys) => {
                    for subkey in subkeys {
                        let full_path = format!("{}\\{}", path, subkey);
                        if let Ok(name) = self.reg.read_key(&full_path, "DisplayName") {
                            software.push(SoftwareInfo {
                                display_name: name,
                                display_version: self.reg.read_key(&full_path, "DisplayVersion").unwrap_or_default(),
                                publisher: self.reg.read_key(&full_path, "Publisher").unwrap_or_default(),
                                install_date: self.reg.read_key(&full_path, "InstallDate").unwrap_or_default(),
                                install_location: self.reg.read_key(&full_path, "InstallLocation").unwrap_or_default(),
                            });
                        }
                    }
                }
                Err(e) => warn!("Nao foi possivel ler {}: {}", path, e),
            }
        }
        software.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(software)
    }

    fn collect_run_keys(&self) -> Result<Vec<RunKey>, CoreError> {
        let mut keys = Vec::new();
        let paths = [
            ("HKLM", r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
            ("HKLM", r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run"),
            ("HKCU", r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
        ];
        for (hive, path) in &paths {
            if let Ok(values) = self.reg.enum_values(path) {
                for (name, value) in values {
                    keys.push(RunKey { hive: hive.to_string(), name, value });
                }
            }
        }
        Ok(keys)
    }

    fn collect_environment(&self) -> Result<EnvironmentVars, CoreError> {
        let system: Vec<(String, String)> = std::env::vars().collect();
        let user: Vec<(String, String)> = std::env::vars().collect();
        Ok(EnvironmentVars { system, user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::{MockSystemInfoProvider, MockRegistryProvider, MockServiceProvider};

    #[tokio::test]
    async fn test_audit_full_flow() {
        let mut mock_sys = MockSystemInfoProvider::new();
        let mock_reg = MockRegistryProvider::new();
        let mock_svc = MockServiceProvider::new();

        mock_sys.expect_cpu().times(1).returning(|| Ok(CpuInfo {
            name: "Intel i7-9700K".to_string(), manufacturer: "Intel".to_string(),
            cores: 8, threads: 8, max_speed_mhz: 4900, architecture: "x64".to_string(), socket: "LGA1151".to_string(),
        }));
        mock_sys.expect_memory().times(1).returning(|| Ok(MemoryInfo {
            total_bytes: 17179869184,
            modules: vec![MemoryModule { slot: "DIMM1".to_string(), capacity_bytes: 17179869184, speed_mhz: 3200, manufacturer: "Corsair".to_string() }],
        }));
        mock_sys.expect_disks().times(1).returning(|| Ok(vec![]));
        mock_sys.expect_gpu().times(1).returning(|| Ok(vec![]));
        mock_sys.expect_motherboard().times(1).returning(|| Ok(MotherboardInfo::default()));
        mock_sys.expect_temperatures().times(1).returning(|| Ok(vec![]));

        let auditor = Auditor::new(&mock_sys, &mock_reg, &mock_svc);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let result = auditor.run_full(&tx).await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        assert!(matches!(msg, AppMsg::AuditProgress { phase: AuditPhase::Hardware, .. }));
    }
}
