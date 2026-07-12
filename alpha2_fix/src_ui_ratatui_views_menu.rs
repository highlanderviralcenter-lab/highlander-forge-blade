use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::messages::LogEntry;
use crate::app::state::AppState;

const MENU_ITEMS: [&str; 11] = [
    "1. Executar Auditoria Completa",
    "2. Limpeza e Otimizacao",
    "3. Ver Relatorio",
    "4. Ver Logs",
    "5. Reinicializacao Agendada",
    "6. Verificar Atualizacoes",
    "7. Configuracoes",
    "8. Modo Headless (JSON)",
    "9. Exportar Relatorio",
    "10. Ajuda",
    "11. Sair",
];

pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),     // Menu
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Header
    let header = Paragraph::new("Highlander Forge Blade v3.0.0-alpha.1")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Menu
    let menu_items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.menu_selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(*item).style(style)
        })
        .collect();

    let menu = List::new(menu_items)
        .block(Block::default()
            .title(" Menu Principal ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)));
    frame.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑↓ Navegar | Enter Selecionar | Q Sair | ESC Voltar")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}