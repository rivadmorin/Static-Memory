use smol_str::SmolStr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub timestamp: DateTime<Utc>,
    pub app_name: SmolStr,
    pub window_title: SmolStr,
    pub buffer: String, // Buffer can grow, but we'll manage it
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyEvent),
    Window(WindowEvent),
    System(SystemMetrics),
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: KeyType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum KeyType {
    Char(char),
    Backspace,
    Enter,
    Tab,
    Space,
    Modifier(SmolStr), // e.g., "[CTRL+C]"
    Other,
}

#[derive(Debug, Clone)]
pub struct WindowEvent {
    pub app_name: SmolStr,
    pub window_title: SmolStr,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage_mb: u64,
    pub timestamp: DateTime<Utc>,
}
