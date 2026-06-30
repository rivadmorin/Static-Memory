pub mod db;

use crate::models::LogEntry;
use tokio::sync::mpsc;

pub enum StorageCommand {
    Store(LogEntry),
    QueryHistory { sender: mpsc::Sender<Vec<LogEntry>> },
}
