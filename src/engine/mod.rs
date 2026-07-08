pub mod buffer;
#[cfg(test)]
mod tests;
use crate::engine::buffer::TextBuffer;
use crate::models::{Config, LogEntry};
use crate::os::{OSInterface, WindowInfo};
use crate::storage::StorageCommand;
use chrono::{DateTime, Utc};
use smol_str::SmolStr;
use tokio::sync::mpsc;

pub struct Engine<O: OSInterface> {
    config: Config,
    os: O,
    current_window: Option<WindowInfo>,
    buffer: TextBuffer,
    storage_tx: mpsc::Sender<StorageCommand>,
    last_input_time: DateTime<Utc>,
    is_idle: bool,
}

impl<O: OSInterface> Engine<O> {
    pub fn new(config: Config, os: O, storage_tx: mpsc::Sender<StorageCommand>) -> Self {
        Self {
            config,
            os,
            current_window: None,
            buffer: TextBuffer::new(),
            storage_tx,
            last_input_time: Utc::now(),
            is_idle: false,
        }
    }

    pub fn is_excluded(&self, window: &WindowInfo) -> bool {
        if let Ok(privacy) = self.config.privacy.read() {
            // Check process name
            if privacy
                .exclude_processes
                .iter()
                .any(|p| p.as_str() == window.process_name)
            {
                return true;
            }
            // Check window title
            if privacy
                .exclude_titles
                .iter()
                .any(|t| window.title.contains(t))
            {
                return true;
            }
        }
        false
    }

    pub async fn handle_key(&mut self, key: char) {
        let now = Utc::now();

        if self.is_idle {
            let idle_duration = now
                .signed_duration_since(self.last_input_time)
                .num_seconds();
            self.log_event(format!(
                "[IDLE_RETURN] [AFK_DURATION: {} seconds]",
                idle_duration
            ))
            .await;
            self.is_idle = false;
        }

        self.last_input_time = now;
        self.check_window_switch().await;

        if let Some(window) = &self.current_window {
            if self.is_excluded(window) {
                self.buffer.clear();
                return;
            }
        }

        // Basic handling of special keys could be expanded
        if self.buffer.len() >= self.config.engine.flush_threshold_chars {
            self.flush().await;
        }

        match key {
            '\u{8}' => self.buffer.backspace(), // Backspace
            _ => self.buffer.push(key),
        }
    }

    pub async fn check_idle(&mut self) {
        if !self.is_idle {
            let now = Utc::now();
            let idle_threshold = self.config.engine.idle_threshold_seconds as i64;
            if now
                .signed_duration_since(self.last_input_time)
                .num_seconds()
                >= idle_threshold
            {
                self.is_idle = true;
                self.flush().await; // Flush before going idle
                self.log_event("[IDLE_START]".to_string()).await;
            }
        }
    }

    pub fn is_idle(&self) -> bool {
        self.is_idle
    }

    async fn log_event(&self, message: String) {
        if let Some(window) = &self.current_window {
            let entry = LogEntry {
                timestamp: Utc::now(),
                app_name: SmolStr::new(&window.process_name),
                window_title: SmolStr::new(&window.title),
                buffer: message,
            };
            let _ = self.storage_tx.send(StorageCommand::Store(entry)).await;
        }
    }

    pub async fn check_window_switch(&mut self) {
        let new_window = self.os.get_active_window();

        let switch = match (&self.current_window, &new_window) {
            (Some(curr), Some(new)) => {
                curr.title != new.title || curr.process_name != new.process_name
            }
            (None, Some(_)) => true,
            _ => false,
        };

        if switch {
            self.flush().await;
            self.current_window = new_window;
        }
    }

    pub async fn flush(&mut self) {
        if !self.buffer.is_empty() {
            if let Some(window) = &self.current_window {
                let entry = LogEntry {
                    timestamp: Utc::now(),
                    app_name: SmolStr::new(&window.process_name),
                    window_title: SmolStr::new(&window.title),
                    buffer: self.buffer.get_string(),
                };
                let _ = self.storage_tx.send(StorageCommand::Store(entry)).await;
            }
            self.buffer.clear();
        }
    }
}
