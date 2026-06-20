//! Tela de progresso — Gauge + logs coloridos + status

use crate::app::messages::{AppState, LogLevel};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .margin(1)
    .split(area);

    // Header
    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("FASE 1", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Auditoria de Hardware e Software"),
        ]),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Gauge de progresso
    let percent = state.progress.clamp(0.0, 100.0) as u16;
    let gauge_label = format!("{}% — {}", percent, state.status_message);
    let gauge = Gauge::default()
        .percent(percent)
        .label(gauge_label)
        .block(Block::default().borders(Borders::ALL).title("Progresso").border_style(Color::Blue))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray));
    frame.render_widget(gauge, chunks[1]);

    // Logs coloridos
    let log_items: Vec<ListItem> = state
        .logs
        .iter()
        .rev()
        .take(50)
        .map(|entry| {
            let level_style = match entry.level {
                LogLevel::Error => Style::default().fg(Color::Red),
                LogLevel::Warn => Style::default().fg(Color::Yellow),
                LogLevel::Success => Style::default().fg(Color::Green),
                LogLevel::Phase => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                LogLevel::Info => Style::default().fg(Color::White),
                LogLevel::Debug => Style::default().fg(Color::Gray),
            };
            let content = format!("[{}] {} — {}", entry.timestamp.format("%H:%M:%S"), entry.level, entry.message);
            ListItem::new(content).style(level_style)
        })
        .collect();

    let logs_list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title("Logs").border_style(Color::Gray));
    frame.render_widget(logs_list, chunks[2]);

    // Footer
    let footer = Paragraph::new(Span::styled(
        "Voltar: Backspace | Sair: Q/Esc",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(footer, chunks[3]);
}
