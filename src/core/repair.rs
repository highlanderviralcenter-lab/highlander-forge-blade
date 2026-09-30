//! Fase 5: reparo pos-reboot — execucao REAL de SFC/DISM com parsing honesto
//! dos resultados (bug antigo: status sempre "sucesso" sem ler a saida).

use std::process::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairOutcome {
    NoIssues,
    Fixed(u32),
    PendingReboot,
    Failed(String),
}

/// Interpreta a saida consolidada do System File Checker.
/// Strings oficiais do SFC (independentes de locale porque forçamos /en-us via
/// chamadas diretas; fallback cobre ambos os idiomas documentados).
pub fn parse_sfc_output(text: &str) -> RepairOutcome {
    let t = text.to_lowercase();
    if t.contains("found no integrity problems") || t.contains("nenhum problema de integridade") {
        RepairOutcome::NoIssues
    } else if t.contains("successfully repaired") || t.contains("reparou com sucesso") {
        // Contagem exata nao e exposta pelo SFC: Fixed(0) sinaliza "reparou algo".
        RepairOutcome::Fixed(0)
    } else if t.contains("pending reboot") || t.contains("pendente") && t.contains("reinicializacao") {
        RepairOutcome::PendingReboot
    } else if t.contains("could not fix") || t.contains("nao foi possivel corrigir") || t.contains("error") {
        RepairOutcome::Failed(text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("falha desconhecida").to_string())
    } else if t.trim().is_empty() {
        RepairOutcome::Failed("SFC nao produziu saida".to_string())
    } else {
        RepairOutcome::Failed(format!("saida inesperada do SFC: {}", text.lines().last().unwrap_or("")))
    }
}

/// Interpreta a saida do DISM RestoreHealth.
pub fn parse_dism_output(text: &str) -> RepairOutcome {
    let t = text.to_lowercase();
    if t.contains("the restore operation completed successfully") || t.contains("operacao de restauracao concluida com sucesso") {
        RepairOutcome::Fixed(0)
    } else if t.contains("no action is required") || t.contains("repaired image") && t.contains("healthy") {
        RepairOutcome::NoIssues
    } else if t.contains("error") || t.contains("failed") {
        RepairOutcome::Failed(text.lines().rev().find(|l| l.trim().len() > 3).unwrap_or("falha desconhecida").trim().to_string())
    } else if t.trim().is_empty() {
        RepairOutcome::Failed("DISM nao produziu saida".to_string())
    } else {
        RepairOutcome::Failed(format!("saida inesperada do DISM: {}", text.lines().last().unwrap_or("")))
    }
}

fn combine_status(o: &Output) -> String {
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push('\n');
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    s
}

/// Executa `sfc /scannow` (requer elevacao) e interpreta o resultado real.
pub async fn run_sfc() -> RepairOutcome {
    let result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("sfc")
            .args(["/scannow"])
            .output()
    })
    .await;

    match result {
        Ok(Ok(out)) => {
            if !out.status.success() && out.stdout.is_empty() && out.stderr.is_empty() {
                return RepairOutcome::Failed(format!("sfc terminou com exit {:?}", out.status.code()));
            }
            parse_sfc_output(&combine_status(&out))
        }
        Ok(Err(e)) => RepairOutcome::Failed(format!("falha ao iniciar sfc: {} (exige prompt elevado)", e)),
        Err(e) => RepairOutcome::Failed(format!("erro interno ao executar sfc: {}", e)),
    }
}

/// Executa `dism /Online /Cleanup-Image /RestoreHealth` e interpreta o resultado.
pub async fn run_dism() -> RepairOutcome {
    let result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("dism")
            .args(["/Online", "/Cleanup-Image", "/RestoreHealth"])
            .output()
    })
    .await;

    match result {
        Ok(Ok(out)) => parse_dism_output(&combine_status(&out)),
        Ok(Err(e)) => RepairOutcome::Failed(format!("falha ao iniciar dism: {} (exige prompt elevado)", e)),
        Err(e) => RepairOutcome::Failed(format!("erro interno ao executar dism: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfc_no_integrity_problems_en() {
        let o = parse_sfc_output("Windows Resource Protection found no integrity problems.");
        assert_eq!(o, RepairOutcome::NoIssues);
    }

    #[test]
    fn sfc_repaired_pt() {
        let o = parse_sfc_output("A Protecao de Recursos do Windows reparou com sucesso os arquivos.");
        assert!(matches!(o, RepairOutcome::Fixed(_)));
    }

    #[test]
    fn sfc_could_not_fix_is_failure() {
        let o = parse_sfc_output("Windows Resource Protection found corrupt files but was unable to fix some of them.\nDetails are included in the CBS.Log.");
        assert!(matches!(o, RepairOutcome::Failed(_)));
    }

    #[test]
    fn sfc_empty_output_is_failure_not_fake_success() {
        // Bug antigo: vazio virava "sucesso". Agora e falha explicita.
        assert!(matches!(parse_sfc_output("   "), RepairOutcome::Failed(_)));
    }

    #[test]
    fn dism_restore_success() {
        let o = parse_dism_output("The restore operation completed successfully. The operation completed successfully.");
        assert!(matches!(o, RepairOutcome::Fixed(_)));
    }

    #[test]
    fn dism_error_detected() {
        let o = parse_dism_output("Error: 0x800f081f\nThe source files could not be found.");
        assert!(matches!(o, RepairOutcome::Failed(_)));
    }
}
