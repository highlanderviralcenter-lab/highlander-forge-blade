//! Implementacao real de ServiceProvider

use crate::core::error::CoreError;
use crate::core::traits::ServiceProvider;
use crate::app::messages::ServiceInfo;

pub struct WinServiceProvider;

impl WinServiceProvider {
    pub fn new() -> Self { Self }
}

impl ServiceProvider for WinServiceProvider {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError> {
        let mut services = Vec::new();
        let output = std::process::Command::new("sc")
            .args(["query", "type=", "service", "state=", "all"])
            .output()
            .map_err(|e| CoreError::ServiceNotFound(e.to_string()))?;
        let text = String::from_utf8_lossy(&output.stdout);
        let mut current_name = String::new();
        let mut current_display = String::new();
        let mut current_state = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("SERVICE_NAME:") {
                if !current_name.is_empty() {
                    services.push(ServiceInfo {
                        name: current_name.clone(), display_name: current_display.clone(),
                        state: current_state.clone(), start_mode: "Desconhecido".to_string(),
                        is_third_party: false, path: String::new(),
                    });
                }
                current_name = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                current_display = String::new();
                current_state = String::new();
            } else if trimmed.starts_with("DISPLAY_NAME:") {
                current_display = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            } else if trimmed.starts_with("STATE") {
                if let Some(idx) = trimmed.find("4  RUNNING") { current_state = "Running".to_string(); }
                else if let Some(idx) = trimmed.find("1  STOPPED") { current_state = "Stopped".to_string(); }
                else { current_state = "Unknown".to_string(); }
            }
        }
        if !current_name.is_empty() {
            services.push(ServiceInfo {
                name: current_name, display_name: current_display,
                state: current_state, start_mode: "Desconhecido".to_string(),
                is_third_party: false, path: String::new(),
            });
        }
        Ok(services)
    }

    fn get_status(&self, name: &str) -> Result<String, CoreError> {
        let output = std::process::Command::new("sc").args(["query", name]).output()
            .map_err(|e| CoreError::ServiceNotFound(e.to_string()))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn set_start_type(&self, name: &str, start_type: &str) -> Result<(), CoreError> {
        let mode = match start_type {
            "auto" => "auto",
            "manual" => "demand",
            "disabled" => "disabled",
            _ => return Err(CoreError::NotSupported(format!("Start type invalido: {}", start_type))),
        };
        let output = std::process::Command::new("sc").args(["config", name, "start=", mode]).output()
            .map_err(|e| CoreError::ServiceNotFound(e.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::ServiceNotFound(format!("Falha ao alterar {}: {}", name, String::from_utf8_lossy(&output.stderr))));
        }
        Ok(())
    }

    fn stop(&self, name: &str) -> Result<(), CoreError> {
        let output = std::process::Command::new("sc").args(["stop", name]).output()
            .map_err(|e| CoreError::ServiceNotFound(e.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::ServiceNotFound(format!("Falha ao parar {}: {}", name, String::from_utf8_lossy(&output.stderr))));
        }
        Ok(())
    }

    fn start(&self, name: &str) -> Result<(), CoreError> {
        let output = std::process::Command::new("sc").args(["start", name]).output()
            .map_err(|e| CoreError::ServiceNotFound(e.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::ServiceNotFound(format!("Falha ao iniciar {}: {}", name, String::from_utf8_lossy(&output.stderr))));
        }
        Ok(())
    }
}
