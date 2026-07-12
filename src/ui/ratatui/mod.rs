//! Loop principal ratatui

pub mod app;
pub mod views;
pub mod widgets;

use crate::app::messages::AppMsg;
use crate::ui::ratatui::app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Instant;
use tokio::sync::mpsc::channel;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = channel::<AppMsg>(256);
    let mut app = App::new(tx.clone());
    let mut last_enter = Instant::now();

    let result = loop {
        // Drena todas as mensagens
        loop {
            match rx.try_recv() {
                Ok(AppMsg::Shutdown) => { app.should_quit = true; break; }
                Ok(msg) => {
                    if matches!(msg, AppMsg::AuditCompleted(_)) {
                        app.on_audit_complete();
                    }
                    app.state.update(msg);
                }
                Err(_) => break,
            }
        }

        if app.should_quit { break Ok(()); }

        if let Err(e) = terminal.draw(|f| app.draw(f)) {
            break Err(Box::new(e) as Box<dyn std::error::Error>);
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
                        KeyCode::Char(c) => app.on_key(c).await,
                        KeyCode::Up => app.on_up(),
                        KeyCode::Down => app.on_down(),
                        KeyCode::Enter => {
                            if last_enter.elapsed().as_millis() > 300 {
                                last_enter = Instant::now();
                                app.on_enter().await;
                            }
                        }
                        KeyCode::Esc => app.on_esc(),
                        KeyCode::Backspace => app.on_backspace(),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit { break Ok(()); }
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}
