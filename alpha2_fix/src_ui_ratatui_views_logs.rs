use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(size);

    let header = Paragraph::new("Logs do Sistema")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    let log_items: Vec<ListItem> = state.logs.iter().map(|log| {
        let (color, prefix) = match log.level {
            crate::app::messages::LogLevel::Info => (Color::White, "INFO"),
            crate::app::messages::LogLevel::Warn => (Color::Yellow, "WARN"),
            crate::app::messages::LogLevel::Error => (Color::Red, "ERROR"),
            crate::app::messages::LogLevel::Success => (Color::Green, "OK"),
        };

        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", log.timestamp), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("[{}] ", prefix), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(&log.message, Style::default().fg(color)),
        ]))
    }).collect();

    let logs = List::new(log_items)
        .block(Block::default().title(" Todos os Logs ").borders(Borders::ALL));
    frame.render_widget(logs, chunks[1]);

    let footer = Paragraph::new("B - Voltar | ↑↓ Scroll")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}