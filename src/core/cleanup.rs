//! CleanupProvider real + plano de limpeza dirigido por Config.
//!
//! Correcoes vs alpha.1:
//! - Bytes liberados medidos DEPOIS da exclusao (antes somava metadados ANTES do
//!   `remove` e ignorava falhas → contagem inflada/falsa).
//! - Respeita idade minima (`temp_min_age_days`) — nao apaga arquivos em uso
//!   criados ha poucos minutos.
//! - Lixeira via API oficial SHEmptyRecycleBinW em TODOS os volumes (antes
//!   `rd /s /q C:\$Recycle.Bin` hardcoded e inseguro).
//! - Cache de navegador desligado por padrao (risco de dados de sessao).
//! - `WinCleanupProvider` movido para `platform::windows::cleanup` (cross-platform).

use crate::config::Config;
use crate::core::error::CoreError;
use crate::core::traits::CleanupProvider;
use serde::Serialize;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Itens de um plano de limpeza (derivado da Config — nada hardcoded).
#[derive(Debug, Clone, Serialize)]
pub struct CleanupPlan {
    pub temp_dirs: Vec<String>,
    pub min_age: Duration,
    pub clean_recycle_bin: bool,
    pub recycle_bin_all_volumes: bool,
    pub clean_browser_cache: bool,
    pub clean_old_logs: bool,
    pub clean_wu_cache: bool,
}

impl CleanupPlan {
    pub fn from_config(cfg: &Config) -> CleanupPlan {
        let base = cfg.resolved_base_dir();
        let mut temp_dirs = vec![std::env::temp_dir().display().to_string()];
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            temp_dirs.push(Path::new(&local).join("Temp").display().to_string());
        }
        #[cfg(windows)]
        temp_dirs.push(r"C:\Windows\Temp".to_string());
        // Nunca limpa o proprio diretorio de trabalho a menos que explicito.
        let _ = base;
        CleanupPlan {
            temp_dirs,
            min_age: Duration::from_secs(cfg.temp_min_age_days * 24 * 3600),
            clean_recycle_bin: true,
            recycle_bin_all_volumes: cfg.clean_recycle_bin_all_volumes,
            clean_browser_cache: cfg.clean_browser_cache,
            clean_old_logs: true,
            clean_wu_cache: true,
        }
    }
}

/// Resultio honesto da execucao do plano.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CleanupReport {
    pub bytes_freed: u64,
    pub files_removed: u64,
    pub per_step: Vec<StepResult>,
    /// Falhas individuais (arquivos em uso etc.) — nao derrubam o plano.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub step: String,
    pub bytes_freed: u64,
    pub files_removed: u64,
}

impl CleanupReport {
    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

/// Executa o plano contra um provider injetavel (testavel com mocks).
pub fn execute_cleanup_plan(provider: &dyn CleanupProvider, plan: &CleanupPlan) -> Result<CleanupReport, CoreError> {
    let mut report = CleanupReport::default();

    let (bytes, files) = provider.clean_temp_files_detailed(plan)?;
    report.per_step.push(StepResult { step: "temp_files".into(), bytes_freed: bytes, files_removed: files });
    report.bytes_freed += bytes;
    report.files_removed += files;

    if plan.clean_recycle_bin {
        match provider.clean_recycle_bin_detailed(plan.recycle_bin_all_volumes) {
            Ok((bytes, files)) => {
                report.per_step.push(StepResult { step: "recycle_bin".into(), bytes_freed: bytes, files_removed: files });
                report.bytes_freed += bytes;
                report.files_removed += files;
            }
            Err(e) => report.failures.push(format!("lixeira: {}", e)),
        }
    }

    if plan.clean_old_logs {
        match provider.clean_old_logs_detailed(plan.min_age) {
            Ok((bytes, files)) => {
                report.per_step.push(StepResult { step: "old_logs".into(), bytes_freed: bytes, files_removed: files });
                report.bytes_freed += bytes;
                report.files_removed += files;
            }
            Err(e) => report.failures.push(format!("logs antigos: {}", e)),
        }
    }

    if plan.clean_wu_cache {
        match provider.clean_windows_update_cache() {
            Ok(bytes) => {
                report.per_step.push(StepResult { step: "wu_cache".into(), bytes_freed: bytes, files_removed: 0 });
                report.bytes_freed += bytes;
            }
            Err(e) => report.failures.push(format!("cache Windows Update: {}", e)),
        }
    }

    if plan.clean_browser_cache {
        match provider.clean_browser_cache() {
            Ok(bytes) => {
                report.per_step.push(StepResult { step: "browser_cache".into(), bytes_freed: bytes, files_removed: 0 });
                report.bytes_freed += bytes;
            }
            Err(e) => report.failures.push(format!("cache de navegadores: {}", e)),
        }
    }

    Ok(report)
}

// ── Helpers compartilhados pelas implementacoes reais ────────────────

/// Remove conteudo de `dir` respeitando idade minima; retorna (bytes_removidos, arquivos).
/// A contagem de bytes acontece APENAS apos remocao bem-sucedida (correcao P0).
pub fn purge_dir_older_than(dir: &Path, min_age: Duration, failures: &mut Vec<String>) -> (u64, u64) {
    let mut bytes: u64 = 0;
    let mut count: u64 = 0;
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (0, 0), // dir inexistente/sem acesso: nao e falha critica
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        // Protege arquivos recentes (possivelmente em uso).
        if let Ok(modified) = meta.modified() {
            if now.duration_since(modified).unwrap_or(Duration::ZERO) < min_age {
                continue;
            }
        }
        let size = file_tree_size(&path);
        let removed = if meta.is_dir() {
            std::fs::remove_dir_all(&path).is_ok()
        } else {
            std::fs::remove_file(&path).is_ok()
        };
        if removed {
            bytes += size;
            count += 1;
        } else {
            failures.push(format!("em uso/sem permissao: {}", path.display()));
        }
    }
    (bytes, count)
}

