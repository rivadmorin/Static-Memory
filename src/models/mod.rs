use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub app_name: SmolStr,
    pub window_title: SmolStr,
    pub buffer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub privacy: PrivacyConfig,
    pub linux: Option<LinuxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
    pub rotation_size_mb: u64,
    pub rotation_interval_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub exclude_processes: Vec<SmolStr>,
    pub exclude_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfig {
    pub keyboard_device_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                db_path: "activity_log.db".to_string(),
                rotation_size_mb: 50,
                rotation_interval_days: 30,
            },
            privacy: PrivacyConfig {
                exclude_processes: vec!["bitwarden.exe".into(), "keepassxc".into()],
                exclude_titles: vec!["Incognito".to_string(), "Private Browsing".to_string()],
            },
            linux: None,
        }
    }
}
