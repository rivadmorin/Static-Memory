pub mod db;

use crate::models::LogEntry;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

pub enum StorageCommand {
    Store(LogEntry),
    QueryHistory { sender: mpsc::Sender<Vec<LogEntry>> },
    GetAnalytics { sender: mpsc::Sender<AnalyticsData> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsData {
    pub top_apps: Vec<(String, usize)>,
    pub hourly_activity: Vec<(u32, usize)>,
    pub total_words: usize,
}
