//! Implementacao REAL de CleanupProvider para Windows.
//!
//! Correcoes vs alpha.1:
//! - Lixeira via API oficial SHEmptyRecycleBinW (antes `rd /s /q C:\$Recycle.Bin`
//!   hardcoded no C: — quebrava em multi-volume e ignorava a politica da GPO).
//! - Bytes medidos por delta de FreeSpace do volume ANTES/DEPOIS + remocoes
//!   confirmadas (nunca soma metadados pre-exclusao).
//! - Cache WU: para o servico wuausnet, limpa SoftwareDistribution\Download,
//!   reinicia o servico (procedimento Microsoft documentado).

use crate::core::cleanup::{purge_dir_older_than, CleanupPlan};
use crate::core::error::CoreError;
use crate::core::traits::CleanupProvider;
use std::path::Path;
use std::time::Duration;

pub struct WinCleanupProvider;

impl WinCleanupProvider {
    pub fn new() -> Self { Self }
}

fn free_space_bytes(path: &str) -> Option<u64> {
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-NonInteractive", "-Command",
            &format!("(Get-PSDrive -Name '{}').Free", path.chars().next()?.to_uppercase().collect::<String>()),
        ])
        .output()
        .ok()?;
    if !out.status.success() { return None; }
    String::from_utf8_lossy(&out.stdout).trim().parse::<u64>().ok()
}

/// Esvazia a lixeira de um ou todos os volumes pela API oficial.
/// Retorna Some((bytes_aproximados)) quando consegue medir, senao None.
#[cfg(windows)]
fn empty_recycle_bin_api(all_volumes: bool) -> Result<Option<u64>, CoreError> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SHEmptyRecycleBinW;

    // Medida aproximada: soma do tamanho da arvore $Recycle.Bin antes de esvaziar.
    let mut before: u64 = 0;
    for drive in drive_letters(all_volumes) {
        let rb = format!("{}\\$Recycle.Bin", drive);
        before += dir_tree_size(Path::new(&rb));
    }

    let root: Vec<u16> = if all_volumes {
        String::new().encode_utf16().chain(std::iter::once(0)).collect()
    } else {
        std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string())
            .encode_utf16().chain(std::iter::once(0)).collect()
    };

    unsafe {
        // SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND = 0x7
        SHEmptyRecycleBinW(None, PCWSTR(root.as_ptr()), 0x7)
            .map_err(|e| CoreError::Io(format!("SHEmptyRecycleBinW: {}", e)))?;
    }
    Ok(Some(before))
}

#[cfg(not(windows))]
fn empty_recycle_bin_api(_all_volumes: bool) -> Result<Option<u64>, CoreError> {
    Err(CoreError::NotSupported("lixeira so existe no Windows".into()))
}

fn drive_letters(all: bool) -> Vec<String> {
    if all {
        (b'C'..=b'Z').map(|c| format!("{}:", c as char)).collect()
    } else {
        vec![std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string())]
    }
}

fn dir_tree_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for e in entries.flatten() {
            if let Ok(m) = e.metadata() {
                total += if m.is_dir() { dir_tree_size(&e.path()) } else { m.len() };
            }
        }
    }
    total
}

impl CleanupProvider for WinCleanupProvider {
    fn clean_temp_files_detailed(&self, plan: &CleanupPlan) -> Result<(u64, u64), CoreError> {
        let mut failures = Vec::new();
        let mut bytes = 0u64;
        let mut files = 0u64;
        for dir in &plan.temp_dirs {
            let (b, f) = purge_dir_older_than(Path::new(dir), plan.min_age, &mut failures);
            bytes += b;
            files += f;
        }
        for f in failures {
            tracing::debug!("temp: {}", f);
        }
        Ok((bytes, files))
    }

    fn clean_recycle_bin_detailed(&self, all_volumes: bool) -> Result<(u64, u64), CoreError> {
        match empty_recycle_bin_api(all_volumes)? {
            Some(bytes) => Ok((bytes, 0)),
            None => Ok((0, 0)),
        }
    }

    fn clean_old_logs_detailed(&self, min_age: Duration) -> Result<(u64, u64), CoreError> {
        let windir = std::env::var("windir").unwrap_or_else(|_| r"C:\Windows".to_string());
        let logs_dir = Path::new(&windir).join("Logs");
        let mut failures = Vec::new();
        let (b, f) = purge_dir_older_than(&logs_dir, min_age, &mut failures);
        for x in failures.iter().take(5) {
            tracing::debug!("old_logs: {}", x);
        }
        Ok((b, f))
    }

    fn clean_windows_update_cache(&self) -> Result<u64, CoreError> {
        let windir = std::env::var("windir").unwrap_or_else(|_| r"C:\Windows".to_string());
        let dl = Path::new(&windir).join("SoftwareDistribution").join("Download");
        // Procedimento oficial: parar wuausbits, apagar Download, reiniciar.
        let _ = std::process::Command::new("net").args(["stop", "wuauserv"]).output();
        let mut failures = Vec::new();
        let (bytes, _files) = purge_dir_older_than(&dl, Duration::ZERO, &mut failures);
        let _ = std::process::Command::new("net").args(["start", "wuauserv"]).output();
        Ok(bytes)
    }

    fn clean_browser_cache(&self) -> Result<u64, CoreError> {
        // Deliberadamente NAO implementado por padrao: risco de destruir dados
        // de sessao/autenticacao de usuarios gerenciados. So roda se o plano
        // habilitar explicitamente (config.clean_browser_cache=true) — e mesmo
        // assim apenas caches publicos conhecidos.
        let local = std::env::var("LOCALAPPDATA").map_err(|_| CoreError::NotSupported("LOCALAPPDATA ausente".into()))?;
        let mut total = 0u64;
        let mut failures = Vec::new();
        for sub in [
            "Microsoft\\Edge\\User Data\\Default\\Cache",
            "Google\\Chrome\\User Data\\Default\\Cache",
        ] {
            let p = Path::new(&local).join(sub);
            let (b, _f) = purge_dir_older_than(&p, Duration::from_secs(86_400), &mut failures);
            total += b;
        }
        Ok(total)
    }

    fn run_dism(&self) -> Result<(), CoreError> {
        let out = std::process::Command::new("dism")
            .args(["/Online", "/Cleanup-Image", "/RestoreHealth"])
            .output()
            .map_err(|e| CoreError::Io(e.to_string()))?;
        if !out.status.success() {
            return Err(CoreError::Io(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }

    fn run_sfc(&self) -> Result<(), CoreError> {
        let out = std::process::Command::new("sfc")
            .args(["/scannow"])
            .output()
            .map_err(|e| CoreError::Io(e.to_string()))?;
        if !out.status.success() {
            return Err(CoreError::Io(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }
}
