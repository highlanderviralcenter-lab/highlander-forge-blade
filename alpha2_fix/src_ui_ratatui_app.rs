use ratatui::Frame;
use tokio::sync::mpsc::Sender;

use crate::app::messages::{Message, Screen, LogEntry};
use crate::app::state::AppState;
use crate::ui::ratatui::views::{
    menu, progress, summary, confirm, report, logs, detailed
};

pub struct App {
    pub state: AppState,
    pub tx: Sender<Message>,
    pub should_quit: bool,
}

impl App {
    pub fn new(tx: Sender<Message>) -> Self {
        Self {
            state: AppState::default(),
            tx,
            should_quit: false,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self.state.current_screen {
            Screen::Menu => menu::draw(frame, &self.state),
            Screen::Progress => progress::draw(frame, &self.state),
            Screen::Summary => summary::draw(frame, &self.state),
            Screen::Confirm => confirm::draw(frame, &self.state),
            Screen::Report => report::draw(frame, &self.state),
            Screen::Logs => logs::draw(frame, &self.state),
            Screen::Detailed => detailed::draw(frame, &self.state),
        }
    }

    pub async fn on_key(&mut self, c: char) {
        match self.state.current_screen {
            Screen::Menu => {
                match c {
                    '1' => self.start_audit().await,
                    '2' => self.start_cleanup().await,
                    '3' => self.state.current_screen = Screen::Report,
                    '4' => self.state.current_screen = Screen::Logs,
                    '5' => self.state.current_screen = Screen::Confirm,
                    'q' | 'Q' => self.should_quit = true,
                    _ => {}
                }
            }
            Screen::Summary => {
                if c == 'd' || c == 'D' {
                    self.state.current_screen = Screen::Detailed;
                } else if c == 'r' || c == 'R' {
                    self.state.current_screen = Screen::Report;
                } else if c == 'b' || c == 'B' {
                    self.state.current_screen = Screen::Menu;
                }
            }
            Screen::Detailed => {
                if c == 'b' || c == 'B' || c == 'd' || c == 'D' {
                    self.state.current_screen = Screen::Summary;
                }
            }
            Screen::Confirm => {
                match c {
                    's' | 'S' | 'y' | 'Y' => {
                        let _ = self.tx.send(Message::ConfirmReboot(true)).await;
                        self.state.current_screen = Screen::Menu;
                    }
                    'n' | 'N' => {
                        let _ = self.tx.send(Message::ConfirmReboot(false)).await;
                        self.state.current_screen = Screen::Menu;
                    }
                    _ => {}
                }
            }
            Screen::Report | Screen::Logs | Screen::Progress => {
                if c == 'b' || c == 'B' || c == 'q' || c == 'Q' {
                    self.state.current_screen = Screen::Menu;
                }
            }
        }
    }

    pub async fn on_enter(&mut self) {
        match self.state.current_screen {
            Screen::Menu => {
                match self.state.menu_selected {
                    0 => self.start_audit().await,
                    1 => self.start_cleanup().await,
                    2 => self.state.current_screen = Screen::Report,
                    3 => self.state.current_screen = Screen::Logs,
                    4 => self.state.current_screen = Screen::Confirm,
                    10 => self.should_quit = true,
                    _ => {}
                }
            }
            Screen::Confirm => {
                // Enter no confirm seleciona Sim (default)
                let _ = self.tx.send(Message::ConfirmReboot(true)).await;
                self.state.current_screen = Screen::Menu;
            }
            _ => {}
        }
    }

    pub async fn on_up(&mut self) {
        if self.state.current_screen == Screen::Menu {
            if self.state.menu_selected > 0 {
                self.state.menu_selected -= 1;
            }
        }
    }

    pub async fn on_down(&mut self) {
        if self.state.current_screen == Screen::Menu {
            if self.state.menu_selected < 10 {
                self.state.menu_selected += 1;
            }
        }
    }

    pub async fn on_left(&mut self) {
        if self.state.current_screen == Screen::Confirm {
            self.state.confirm_selected = false; // Nao
        }
    }

    pub async fn on_right(&mut self) {
        if self.state.current_screen == Screen::Confirm {
            self.state.confirm_selected = true; // Sim
        }
    }

    pub async fn on_esc(&mut self) {
        match self.state.current_screen {
            Screen::Menu => self.should_quit = true,
            _ => self.state.current_screen = Screen::Menu,
        }
    }

    async fn start_audit(&mut self) {
        self.state.current_screen = Screen::Progress;
        self.state.progress_percent = 0;
        self.state.current_phase = "Iniciando auditoria...".to_string();
        self.state.logs.clear();
        self.state.logs.push(LogEntry::info("Iniciando coleta de hardware..."));
        let _ = self.tx.send(Message::StartAudit).await;
    }

    async fn start_cleanup(&mut self) {
        self.state.current_screen = Screen::Progress;
        self.state.progress_percent = 0;
        self.state.current_phase = "Iniciando limpeza...".to_string();
        self.state.logs.clear();
        self.state.logs.push(LogEntry::info("Iniciando limpeza do sistema..."));
        let _ = self.tx.send(Message::StartCleanup).await;
    }
}