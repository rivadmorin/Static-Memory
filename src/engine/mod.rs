pub mod buffer;
#[cfg(test)]
mod tests;
use crate::models::{LogEntry, Config};
use crate::os::{WindowInfo, OSInterface};
use crate::storage::StorageCommand;
use crate::engine::buffer::TextBuffer;
use tokio::sync::mpsc;
use chrono::{Utc, DateTime};
use smol_str::SmolStr;
use std::sync::OnceLock;

static CC_REGEX: OnceLock<regex::Regex> = OnceLock::new();
static PWD_REGEX: OnceLock<regex::Regex> = OnceLock::new();

pub fn redact_sensitive_data(buffer: &str) -> String {
    let mut redacted = buffer.to_string();
    // Redact credit card patterns (13-16 digits with optional spaces/dashes)
    let cc_regex = CC_REGEX.get_or_init(|| regex::Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap());
    redacted = cc_regex.replace_all(&redacted, "[REDACTED_CARD]").into_owned();

    // Simple mock pattern for passwords (e.g. after 'password=' or 'pwd=')
    let pwd_regex = PWD_REGEX.get_or_init(|| regex::Regex::new(r"(?i)(password|pwd|token)\s*[:=]\s*\S+").unwrap());
    redacted = pwd_regex.replace_all(&redacted, "$1=[REDACTED]").into_owned();

    redacted
}

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
            if privacy.exclude_processes.iter().any(|p| p.as_str() == window.process_name) {
                return true;
            }
            // Check window title
            if privacy.exclude_titles.iter().any(|t| window.title.contains(t)) {
                return true;
            }
        }
        false
    }

    pub async fn handle_key(&mut self, key: char) {
        let now = Utc::now();

        if self.is_idle {
            let idle_duration = now.signed_duration_since(self.last_input_time).num_seconds();
            self.log_event(format!("[IDLE_RETURN] [AFK_DURATION: {} seconds]", idle_duration)).await;
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
        match key {
            '\u{8}' => self.buffer.backspace(), // Backspace
            _ => self.buffer.push(key),
        }
    }

    pub async fn check_idle(&mut self) {
        if !self.is_idle {
            let now = Utc::now();
            let idle_threshold = self.config.engine.idle_threshold_seconds as i64;
            if now.signed_duration_since(self.last_input_time).num_seconds() >= idle_threshold {
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
            let redacted_message = redact_sensitive_data(&message);

            tracing::info!(
                app_name = %window.process_name,
                window_title = %window.title,
                buffer_length = redacted_message.len(),
                "Flushing text buffer to storage"
            );

            let entry = LogEntry {
                timestamp: Utc::now(),
                app_name: window.process_name.clone(),
                window_title: window.title.clone(),
                buffer: SmolStr::new(message),
            };
            let _ = self.storage_tx.send(StorageCommand::Store(entry)).await;
        }
    }

    pub async fn check_window_switch(&mut self) {
        let new_window = self.os.get_active_window();

        let switch = match (&self.current_window, &new_window) {
            (Some(curr), Some(new)) => curr.title != new.title || curr.process_name != new.process_name,
            (None, Some(_)) => true,
            (Some(_), None) => false, // Keep current window if new is none
            (None, None) => false,
        };

        if switch {
            self.flush().await;
            self.current_window = new_window;
        }
    }

    pub async fn flush(&mut self) {
        if !self.buffer.is_empty() {
            if let Some(window) = &self.current_window {
                let raw_string = self.buffer.get_string();
                let redacted_string = redact_sensitive_data(&raw_string);

                tracing::info!(
                    app_name = %window.process_name,
                    window_title = %window.title,
                    buffer_length = redacted_string.len(),
                    "Flushing text buffer to storage"
                );

                let entry = LogEntry {
                    timestamp: Utc::now(),
                    app_name: window.process_name.clone(),
                    window_title: window.title.clone(),
                    buffer: SmolStr::new(raw_string),
                };
                let _ = self.storage_tx.send(StorageCommand::Store(entry)).await;
            }
            self.buffer.clear();
        }
    }
}
