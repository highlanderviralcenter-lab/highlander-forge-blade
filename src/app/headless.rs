//! Modo headless — execucao real das fases sem TUI, saida JSON parseable para RMMs/MSPs.
//!
//! Correcoes vs alpha.1:
//! - Fases 1/3/5 agora usam os providers REAIS via `ProviderFactory` (antes eram
//!   `tokio::sleep` simulados que sempre reportavam "sucesso").
//! - `logs_path` vem de Config (bug antigo: BASE_DIR com caminho quebrado).
//! - Exit codes honestos: NEEDS_REBOOT / PARTIAL_SUCCESS / SUCCESS_WITH_WARNINGS
//!   derivados dos resultados reais, nao hardcoded.
//! - `--what-if` executa coleta READ-ONLY real (Fase 1) + relatorios; nenhuma
//!   operacao mutativa e executada.

use crate::app::machine_id;
use crate::app::messages::{AppMsg, AuditData};
use crate::config::Config;
use crate::core::audit::Auditor;
use crate::core::error::CoreError;
use crate::core::traits::ProviderFactory;
use chrono::Utc;
use serde::Serialize;
use tokio::sync::mpsc;

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
    pub simulation: bool,
    pub phases: Vec<PhaseResult>,
    pub summary: Summary,
    pub logs_path: String,
    pub reports_path: String,
}

#[derive(Debug, Serialize)]
pub struct PhaseResult {
    pub phase: String,
    pub name: String,
    pub status: PhaseStatus,
    pub duration_seconds: u64,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus { Success, Warning, Failed, Skipped }

#[derive(Debug, Serialize, Default)]
pub struct Summary {
    pub bytes_freed: u64,
    pub services_altered: u32,
    pub registry_keys_removed: u32,
    pub updates_installed: u32,
    pub reboot_required: bool,
}

/// Canal de progresso descartado: o reducer da TUI nao existe no modo headless,
/// mas `Auditor` e generico e exige um Sender (DI mantida).
fn null_channel() -> mpsc::Sender<AppMsg> {
    let (tx, _rx) = mpsc::channel(1);
    tx
}

pub async fn run(auto_phase: Option<String>, what_if: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::load();
    let _ = cfg.ensure_dirs();
    let machine_id = machine_id::get_or_create_machine_id_with(&cfg)
        .unwrap_or_else(|_| "unknown".to_string());

    let mut output = HeadlessOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        machine_id,
        timestamp: Utc::now(),
        exit_code: exit_codes::SUCCESS,
        exit_reason: String::new(),
        simulation: what_if,
        phases: Vec::new(),
        summary: Summary::default(),
        logs_path: cfg.logs_dir().display().to_string(),
        reports_path: cfg.reports_dir().display().to_string(),
    };

    let phase = auto_phase.as_deref().unwrap_or("all");
    let code = match phase {
        "0" | "all" => run_pipeline(&mut output, &cfg, what_if).await,
        "1" => run_single_audit(&mut output, &cfg).await,
        "3" => run_single_cleanup(&mut output, &cfg, what_if).await,
        "5" => run_single_repair(&mut output, &cfg, what_if).await,
        other => {
            output.exit_code = exit_codes::FATAL_ERROR;
            output.exit_reason = format!("Fase invalida: {} (use 0|1|3|5|all)", other);
            print_and_exit(output);
        }
    };

    output.exit_code = code;
    print_and_exit(output);
}

fn print_and_exit(output: HeadlessOutput) -> ! {
    let code = output.exit_code;
    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{{\"error\":\"Falha ao serializar: {}\"}}", e),
    }
    std::process::exit(code);
}

// ── Pipeline completo ────────────────────────────────────────────────

async fn run_pipeline(output: &mut HeadlessOutput, cfg: &Config, what_if: bool) -> i32 {
    // Fase 1: auditoria (sempre executada — read-only).
    let audit = match run_phase1_audit(output).await {
        Ok(data) => data,
        Err(e) => {
            output.exit_reason = format!("Fase 1 falhou: {}", e);
            return exit_codes::FATAL_ERROR;
        }
    };

    if what_if {
        // Simulacao: relatorio do que SERA feito; nada mutativo executado.
        output.summary.services_altered = estimate_third_party_services(&audit);
        output.exit_reason = "Modo simulacao: auditoria + estimativa, nada alterado".to_string();
        write_reports(output, cfg, &audit);
        return exit_codes::SIMULATION_COMPLETE;
    }

    // Fase 3: limpeza real.
    let freed = match run_phase3_cleanup(output, cfg).await {
        Ok(bytes) => bytes,
        Err(e) => {
            output.exit_reason = format!("Fase 3 falhou: {}", e);
            return exit_codes::FATAL_ERROR;
        }
    };
    output.summary.bytes_freed = freed;

    // Fase 4: gravar flag de pos-reboot (o reboot em si fica a criterio do
    // operador/RMM — seguranca: nunca reiniciar maquina gerenciada sem janela).
    let reboot_ok = schedule_reboot_flag(output, cfg);
    output.summary.reboot_required = reboot_ok;

    // Relatorios fisicos em disco.
    write_reports(output, cfg, &audit);

    let warnings = output.phases.iter().any(|p| p.status == PhaseStatus::Warning);
    if output.summary.reboot_required {
        output.exit_reason = "Fases concluidas; reinicializacao necessaria".to_string();
        exit_codes::NEEDS_REBOOT
    } else if warnings {
        output.exit_reason = "Concluido com avisos".to_string();
        exit_codes::SUCCESS_WITH_WARNINGS
    } else {
        output.exit_reason = "Todas as fases concluidas".to_string();
        exit_codes::SUCCESS
    }
}

