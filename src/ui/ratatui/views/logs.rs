//! Tela de logs â€” exibe todos os logs do sistema com scroll

use crate::app::messages::{AppState, LogLevel};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
    ])
    .margin(1)
    .split(area);

    // Header
    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("LOGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" â€” {} entradas", state.logs.len())),
        ]),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Logs coloridos
    let log_lines: Vec<Line> = state
        .logs
        .iter()
        .map(|entry| {
            let level_color = match entry.level {
                LogLevel::Error => Color::Red,
                LogLevel::Warn => Color::Yellow,
                LogLevel::Success => Color::Green,
                LogLevel::Phase => Color::Cyan,
                LogLevel::Info => Color::White,
                LogLevel::Debug => Color::Gray,
            };
            Line::from(vec![
                Span::styled(format!("[{}] ", entry.timestamp.format("%H:%M:%S")), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:8} ", entry.level.to_string()), Style::default().fg(level_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[{}] ", entry.source), Style::default().fg(Color::Magenta)),
                Span::raw(&entry.message),
            ])
        })
        .collect();

    let content = Paragraph::new(Text::from(log_lines))
        .block(Block::default().borders(Borders::ALL).title("Log Completo").border_style(Color::Gray));
    frame.render_widget(content, chunks[1]);

    // Footer
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Voltar: Backspace", Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled("Sair: Q/Esc", Style::default().fg(Color::Gray)),
        ]),
    ]));
    frame.render_widget(footer, chunks[2]);
}
