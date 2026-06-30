mod models;
mod storage;
mod os;
mod collector;
mod engine;
mod ui;

use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Duration;
use crate::storage::Storage;
use crate::engine::LoggerEngine;
use crate::ui::{TuiState, draw_ui};
use crate::models::InputEvent;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Storage
    let storage = Arc::new(Storage::new("activity_log.db")?);

    // 2. Setup Communication Channels
    let (tx, rx) = mpsc::channel::<InputEvent>(100);

    // 3. Start Collectors
    let tx_win = tx.clone();
    tokio::spawn(async move {
        collector::window::run_window_collector(tx_win).await;
    });

    let tx_sys = tx.clone();
    tokio::spawn(async move {
        collector::system::run_system_collector(tx_sys).await;
    });

    let tx_kb = tx.clone();
    tokio::spawn(async move {
        collector::keyboard::run_keyboard_collector(tx_kb).await;
    });

    // 4. Start Logger Engine
    let engine_storage = storage.clone();
    let mut engine = LoggerEngine::new(engine_storage, rx);
    tokio::spawn(async move {
        engine.run().await;
    });

    // 5. Setup TUI
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new();

    // 6. Main TUI Event Loop
    loop {
        terminal.draw(|f| draw_ui(f, &state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => {
                        state.current_tab = (state.current_tab + 1) % 3;
                    }
                    KeyCode::Char('c') if state.current_tab == 2 => {
                        let _ = storage.clear_logs();
                    }
                    _ => {}
                }
            }
        }

        // Update timeline in real-time (simplified)
        if let Ok(logs) = storage.get_recent_logs(20) {
            state.timeline.clear();
            for log in logs {
                state.timeline.push_back(log);
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