async fn run_single_audit(output: &mut HeadlessOutput, cfg: &Config) -> i32 {
    match run_phase1_audit(output).await {
        Ok(data) => {
            write_reports(output, cfg, &data);
            output.exit_reason = "Fase 1 concluida".to_string();
            exit_codes::SUCCESS
        }
        Err(e) => {
            output.exit_reason = format!("Fase 1 falhou: {}", e);
            exit_codes::FATAL_ERROR
        }
    }
}

async fn run_single_cleanup(output: &mut HeadlessOutput, cfg: &Config, what_if: bool) -> i32 {
    if what_if {
        output.phases.push(PhaseResult {
            phase: "3".into(), name: "Limpeza (simulacao)".into(),
            status: PhaseStatus::Skipped, duration_seconds: 0,
            details: serde_json::json!({"motivo": "modo what-if: nenhuma exclusao foi executada"}),
        });
        output.exit_reason = "Modo simulacao".to_string();
        return exit_codes::SIMULATION_COMPLETE;
    }
    match run_phase3_cleanup(output, cfg).await {
        Ok(bytes) => {
            output.summary.bytes_freed = bytes;
            output.summary.reboot_required = true;
            output.exit_reason = "Fase 3 concluida; reboot recomendado".to_string();
            exit_codes::NEEDS_REBOOT
        }
        Err(e) => {
            output.exit_reason = format!("Fase 3 falhou: {}", e);
            exit_codes::FATAL_ERROR
        }
    }
}

async fn run_single_repair(output: &mut HeadlessOutput, cfg: &Config, what_if: bool) -> i32 {
    if what_if {
        output.phases.push(PhaseResult {
            phase: "5".into(), name: "Reparo pos-reboot (simulacao)".into(),
            status: PhaseStatus::Skipped, duration_seconds: 0,
            details: serde_json::json!({"planejado": ["sfc /scannow", "dism /Online /Cleanup-Image /RestoreHealth"]}),
        });
        output.exit_reason = "Modo simulacao".to_string();
        return exit_codes::SIMULATION_COMPLETE;
    }
    run_phase5_repair(output, cfg).await;
    let failed = output.phases.iter().any(|p| p.status == PhaseStatus::Failed);
    if failed {
        output.exit_reason = "Fase 5 parcialmente falha".to_string();
        exit_codes::PARTIAL_SUCCESS
    } else {
        output.exit_reason = "Fase 5 concluida".to_string();
        exit_codes::SUCCESS
    }
}

// ── Fases reais ──────────────────────────────────────────────────────

async fn run_phase1_audit(output: &mut HeadlessOutput) -> Result<AuditData, CoreError> {
    tracing::info!("Executando Fase 1: Auditoria (providers reais)");
    let start = std::time::Instant::now();
    let sys = ProviderFactory::system_info();
    let reg = ProviderFactory::registry();
    let svc = ProviderFactory::services();
    let auditor = Auditor::new(&*sys, &*reg, &*svc);
    let result = auditor.run_full(&null_channel()).await;
    let dur = start.elapsed().as_secs();
    match result {
        Ok(data) => {
            let details = serde_json::json!({
                "cpu": data.cpu.as_ref().map(|c| c.name.clone()).unwrap_or_default(),
                "discos": data.disks.len(),
                "softwares": data.software.len(),
                "servicos": data.services.len(),
                "run_keys": data.registry_run_keys.len(),
            });
            output.phases.push(PhaseResult {
                phase: "1".into(), name: "Auditoria".into(),
                status: PhaseStatus::Success, duration_seconds: dur, details,
            });
            Ok(data)
        }
        Err(e) => {
            output.phases.push(PhaseResult {
                phase: "1".into(), name: "Auditoria".into(),
                status: PhaseStatus::Failed, duration_seconds: dur,
                details: serde_json::json!({"erro": e.to_string()}),
            });
            Err(e)
        }
    }
}

