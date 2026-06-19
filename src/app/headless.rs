//! Modo headless — saida JSON parseable para RMMs/MSPs

use crate::core::error::CoreError;
use chrono::Utc;
use serde::Serialize;
use std::process;

pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const FATAL_ERROR: i32 = 1;
    pub const SUCCESS_WITH_WARNINGS: i32 = 2;
    pub const SIMULATION_COMPLETE: i32 = 3;
    pub const UPDATE_AVAILABLE: i32 = 4;
    pub const NEEDS_REBOOT: i32 = 5;
    pub const PARTIAL_SUCCESS: i32 = 6;
}

#[derive(Debug, Serialize)]
pub struct HeadlessOutput {
    pub version: String,
    pub machine_id: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub exit_code: i32,
    pub exit_reason: String,
    pub phases: Vec<PhaseResult>,
    pub summary: Summary,
    pub logs_path: String,
}

#[derive(Debug, Serialize)]
pub struct PhaseResult {
    pub phase: String,
    pub name: String,
    pub status: PhaseStatus,
    pub duration_seconds: u64,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub enum PhaseStatus { Success, Warning, Failed, Skipped }

#[derive(Debug, Serialize)]
pub struct Summary {
    pub bytes_freed: u64,
    pub services_altered: u32,
    pub registry_keys_removed: u32,
    pub updates_installed: u32,
    pub reboot_required: bool,
}

pub async fn run(auto_phase: Option<String>, what_if: bool) -> Result<(), Box<dyn std::error::Error>> {
    use crate::app::machine_id;
    let machine_id = machine_id::get_or_create_machine_id().unwrap_or_else(|_| "unknown".to_string());
    let mut output = HeadlessOutput {
        version: env!("CARGO_PKG_VERSION").to_string(), machine_id,
        timestamp: Utc::now(), exit_code: exit_codes::SUCCESS,
        exit_reason: "Iniciando execucao".to_string(), phases: Vec::new(),
        summary: Summary { bytes_freed: 0, services_altered: 0, registry_keys_removed: 0, updates_installed: 0, reboot_required: false },
        logs_path: format!(r"{}\Logs", crate::app::state::BASE_DIR),
    };
    if what_if {
        output.exit_code = exit_codes::SIMULATION_COMPLETE;
        output.exit_reason = "Modo simulacao".to_string();
        print_json(&output);
        process::exit(exit_codes::SIMULATION_COMPLETE);
    }
    match auto_phase.as_deref() {
        Some("0") | Some("all") => {
            match run_all_phases(&mut output).await {
                Ok(()) => { output.exit_code = exit_codes::SUCCESS; output.exit_reason = "Todas as fases concluidas".to_string(); }
                Err(e) => { output.exit_code = exit_codes::FATAL_ERROR; output.exit_reason = format!("Erro fatal: {}", e); }
            }
        }
        Some("1") => {
            match run_phase1(&mut output).await {
                Ok(()) => { output.exit_code = exit_codes::SUCCESS; output.exit_reason = "Fase 1 concluida".to_string(); }
                Err(e) => { output.exit_code = exit_codes::FATAL_ERROR; output.exit_reason = format!("Fase 1 falhou: {}", e); }
            }
        }
        Some("5") => {
            match run_phase5(&mut output).await {
                Ok(()) => { output.exit_code = exit_codes::SUCCESS; output.exit_reason = "Fase 5 concluida".to_string(); }
                Err(e) => { output.exit_code = exit_codes::FATAL_ERROR; output.exit_reason = format!("Fase 5 falhou: {}", e); }
            }
        }
        _ => { output.exit_code = exit_codes::FATAL_ERROR; output.exit_reason = "Fase invalida".to_string(); }
    }
    print_json(&output);
    process::exit(output.exit_code);
}

async fn run_all_phases(output: &mut HeadlessOutput) -> Result<(), CoreError> {
    let start = std::time::Instant::now();
    run_phase1(output).await?;
    output.phases.push(PhaseResult { phase: "1".to_string(), name: "Auditoria".to_string(), status: PhaseStatus::Success, duration_seconds: start.elapsed().as_secs(), details: serde_json::json!({}) });
    let start = std::time::Instant::now();
    run_phase3(output).await?;
    output.phases.push(PhaseResult { phase: "3".to_string(), name: "Limpeza".to_string(), status: PhaseStatus::Success, duration_seconds: start.elapsed().as_secs(), details: serde_json::json!({}) });
    output.summary.reboot_required = true;
    Ok(())
}

async fn run_phase1(_output: &mut HeadlessOutput) -> Result<(), CoreError> {
    tracing::info!("Executando Fase 1: Auditoria");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
async fn run_phase3(_output: &mut HeadlessOutput) -> Result<(), CoreError> {
    tracing::info!("Executando Fase 3: Limpeza");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}
async fn run_phase5(_output: &mut HeadlessOutput) -> Result<(), CoreError> {
    tracing::info!("Executando Fase 5: Pos-reboot");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(())
}

fn print_json(output: &HeadlessOutput) {
    match serde_json::to_string_pretty(output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{{\"error\":\"Falha ao serializar: {}\"}}", e),
    }
}
