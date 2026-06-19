//! Views — telas da aplicacao

mod menu;
mod progress;
mod logs;
mod summary;
mod confirm;
mod report;

use crate::app::state::{AppState, Screen};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &mut AppState) {
    match state.current_screen {
        Screen::Menu => menu::render(frame, state),
        Screen::AuditProgress => progress::render(frame, state),
        Screen::Summary => summary::render(frame, state),
        Screen::CleanupProgress => progress::render(frame, state),
        Screen::RebootConfirm => confirm::render(frame, state),
        Screen::PostRebootProgress => progress::render(frame, state),
        Screen::ReportView => report::render(frame, state),
        Screen::LogsView => logs::render(frame, state),
    }
}
