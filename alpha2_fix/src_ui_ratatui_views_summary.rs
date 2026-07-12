use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::state::AppState;

pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),     // Conteudo
            Constraint::Length(2),  // Footer
        ])
        .split(size);

    // Header
    let header = Paragraph::new("Resumo da Auditoria")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    if let Some(data) = &state.audit_data {
        // Layout em duas colunas
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Coluna 1: CPU + RAM
        let col1_text = format!(
            "CPU: {}
Cores: {} | Threads: {}

RAM Total: {}
Modulos:
{}",
            data.cpu.name,
            data.cpu.cores,
            data.cpu.threads,
            data.memory.total,
            data.memory.modules.join("
")
        );
        let col1 = Paragraph::new(col1_text)
            .block(Block::default().title(" Processamento ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(col1, cols[0]);

        // Coluna 2: Discos + GPU + Placa-mae
        let disks_str = data.disks.iter()
            .map(|d| format!("{} - {} - {} - {}% livre", d.model, d.disk_type, d.size, d.free_percent))
            .collect::<Vec<_>>()
            .join("
");

        let col2_text = format!(
            "Discos:
{}

GPU: {}
Placa-mae: {}
BIOS: {}",
            disks_str,
            data.gpu.name,
            data.motherboard.model,
            data.motherboard.bios_version
        );
        let col2 = Paragraph::new(col2_text)
            .block(Block::default().title(" Armazenamento & Video ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(col2, cols[1]);
    } else {
        let msg = Paragraph::new("Nenhum dado de auditoria disponivel. Execute a auditoria primeiro.")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new("D - Modo Detalhado | R - Relatorio | B - Voltar")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}