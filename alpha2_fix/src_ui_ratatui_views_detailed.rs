use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

    let header = Paragraph::new("Modo Detalhado - HWMonitor Style")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    if let Some(data) = &state.audit_data {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Temperaturas
        let mut temp_text = String::from("TEMPERATURAS:

");
        if let Some(t) = data.temperatures.cpu_package { temp_text.push_str(&format!("CPU Package: {} C
", t)); }
        if let Some(t) = data.temperatures.cpu_cores { temp_text.push_str(&format!("CPU Cores: {} C
", t)); }
        if let Some(t) = data.temperatures.gpu { temp_text.push_str(&format!("GPU: {} C
", t)); }
        if let Some(t) = data.temperatures.ssd { temp_text.push_str(&format!("SSD: {} C
", t)); }

        let temp_block = Paragraph::new(temp_text)
            .block(Block::default().title(" Sensores ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(temp_block, cols[0]);

        // Power + Clocks
        let mut power_text = String::from("POTENCIA & CLOCKS:

");
        if let Some(p) = data.power.package { power_text.push_str(&format!("Package: {} W
", p)); }
        if let Some(p) = data.power.ia_cores { power_text.push_str(&format!("IA Cores: {} W
", p)); }
        if let Some(c) = data.clocks.bclk { power_text.push_str(&format!("BCLK: {} MHz
", c)); }
        if let Some(c) = data.clocks.core_max { power_text.push_str(&format!("Core Max: {} MHz
", c)); }
        if let Some(c) = data.clocks.memory { power_text.push_str(&format!("Memory: {} MHz
", c)); }

        let power_block = Paragraph::new(power_text)
            .block(Block::default().title(" Energia ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(power_block, cols[1]);
    } else {
        let msg = Paragraph::new("Dados detalhados nao disponiveis. Execute a auditoria primeiro.")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[1]);
    }

    let footer = Paragraph::new("D/B - Voltar ao Resumo")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}