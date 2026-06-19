//! Comandos async que orquestram o core

use crate::app::messages::AppMsg;
use tokio::sync::mpsc::Sender;

pub async fn start_audit(tx: Sender<AppMsg>) {
    let _ = tx.send(AppMsg::AuditStarted).await;
    // Delega para core::audit
    let _ = tx.send(AppMsg::AuditCompleted(Box::default())).await;
}

pub async fn start_cleanup(tx: Sender<AppMsg>) {
    let _ = tx.send(AppMsg::CleanupStarted).await;
    // Delega para core::cleanup
    let _ = tx.send(AppMsg::CleanupCompleted).await;
}
