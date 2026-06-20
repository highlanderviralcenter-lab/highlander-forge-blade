//! Tela summary — exibe resultados da auditoria (CPU, RAM, Discos, GPU, Motherboard)

use crate::app::messages::{AppState, DiskInfo};
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
            Span::styled("FASE 2", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Resumo da Auditoria"),
        ]),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Conteudo principal — duas colunas
    let content = if let Some(ref audit) = state.audit_data {
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);

        // Coluna esquerda: CPU + RAM
        let left_text = build_cpu_ram_text(audit);
        let left = Paragraph::new(left_text)
            .block(Block::default().borders(Borders::ALL).title("Processador e Memoria").border_style(Color::Blue));
        frame.render_widget(left, cols[0]);

        // Coluna direita: Discos + GPU + Motherboard
        let right_text = build_disks_gpu_mb_text(audit);
        let right = Paragraph::new(right_text)
            .block(Block::default().borders(Borders::ALL).title("Armazenamento, Video e Placa-mae").border_style(Color::Blue));
        frame.render_widget(right, cols[1]);

        Paragraph::new("") // placeholder
    } else {
        Paragraph::new(Span::styled(
            "Nenhum dado de auditoria disponivel. Execute a Fase 1 primeiro.",
            Style::default().fg(Color::Yellow),
        ))
    };

    if state.audit_data.is_none() {
        frame.render_widget(content, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Proximo: Enter", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("Voltar: Backspace", Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled("Sair: Q/Esc", Style::default().fg(Color::Gray)),
        ]),
    ]));
    frame.render_widget(footer, chunks[2]);
}

fn build_cpu_ram_text(audit: &crate::app::messages::AuditData) -> Text<'_> {
    let mut lines: Vec<Line> = Vec::new();

    // CPU
    lines.push(Line::from(vec![Span::styled("CPU", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref cpu) = audit.cpu {
        lines.push(Line::from(vec![Span::raw(format!("  Nome:      {} {}", cpu.manufacturer, cpu.name))]));
        lines.push(Line::from(vec![Span::raw(format!("  Nucleos:   {} ({} threads)", cpu.cores, cpu.threads))]));
        lines.push(Line::from(vec![Span::raw(format!("  Clock Max: {} MHz", cpu.max_speed_mhz))]));
        lines.push(Line::from(vec![Span::raw(format!("  Arquitetura: {}", cpu.architecture))]));
    } else {
        lines.push(Line::from(vec![Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))]));
    }
    lines.push(Line::raw(""));

    // RAM
    lines.push(Line::from(vec![Span::styled("MEMORIA RAM", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref mem) = audit.memory {
        let total_gb = mem.total_bytes as f64 / 1_073_741_824.0;
        lines.push(Line::from(vec![Span::raw(format!("  Total:     {:.2} GB", total_gb))]));
        for (i, module) in mem.modules.iter().enumerate() {
            let cap_gb = module.capacity_bytes as f64 / 1_073_741_824.0;
            lines.push(Line::from(vec![Span::raw(format!(
                "  Modulo {}:  {:.1} GB @ {} MHz — {}",
                i + 1, cap_gb, module.speed_mhz, module.manufacturer
            ))]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))]));
    }

    Text::from(lines)
}

fn build_disks_gpu_mb_text(audit: &crate::app::messages::AuditData) -> Text<'_> {
    let mut lines: Vec<Line> = Vec::new();

    // Discos
    lines.push(Line::from(vec![Span::styled("ARMAZENAMENTO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if audit.disks.is_empty() {
        lines.push(Line::from(vec![Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))]));
    } else {
        for disk in &audit.disks {
            let total_gb = disk.total_bytes as f64 / 1_073_741_824.0;
            let free_gb = disk.free_bytes as f64 / 1_073_741_824.0;
            let used_gb = disk.used_bytes as f64 / 1_073_741_824.0;
            let tipo = detect_disk_type(disk);
            let color = if free_gb < 10.0 { Color::Red } else { Color::White };
            lines.push(Line::from(vec![
                Span::raw(format!("  [{}] ", tipo)),
                Span::styled(
                    format!("{}  {:.0}GB / {:.0}GB livre ({:.1}%)", disk.volume_name, used_gb, free_gb, disk.percent_free),
                    Style::default().fg(color),
                ),
            ]));
        }
    }
    lines.push(Line::raw(""));

    // GPU
    lines.push(Line::from(vec![Span::styled("PLACA DE VIDEO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if audit.gpus.is_empty() {
        lines.push(Line::from(vec![Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))]));
    } else {
        for gpu in &audit.gpus {
            let vram_mb = gpu.adapter_ram_bytes / 1_048_576;
            lines.push(Line::from(vec![Span::raw(format!("  {} {}", gpu.manufacturer, gpu.name))]));
            lines.push(Line::from(vec![Span::raw(format!("  VRAM: {} MB | Driver: {} | {}", vram_mb, gpu.driver_version, gpu.resolution))]));
        }
    }
    lines.push(Line::raw(""));

    // Motherboard
    lines.push(Line::from(vec![Span::styled("PLACA-MAE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    if let Some(ref mb) = audit.motherboard {
        lines.push(Line::from(vec![Span::raw(format!("  Fabricante: {}", mb.manufacturer))]));
        lines.push(Line::from(vec![Span::raw(format!("  Modelo:     {}", mb.product))]));
        lines.push(Line::from(vec![Span::raw(format!("  Versao:     {}", mb.version))]));
        lines.push(Line::from(vec![Span::raw(format!("  BIOS:       {} {}", mb.bios_vendor, mb.bios_version))]));
    } else {
        lines.push(Line::from(vec![Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))]));
    }

    Text::from(lines)
}

/// Detecta tipo de disco baseado em heurísticas
fn detect_disk_type(disk: &DiskInfo) -> &'static str {
    let name_upper = disk.volume_name.to_uppercase();
    if name_upper.contains("NVME") || name_upper.contains("NVMe") {
        "NVMe"
    } else if name_upper.contains("SSD") || name_upper.contains("SOLID") {
        "SSD"
    } else if disk.total_bytes > 0 && disk.total_bytes < 128_849_018_880 {
        // < 120GB geralmente é HDD antigo ou SSD pequeno
        if name_upper.contains("HDD") || name_upper.contains("WD") || name_upper.contains("SEAGATE") {
            "HDD"
        } else {
            "SSD?"
        }
    } else {
        "HDD?"
    }
}
