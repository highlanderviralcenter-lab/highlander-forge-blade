use std::io;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use tokio::sync::mpsc;

use crate::app::messages::{Message, Screen};
use crate::app::state::AppState;
use crate::ui::ratatui::app::App;

pub mod app;
pub mod views;
pub mod widgets;

pub async fn run_tui(mut rx: mpsc::Receiver<Message>, tx: mpsc::Sender<Message>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(tx.clone());

    let tick_rate = std::time::Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| app.draw(f))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            let _ = tx.send(Message::Quit).await;
                        } else {
                            app.on_key('q').await;
                        }
                    }
                    KeyCode::Char(c) => app.on_key(c).await,
                    KeyCode::Up => app.on_up().await,
                    KeyCode::Down => app.on_down().await,
                    KeyCode::Enter => app.on_enter().await,
                    KeyCode::Esc => app.on_esc().await,
                    KeyCode::Left => app.on_left().await,
                    KeyCode::Right => app.on_right().await,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let _ = tx.send(Message::Tick).await;
            last_tick = std::time::Instant::now();
        }

        // Processa mensagens do canal async
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Message::Quit => {
                    break;
                }
                Message::AuditComplete(data) => {
                    app.state.audit_data = Some(data);
                    app.state.current_screen = Screen::Summary;
                    app.state.logs.push(LogEntry::success("Auditoria concluida com sucesso!"));
                }
                Message::AuditProgress { phase, percent, log } => {
                    app.state.progress_percent = percent;
                    app.state.current_phase = phase;
                    if !log.is_empty() {
                        app.state.logs.push(LogEntry::info(log));
                    }
                }
                Message::AuditError(err) => {
                    app.state.logs.push(LogEntry::error(err));
                    app.state.current_screen = Screen::Menu;
                }
                Message::CleanupProgress { item, percent } => {
                    app.state.progress_percent = percent;
                    app.state.current_phase = item;
                }
                Message::CleanupComplete => {
                    app.state.logs.push(LogEntry::success("Limpeza concluida!"));
                    app.state.current_screen = Screen::Summary;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}