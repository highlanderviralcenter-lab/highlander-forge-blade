//! Loop principal ratatui com canal mpsc tokio

mod app;
pub mod views;
pub mod widgets;

use crate::app::messages::AppMsg;
use crate::app::state::AppState;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Sender};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = channel::<AppMsg>(256);
    let mut state = AppState::default();

    // Task de eventos de teclado
    let tx_input = tx.clone();
    tokio::spawn(async move {
        loop {
            match event::poll(Duration::from_millis(50)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            let msg = match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => AppMsg::Shutdown,
                                KeyCode::Up => AppMsg::NavigateUp,
                                KeyCode::Down => AppMsg::NavigateDown,
                                KeyCode::Enter => AppMsg::Select,
                                KeyCode::Backspace => AppMsg::Back,
                                _ => continue,
                            };
                            let _ = tx_input.send(msg).await;
                        }
                    }
                }
                Ok(false) => {}
                Err(_) => break,
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    // Task de heartbeat
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let _ = tx_tick.send(AppMsg::Tick).await;
        }
    });

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    let result = loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMsg::Shutdown => break Ok(()),
                _ => state.update(msg),
            }
        }

        if last_tick.elapsed() >= tick_rate {
            state.update(AppMsg::Tick);
            last_tick = Instant::now();
        }

        if let Err(e) = terminal.draw(|frame| {
            views::render(frame, &mut state);
        }) {
            break Err(Box::new(e) as Box<dyn std::error::Error>);
        }

        tokio::time::sleep(Duration::from_millis(16)).await;
    };

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;

    result
}