async fn run_phase3_cleanup(output: &mut HeadlessOutput, cfg: &Config) -> Result<u64, CoreError> {
    use crate::core::cleanup::{execute_cleanup_plan, CleanupPlan};
    tracing::info!("Executando Fase 3: Limpeza (provider real)");
    let start = std::time::Instant::now();
    let provider = ProviderFactory::cleanup();
    let plan = CleanupPlan::from_config(cfg);
    let report = execute_cleanup_plan(&*provider, &plan)?;
    let dur = start.elapsed().as_secs();

    let mut status = PhaseStatus::Success;
    if !report.failures.is_empty() {
        status = PhaseStatus::Warning;
    }
    output.phases.push(PhaseResult {
        phase: "3".into(), name: "Limpeza".into(),
        status, duration_seconds: dur,
        details: serde_json::to_value(&report).unwrap_or_default(),
    });
    Ok(report.bytes_freed)
}

fn schedule_reboot_flag(output: &mut HeadlessOutput, cfg: &Config) -> bool {
    use crate::app::state::{write_post_reboot_flag, PostRebootFlag};
    let flag = PostRebootFlag::new(env!("CARGO_PKG_VERSION"));
    match write_post_reboot_flag(cfg, &flag) {
        Ok(()) => {
            output.phases.push(PhaseResult {
                phase: "4".into(), name: "Agendamento pos-reboot".into(),
                status: PhaseStatus::Success, duration_seconds: 0,
                details: serde_json::json!({
                    "flag_path": cfg.post_reboot_flag_path().display().to_string(),
                    "instrucao": "hfb --phase 5 deve ser executado apos o proximo boot (RunOnce registrado)"
                }),
            });
            true
        }
        Err(e) => {
            tracing::warn!("Falha ao gravar flag pos-reboot: {}", e);
            output.phases.push(PhaseResult {
                phase: "4".into(), name: "Agendamento pos-reboot".into(),
                status: PhaseStatus::Failed, duration_seconds: 0,
                details: serde_json::json!({"erro": e.to_string()}),
            });
            false
        }
    }
}

async fn run_phase5_repair(output: &mut HeadlessOutput, cfg: &Config) {
    use crate::core::repair::{run_dism, run_sfc, RepairOutcome};
    tracing::info!("Executando Fase 5: Reparo pos-reboot (SFC + DISM)");

    let jobs: Vec<(&str, std::pin::Pin<Box<dyn std::future::Future<Output = RepairOutcome> + Send>>)> = vec![
        ("SFC", Box::pin(run_sfc())),
        ("DISM", Box::pin(run_dism())),
    ];

    for (tool, future) in jobs {
        let start = std::time::Instant::now();
        let outcome = future.await;
        let dur = start.elapsed().as_secs();
        let (status, details) = match &outcome {
            RepairOutcome::NoIssues => (PhaseStatus::Success, serde_json::json!({"resultado": "nenhum problema encontrado"})),
            RepairOutcome::Fixed(n) => (PhaseStatus::Success, serde_json::json!({"resultado": format!("{} itens reparados", n)})),
            RepairOutcome::PendingReboot => (PhaseStatus::Warning, serde_json::json!({"resultado": "pendente novo reboot"})),
            RepairOutcome::Failed(err) => (PhaseStatus::Failed, serde_json::json!({"erro": err})),
        };
        output.phases.push(PhaseResult {
            phase: "5".into(), name: format!("Reparo {}", tool),
            status, duration_seconds: dur, details,
        });
    }

    // Concluiu: limpa a flag pos-reboot.
    let _ = crate::app::state::clear_post_reboot_flag(cfg);
}

fn estimate_third_party_services(audit: &AuditData) -> u32 {
    audit.services.iter().filter(|s| s.is_third_party).count() as u32
}

fn write_reports(output: &mut HeadlessOutput, cfg: &Config, audit: &AuditData) {
    use crate::core::report::{generate_html, generate_json, generate_txt};
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let base = cfg.reports_dir();
    let mut wrote = Vec::new();
    for (name, content) in [
        (format!("auditoria_{}.json", ts), generate_json(audit)),
        (format!("auditoria_{}.txt", ts), generate_txt(audit)),
        (format!("auditoria_{}.html", ts), generate_html(audit, &output.machine_id)),
    ] {
        let path = base.join(&name);
        match std::fs::write(&path, content) {
            Ok(()) => wrote.push(path.display().to_string()),
            Err(e) => tracing::warn!("Falha ao escrever {}: {}", path.display(), e),
        }
    }
    if !wrote.is_empty() {
        output.phases.push(PhaseResult {
            phase: "R".into(), name: "Relatorios".into(),
            status: PhaseStatus::Success, duration_seconds: 0,
            details: serde_json::json!({"arquivos": wrote}),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable_contract() {
        // Contratos publicos para RMMs — mudar = breaking change.
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::NEEDS_REBOOT, 5);
        assert_eq!(exit_codes::SIMULATION_COMPLETE, 3);
        assert_eq!(exit_codes::PARTIAL_SUCCESS, 6);
    }

    #[test]
    fn summary_serializes_snake_case_contract() {
        let s = Summary { bytes_freed: 10, services_altered: 0, registry_keys_removed: 0, updates_installed: 0, reboot_required: true };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["bytes_freed"], 10);
        assert_eq!(v["reboot_required"], true);
    }
}
