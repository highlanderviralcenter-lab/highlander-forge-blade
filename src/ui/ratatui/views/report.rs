//! Tela de relatorio — exibe dados da auditoria em formato texto

use crate::app::messages::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
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
            Span::styled("RELATORIO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Dados Coletados na Auditoria"),
        ]),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Conteudo
    let text = if let Some(ref audit) = state.audit_data {
        build_report_text(audit)
    } else {
        Text::from(vec![
            Line::from(Span::styled(
                "Nenhum dado de auditoria disponivel.",
                Style::default().fg(Color::Yellow),
            )),
        ])
    };

    let content = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Relatorio").border_style(Color::Blue))
        .scroll((state.selected_menu_item as u16, 0));
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

fn build_report_text(audit: &crate::app::messages::AuditData) -> Text<'_> {
    let mut lines: Vec<Line> = Vec::new();

    // CPU
    lines.push(Line::from(vec![Span::styled("=== PROCESSADOR ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref cpu) = audit.cpu {
        lines.push(Line::from(format!("  Fabricante: {}", cpu.manufacturer)));
        lines.push(Line::from(format!("  Modelo:     {}", cpu.name)));
        lines.push(Line::from(format!("  Nucleos:    {} ({} threads)", cpu.cores, cpu.threads)));
        lines.push(Line::from(format!("  Clock:      {} MHz", cpu.max_speed_mhz)));
        lines.push(Line::from(format!("  Arquitetura:{}", cpu.architecture)));
        lines.push(Line::from(format!("  Socket:     {}", cpu.socket)));
    } else {
        lines.push(Line::from("  N/A"));
    }
    lines.push(Line::raw(""));

    // RAM
    lines.push(Line::from(vec![Span::styled("=== MEMORIA RAM ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref mem) = audit.memory {
        let total_gb = mem.total_bytes as f64 / 1_073_741_824.0;
        lines.push(Line::from(format!("  Total: {:.2} GB", total_gb)));
        for (i, m) in mem.modules.iter().enumerate() {
            let cap_gb = m.capacity_bytes as f64 / 1_073_741_824.0;
            lines.push(Line::from(format!(
                "  Slot {}: {:.1} GB @ {} MHz — {}",
                i + 1, cap_gb, m.speed_mhz, m.manufacturer
            )));
        }
    } else {
        lines.push(Line::from("  N/A"));
    }
    lines.push(Line::raw(""));

    // Discos
    lines.push(Line::from(vec![Span::styled("=== ARMAZENAMENTO ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if audit.disks.is_empty() {
        lines.push(Line::from("  N/A"));
    } else {
        for d in &audit.disks {
            let total_gb = d.total_bytes as f64 / 1_073_741_824.0;
            let free_gb = d.free_bytes as f64 / 1_073_741_824.0;
            lines.push(Line::from(format!(
                "  {} ({}) — {:.0} GB total, {:.1} GB livre ({:.1}%)",
                d.volume_name, d.filesystem, total_gb, free_gb, d.percent_free
            )));
        }
    }
    lines.push(Line::raw(""));

    // GPU
    lines.push(Line::from(vec![Span::styled("=== PLACA DE VIDEO ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if audit.gpus.is_empty() {
        lines.push(Line::from("  N/A"));
    } else {
        for g in &audit.gpus {
            lines.push(Line::from(format!("  {} {}", g.manufacturer, g.name)));
            lines.push(Line::from(format!("  VRAM: {} MB | Driver: {} | {}", g.adapter_ram_bytes / 1_048_576, g.driver_version, g.resolution)));
        }
    }
    lines.push(Line::raw(""));

    // Motherboard
    lines.push(Line::from(vec![Span::styled("=== PLACA-MAE ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref mb) = audit.motherboard {
        lines.push(Line::from(format!("  Fabricante: {}", mb.manufacturer)));
        lines.push(Line::from(format!("  Modelo:     {}", mb.product)));
        lines.push(Line::from(format!("  Versao:     {}", mb.version)));
        lines.push(Line::from(format!("  Serial:     {}", mb.serial_number)));
        lines.push(Line::from(format!("  BIOS:       {} {}", mb.bios_vendor, mb.bios_version)));
    } else {
        lines.push(Line::from("  N/A"));
    }
    lines.push(Line::raw(""));

    // Temperaturas
    if !audit.temperatures.is_empty() {
        lines.push(Line::from(vec![Span::styled("=== TEMPERATURAS ===", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
        for t in &audit.temperatures {
            let color = if t.celsius > 80.0 { Color::Red } else if t.celsius > 60.0 { Color::Yellow } else { Color::Green };
            lines.push(Line::from(vec![
                Span::raw(format!("  {:20} ", t.zone)),
                Span::styled(format!("{:.1} C", t.celsius), Style::default().fg(color)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    // Rodape
    lines.push(Line::from(vec![Span::styled(
        "=== FIM DO RELATORIO ===",
        Style::default().fg(Color::Gray),
    )]));

    Text::from(lines)
}
