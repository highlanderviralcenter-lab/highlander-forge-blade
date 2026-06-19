//! Stub — implementar na Fase B completa
use crate::app::state::AppState;
use ratatui::{widgets::Paragraph, Frame};

pub fn render(frame: &mut Frame, _state: &mut AppState) {
    frame.render_widget(
        Paragraph::new("Em desenvolvimento..."),
        frame.area(),
    );
}
