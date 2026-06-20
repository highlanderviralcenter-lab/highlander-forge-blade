//! Tela de confirmacao — Sim/Nao para reboot

use crate::app::messages::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    // Layout vertical: header, conteudo central, footer
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .margin(1)
    .split(area);

    // Header
    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("FASE 4", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Confirmacao de Reinicializacao"),
        ]),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Painel central com a pergunta
    let popup_area = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(3),
    ])
    .flex(ratatui::layout::Flex::Center)
    .split(chunks[1]);

    let inner = Layout::horizontal([Constraint::Percentage(60)])
        .flex(ratatui::layout::Flex::Center)
        .split(popup_area[1]);

    let question = Paragraph::new(Text::from(vec![
        Line::from(Span::styled("Reiniciar o sistema agora?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Apos o reboot, a Fase 5 sera executada automaticamente: "),
        ]),
        Line::from(vec![
            Span::styled("SFC /SCANNOW", Style::default().fg(Color::Green)),
            Span::raw(", "),
            Span::styled("DISM /RestoreHealth", Style::default().fg(Color::Green)),
            Span::raw(", "),
            Span::styled("CHKDSK", Style::default().fg(Color::Green)),
        ]),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Color::Yellow).title("Atencao"));
    frame.render_widget(question, inner[0]);

    // Footer com opcoes
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" = SIM, reiniciar agora  |  "),
            Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" = NAO, voltar ao menu  |  "),
            Span::styled("Backspace", Style::default().fg(Color::Gray)),
            Span::raw(" = Voltar"),
        ]),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
