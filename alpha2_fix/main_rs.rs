//! Entry point — Highlander Forge Blade

use highlander_forge_blade::logging::{self, LogFormat};
use highlander_forge_blade::app::messages::Message;
use highlander_forge_blade::core::audit::Auditor;
use tokio::sync::mpsc;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init_logging(LogFormat::Human);

    tracing::info!("Highlander Forge Blade v{}", env!("CARGO_PKG_VERSION"));

    #[cfg(windows)]
    if !is_admin() {
        eprintln!("ERRO: Execute como Administrador.");
        process::exit(1);
    }

    // Canal duplex: UI envia comandos, core responde com progresso
    let (tx_to_core, mut rx_from_ui) = mpsc::channel::<Message>(256);
    let (tx_to_ui, rx_to_ui) = mpsc::channel::<Message>(256);

    // Task do core — fica esperando comandos da UI
    let tx_progress = tx_to_ui.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx_from_ui.recv().await {
            match msg {
                Message::StartAudit => {
                    let auditor = Auditor::new();
                    match auditor.run_full_audit(tx_progress.clone()).await {
                        Ok(data) => {
                            let _ = tx_progress.send(Message::AuditComplete(data)).await;
                        }
                        Err(e) => {
                            let _ = tx_progress.send(Message::AuditError(e.to_string())).await;
                        }
                    }
                }
                Message::StartCleanup => {
                    // TODO: implementar Fase 3
                    let _ = tx_progress.send(Message::CleanupComplete).await;
                }
                Message::Quit => break,
                _ => {}
            }
        }
    });

    // Inicia TUI
    #[cfg(feature = "tui")]
    highlander_forge_blade::ui::ratatui::run_tui(rx_to_ui, tx_to_core).await?;

    Ok(())
}

#[cfg(windows)]
fn is_admin() -> bool {
    use windows::Win32::Security::*;
    use windows::Win32::Foundation::HANDLE;
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(
            windows::Win32::System::Threading::GetCurrentProcess(),
            TOKEN_QUERY,
            &mut token
        ).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            if GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size,
                &mut size
            ).is_ok() {
                return elevation.TokenIsElevated != 0;
            }
        }
        false
    }
}
