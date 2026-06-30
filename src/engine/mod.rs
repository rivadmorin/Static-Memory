pub mod buffer;
#[cfg(test)]
mod tests;
use crate::models::{LogEntry, Config};
use crate::os::{WindowInfo, OSInterface};
use crate::storage::StorageCommand;
use crate::engine::buffer::TextBuffer;
use tokio::sync::mpsc;
use chrono::Utc;
use smol_str::SmolStr;

pub struct Engine<O: OSInterface> {
    config: Config,
    os: O,
    current_window: Option<WindowInfo>,
    buffer: TextBuffer,
    storage_tx: mpsc::Sender<StorageCommand>,
}

impl<O: OSInterface> Engine<O> {
    pub fn new(config: Config, os: O, storage_tx: mpsc::Sender<StorageCommand>) -> Self {
        Self {
            config,
            os,
            current_window: None,
            buffer: TextBuffer::new(),
            storage_tx,
        }
    }

    pub fn is_excluded(&self, window: &WindowInfo) -> bool {
        // Check process name
        if self.config.privacy.exclude_processes.iter().any(|p| p.as_str() == window.process_name) {
            return true;
        }
        // Check window title
        if self.config.privacy.exclude_titles.iter().any(|t| window.title.contains(t)) {
            return true;
        }
        false
    }

    pub async fn handle_key(&mut self, key: char) {
        self.check_window_switch().await;

        if let Some(window) = &self.current_window {
            if self.is_excluded(window) {
                self.buffer.clear();
                return;
            }
        }

        // Basic handling of special keys could be expanded
        match key {
            '\u{8}' => self.buffer.backspace(), // Backspace
            _ => self.buffer.push(key),
        }
    }

    pub async fn check_window_switch(&mut self) {
        let new_window = self.os.get_active_window();

        let switch = match (&self.current_window, &new_window) {
            (Some(curr), Some(new)) => curr.title != new.title || curr.process_name != new.process_name,
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
