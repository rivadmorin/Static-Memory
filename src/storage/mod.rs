pub mod db;

use crate::models::LogEntry;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub enum StorageCommand {
    Store(LogEntry),
    QueryHistory { sender: mpsc::Sender<Vec<LogEntry>> },
    GetAnalytics { sender: mpsc::Sender<AnalyticsData> },
    ExportCsv {
        target_path: String,
        sender: mpsc::Sender<Result<(), String>>,
    },
    ExportTxt {
        target_path: String,
        sender: mpsc::Sender<Result<(), String>>,
    },
    PurgeAll {
        sender: mpsc::Sender<Result<(), String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsData {
    pub top_apps: Vec<(String, usize)>,
    pub hourly_activity: Vec<(u32, usize)>,
    pub total_words: usize,
    pub daily_activity_trend: Vec<(String, usize)>,
    pub most_active_window_titles: Vec<(String, usize)>,
    pub total_characters: usize,
    pub most_productive_hour: u32,
    pub average_words_per_entry: f64,
    pub longest_active_session: usize,
    pub busiest_day_of_week: String,
    pub most_used_app_heatmap: Vec<(String, u32, usize)>,
    pub recent_apps: Vec<String>,
    pub total_entries: usize,
}
