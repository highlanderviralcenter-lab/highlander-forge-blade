//! Tela inicial — menu principal

use crate::app::state::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

const MENU_ITEMS: &[(&str, &str)] = &[
    ("0", "RODAR TUDO (Fases 1 > 2 > 3 > 4 > 5)"),
    ("1", "Apenas FASE 1 (Levantamento/Auditoria)"),
    ("2", "FASE 2 em diante (Resumo + Confirma + 3 + 4 + 5)"),
    ("2.1", "Apenas FASE 2 (Somente resumo e pausa)"),
    ("3", "FASE 3 em diante (Limpeza + Otimizacao + 4 + 5)"),
    ("3.1", "Apenas FASE 3 (Somente limpeza e otimizacao)"),
    ("4", "FASE 4 em diante (Reboot + 5 automatico)"),
    ("4.1", "Apenas FASE 4 (Somente configurar reboot)"),
    ("5", "Apenas FASE 5 (Pos-reboot: SFC + DISM + CHKDSK)"),
    ("R", "GERAR RELATORIO (auditoria dos dados existentes)"),
    ("X", "SAIR"),
];

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("HIGHLANDER FORGE BLADE", Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(
            "Manutencao Profissional do Windows",
            Style::default().fg(Color::Gray),
        )),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, (key, label))| {
            let style = if i == state.selected_menu_item {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = if i == state.selected_menu_item {
                format!("> [{}] {}", key, label)
            } else {
                format!("  [{}] {}", key, label)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default()
            .title("Menu Principal")
            .borders(Borders::ALL)
            .border_style(Color::Cyan));

    frame.render_widget(menu, chunks[1]);

    let footer = Paragraph::new(Span::styled(
        "Navegar: Setas | Selecionar: Enter | Sair: Q/Esc",
        Style::default().fg(Color::Gray),
    ))
    .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}
