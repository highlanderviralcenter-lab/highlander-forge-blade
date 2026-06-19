//! Atualizacao de estado a partir de mensagens

use crate::app::messages::*;

impl AppState {
    pub fn update(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::Tick => {}
            AppMsg::NavigateUp => {
                if self.selected_menu_item > 0 {
                    self.selected_menu_item -= 1;
                }
            }
            AppMsg::NavigateDown => {
                if self.selected_menu_item < 10 {
                    self.selected_menu_item += 1;
                }
            }
            AppMsg::Select => {
                match self.selected_menu_item {
                    0 => self.current_screen = Screen::AuditProgress,
                    10 => std::process::exit(0),
                    _ => {}
                }
            }
            AppMsg::Back => {
                self.current_screen = Screen::Menu;
            }
            AppMsg::AuditStarted => {
                self.current_screen = Screen::AuditProgress;
                self.status_message = "Iniciando auditoria...".to_string();
            }
            AppMsg::AuditProgress { phase, item, percent } => {
                self.progress = percent as f32;
                self.status_message = format!("{:?}: {}", phase, item);
            }
            AppMsg::AuditCompleted(data) => {
                self.audit_data = Some(data);
                self.progress = 100.0;
                self.status_message = "Auditoria concluida!".to_string();
                self.current_screen = Screen::Summary;
            }
            AppMsg::AuditFailed(err) => {
                self.status_message = format!("Erro: {}", err);
                self.logs.push(LogEntry::warn(format!("Auditoria falhou: {}", err)));
            }
            AppMsg::LogLine(log) => {
                self.logs.push(log);
            }
            AppMsg::Error(err) => {
                self.logs.push(LogEntry::warn(format!("Erro: {}", err)));
            }
            _ => {}
        }
    }
}
