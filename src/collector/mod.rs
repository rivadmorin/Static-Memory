use tokio::sync::mpsc;
use crate::models::{InputEvent, KeyEvent, KeyType, WindowEvent, SystemMetrics};
use chrono::Utc;
use std::time::Duration;

pub mod keyboard;
pub mod window;
pub mod system;
