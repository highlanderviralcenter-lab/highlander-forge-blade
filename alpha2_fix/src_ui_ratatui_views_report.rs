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
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(size);

    let header = Paragraph::new("Relatorio Completo")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    if let Some(data) = &state.audit_data {
        let mut lines = vec![];
        lines.push(Line::from(vec![Span::styled("=== INFORMACOES DO SISTEMA ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        lines.push(Line::from(format!("Machine ID: {}", data.machine_id)));
        lines.push(Line::from(format!("Data: {}", data.timestamp)));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled("=== PROCESSADOR ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        lines.push(Line::from(format!("Modelo: {}", data.cpu.name)));
        lines.push(Line::from(format!("Cores: {} | Threads: {}", data.cpu.cores, data.cpu.threads)));
        lines.push(Line::from(format!("Frequencia: {} MHz", data.cpu.frequency)));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled("=== MEMORIA ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        lines.push(Line::from(format!("Total: {}", data.memory.total)));
        for m in &data.memory.modules {
            lines.push(Line::from(format!("  - {}", m)));
        }
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled("=== DISCOS ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        for d in &data.disks {
            lines.push(Line::from(format!(
                "{} | {} | {} | {} livre",
                d.model, d.disk_type, d.size, d.free_percent
            )));
        }
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled("=== GPU ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        lines.push(Line::from(format!("{} | Driver: {}", data.gpu.name, data.gpu.driver)));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled("=== PLACA-MAE ===", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]));
        lines.push(Line::from(format!("{} | BIOS: {}", data.motherboard.model, data.motherboard.bios_version)));

        let report = Paragraph::new(lines)
            .block(Block::default().title(" Relatorio ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .scroll((0, 0));
        frame.render_widget(report, chunks[1]);
    } else {
        let msg = Paragraph::new("Execute a auditoria primeiro para gerar o relatorio.")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[1]);
    }

    let footer = Paragraph::new("B - Voltar ao Menu")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}