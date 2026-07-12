//! Tela de resumo — dados reais da auditoria

use crate::app::messages::AppState;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
    ]).margin(1).split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("FASE 2", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - Resumo da Auditoria"),
    ])).block(Block::default().borders(Borders::BOTTOM).border_style(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(chunks[1]);

    // Coluna esquerda
    let mut left: Vec<Line> = vec![];
    left.push(Line::from(Span::styled("CPU", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    if let Some(ref data) = state.audit_data {
        if let Some(ref cpu) = data.cpu {
            left.push(Line::from(format!("  {}", cpu.name)));
            left.push(Line::from(format!("  {} cores / {} threads @ {} MHz", cpu.cores, cpu.threads, cpu.max_speed_mhz)));
            left.push(Line::from(format!("  Arq: {}  Socket: {}", cpu.architecture, cpu.socket)));
        } else {
            left.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        }
        left.push(Line::from(""));
        left.push(Line::from(Span::styled("MEMORIA RAM", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        if let Some(ref mem) = data.memory {
            let gb = mem.total_bytes as f64 / 1_073_741_824.0;
            left.push(Line::from(format!("  Total: {:.1} GB", gb)));
            for m in &mem.modules {
                let cap = m.capacity_bytes as f64 / 1_073_741_824.0;
                left.push(Line::from(format!("  {} - {:.0}GB {}MHz {}", m.slot, cap, m.speed_mhz, m.manufacturer)));
            }
        } else {
            left.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        }
    } else {
        left.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        left.push(Line::from(""));
        left.push(Line::from(Span::styled("MEMORIA RAM", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        left.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
    }

    frame.render_widget(
        Paragraph::new(left).block(Block::default().title(" Processador e Memoria ").borders(Borders::ALL).border_style(Color::Blue)),
        cols[0]
    );

    // Coluna direita
    let mut right: Vec<Line> = vec![];
    right.push(Line::from(Span::styled("ARMAZENAMENTO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    if let Some(ref data) = state.audit_data {
        if data.disks.is_empty() {
            right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        } else {
            for d in &data.disks {
                let total = d.total_bytes as f64 / 1_073_741_824.0;
                let free  = d.free_bytes  as f64 / 1_073_741_824.0;
                let color = if d.percent_free < 10.0 { Color::Red }
                            else if d.percent_free < 20.0 { Color::Yellow }
                            else { Color::Green };
                right.push(Line::from(vec![
                    Span::raw(format!("  {} {:.0}GB - ", d.device_id, total)),
                    Span::styled(format!("{:.1}% livre ({:.1}GB)", d.percent_free, free), Style::default().fg(color)),
                ]));
            }
        }
        right.push(Line::from(""));
        right.push(Line::from(Span::styled("PLACA DE VIDEO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        if data.gpus.is_empty() {
            right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        } else {
            for g in &data.gpus {
                let vram = g.adapter_ram_bytes / 1_048_576;
                right.push(Line::from(format!("  {}", g.name)));
                right.push(Line::from(format!("  VRAM: {}MB  Driver: {}", vram, g.driver_version)));
            }
        }
        right.push(Line::from(""));
        right.push(Line::from(Span::styled("PLACA-MAE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        if let Some(ref mb) = data.motherboard {
            right.push(Line::from(format!("  {} {}", mb.manufacturer, mb.product)));
            right.push(Line::from(format!("  BIOS: {} ({})", mb.bios_version, mb.bios_date)));
        } else {
            right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        }
    } else {
        right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        right.push(Line::from(""));
        right.push(Line::from(Span::styled("PLACA DE VIDEO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
        right.push(Line::from(""));
        right.push(Line::from(Span::styled("PLACA-MAE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        right.push(Line::from(Span::styled("  (dados nao disponiveis)", Style::default().fg(Color::DarkGray))));
    }

    frame.render_widget(
        Paragraph::new(right).block(Block::default().title(" Armazenamento, Video e Placa-mae ").borders(Borders::ALL).border_style(Color::Blue)),
        cols[1]
    );

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Proximo: Enter", Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("Voltar: Backspace", Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled("Sair: Q/Esc", Style::default().fg(Color::Gray)),
    ]));
    frame.render_widget(footer, chunks[2]);
}