fn file_tree_size(path: &Path) -> u64 {
    match std::fs::metadata(path) {
        Ok(m) if m.is_file() => m.len(),
        Ok(_) => walk_dir_size(path),
        Err(_) => 0,
    }
}

fn walk_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for e in entries.flatten() {
            if let Ok(m) = e.metadata() {
                total += if m.is_dir() { walk_dir_size(&e.path()) } else { m.len() };
            }
        }
    }
    total
}

/// Stub multiplataforma (Linux/macOS apenas para CI/testes): nao apaga nada alem
/// de uma arvore temporaria local quando explicitamente requisitado por teste.
pub struct StubCleanupProvider;

impl StubCleanupProvider {
    pub fn new() -> Self { Self }
}

impl CleanupProvider for StubCleanupProvider {
    fn clean_temp_files_detailed(&self, _plan: &CleanupPlan) -> Result<(u64, u64), CoreError> {
        Ok((0, 0))
    }
    fn clean_recycle_bin_detailed(&self, _all_volumes: bool) -> Result<(u64, u64), CoreError> {
        Ok((0, 0))
    }
    fn clean_old_logs_detailed(&self, _min_age: Duration) -> Result<(u64, u64), CoreError> {
        Ok((0, 0))
    }
    fn clean_windows_update_cache(&self) -> Result<u64, CoreError> {
        Err(CoreError::NotSupported("fora do Windows".into()))
    }
    fn clean_browser_cache(&self) -> Result<u64, CoreError> {
        Ok(0)
    }
    fn run_dism(&self) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("fora do Windows".into()))
    }
    fn run_sfc(&self) -> Result<(), CoreError> {
        Err(CoreError::NotSupported("fora do Windows".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn purge_respects_min_age_and_counts_after_removal() {
        let dir = std::env::temp_dir().join(format!("hfb_test_purge_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let fresh = dir.join("fresh.tmp");
        let old = dir.join("old.tmp");
        fs::write(&fresh, b"aaa").unwrap();
        fs::write(&old, b"bbbb").unwrap();
        // Forca mtime antigo no "old".
        let very_old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000);
        let t = filetime_set(&old, very_old);
        if t.is_err() { /* filesystem sem suporte: pula assertividade de idade */ }

        let mut failures = Vec::new();
        let (bytes, count) = purge_dir_older_than(&dir, Duration::from_secs(86_400 * 7), &mut failures);

        // O arquivo novo NAO deve ter sido removido (protecao anti-dado-em-uso).
        assert!(fresh.exists(), "arquivo recente deve ser preservado");
        if t.is_ok() {
            assert!(!old.exists(), "arquivo antigo deve ser removido");
            assert_eq!(count, 1);
            assert_eq!(bytes, 4, "bytes contam somente o que FOI removido");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    fn filetime_set(path: &Path, when: SystemTime) -> std::io::Result<()> {
        // Sem crate filetime: usamos unix syscalls via std? std nao expoe set mtime.
        // Em unix usamos `touch -t`? Simples: abrir com OpenOptions truncate nao muda mtime.
        // Implementacao portavel minima via libc-like syscall seria overkill; em Linux
        // podemos usar utimensat pela API std disponivel desde Rust 1.75 (File::set_modified).
        let f = fs::OpenOptions::new().write(true).open(path)?;
        f.set_times(std::fs::FileTimes::new().set_modified(when))?;
        Ok(())
    }

    #[test]
    fn plan_from_config_has_no_hardcoded_double_backslash() {
        let cfg = Config::default();
        let plan = CleanupPlan::from_config(&cfg);
        for d in &plan.temp_dirs {
            assert!(!d.contains(r"C:\\\\"), "caminho duplicado detectado: {}", d);
        }
    }

    #[test]
    fn stub_provider_produces_honest_zero_report() {
        let plan = CleanupPlan {
            temp_dirs: vec![], min_age: Duration::from_secs(1),
            clean_recycle_bin: true, recycle_bin_all_volumes: false,
            clean_browser_cache: false, clean_old_logs: true, clean_wu_cache: false,
        };
        let r = execute_cleanup_plan(&StubCleanupProvider::new(), &plan).unwrap();
        assert_eq!(r.bytes_freed, 0, "stub nao deve inventar espaco liberado");
    }
}
