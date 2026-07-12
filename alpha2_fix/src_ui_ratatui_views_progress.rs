use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Titulo
            Constraint::Length(3),  // Gauge
            Constraint::Length(2),  // Status
            Constraint::Min(5),     // Logs
            Constraint::Length(2),  // Footer
        ])
        .split(size);

    // Titulo
    let title = Paragraph::new("Executando Operacao")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // Gauge de progresso
    let gauge = Gauge::default()
        .block(Block::default().title(" Progresso ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(state.progress_percent as u16)
        .label(format!("{}% - {}", state.progress_percent, state.current_phase));
    frame.render_widget(gauge, chunks[1]);

    // Status atual
    let status = Paragraph::new(state.current_phase.clone())
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[2]);

    // Logs coloridos
    let log_items: Vec<ListItem> = state.logs.iter().rev().take(20).map(|log| {
        let color = match log.level {
            crate::app::messages::LogLevel::Info => Color::White,
            crate::app::messages::LogLevel::Warn => Color::Yellow,
            crate::app::messages::LogLevel::Error => Color::Red,
            crate::app::messages::LogLevel::Success => Color::Green,
        };
        let content = format!("[{}] {}", log.timestamp, log.message);
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", log.timestamp), Style::default().fg(Color::DarkGray)),
            Span::styled(&log.message, Style::default().fg(color)),
        ]))
    }).collect();

    let logs = List::new(log_items)
        .block(Block::default().title(" Logs ").borders(Borders::ALL));
    frame.render_widget(logs, chunks[3]);

    // Footer
    let footer = Paragraph::new("B - Voltar ao Menu | ESC - Cancelar")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}