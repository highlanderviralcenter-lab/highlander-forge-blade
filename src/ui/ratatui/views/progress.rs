use crate::app::messages::{AppState, LogLevel};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
    ]).margin(1).split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("FASE 1", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - Auditoria de Hardware e Software"),
    ])).block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    let percent = state.progress.clamp(0.0, 100.0) as u16;
    let gauge = Gauge::default()
        .percent(percent)
        .label(format!("{}% - {}", percent, state.status_message))
        .block(Block::default().borders(Borders::ALL).title("Progresso").border_style(Color::Blue))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray));
    frame.render_widget(gauge, chunks[1]);

    let items: Vec<ListItem> = state.logs.iter().rev().take(50).map(|e| {
        let color = match e.level {
            LogLevel::Error => Color::Red,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Success => Color::Green,
            LogLevel::Phase => Color::Cyan,
            _ => Color::White,
        };
        ListItem::new(format!("[{}] {}", e.timestamp.format("%H:%M:%S"), e.message))
            .style(Style::default().fg(color))
    }).collect();

    let logs = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Logs").border_style(Color::Gray));
    frame.render_widget(logs, chunks[2]);

    let footer = Paragraph::new(Span::styled(
        "Voltar: Backspace | Sair: Q/Esc",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(footer, chunks[3]);
}
