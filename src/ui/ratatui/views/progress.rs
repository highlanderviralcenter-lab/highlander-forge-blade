//! Tela de progresso — gauge + logs

use crate::app::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(8),
        ])
        .split(frame.area());

    let gauge = Gauge::default()
        .block(Block::default().title("Progresso").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(state.progress as f64 / 100.0)
        .label(format!("{:.1}%", state.progress));

    frame.render_widget(gauge, chunks[0]);

    let detail = Paragraph::new(state.status_message.clone())
        .block(Block::default().title("Status").borders(Borders::ALL));

    frame.render_widget(detail, chunks[1]);

    let log_items: Vec<ListItem> = state.logs
        .iter()
        .rev()
        .take(6)
        .map(|log| {
            let color = match log.level {
                crate::app::messages::LogLevel::Error => Color::Red,
                crate::app::messages::LogLevel::Warn => Color::Yellow,
                crate::app::messages::LogLevel::Success => Color::Green,
                _ => Color::Gray,
            };
            ListItem::new(format!("[{}] {}", log.level, log.message))
                .style(Style::default().fg(color))
        })
        .collect();

    let logs = List::new(log_items)
        .block(Block::default().title("Logs").borders(Borders::ALL));

    frame.render_widget(logs, chunks[2]);
}
