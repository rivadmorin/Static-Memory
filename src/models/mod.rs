use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub app_name: SmolStr,
    pub window_title: SmolStr,
    pub buffer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub engine: EngineConfig,
    #[serde(skip)]
    pub privacy: Arc<RwLock<PrivacyConfig>>,
    pub linux: Option<LinuxConfig>,
}

fn default_privacy() -> Arc<RwLock<PrivacyConfig>> {
    Arc::new(RwLock::new(PrivacyConfig {
        exclude_processes: vec!["bitwarden.exe".into(), "keepassxc".into()],
        exclude_titles: vec!["Incognito".to_string(), "Private Browsing".to_string()],
    }))
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            engine: self.engine.clone(),
            privacy: Arc::clone(&self.privacy),
            linux: self.linux.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
    pub rotation_size_mb: u64,
    pub rotation_interval_days: u32,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub idle_threshold_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyConfig {
    pub exclude_processes: Vec<SmolStr>,
    pub exclude_titles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub storage: StorageConfig,
    pub engine: EngineConfig,
    pub privacy: PrivacyConfig,
    pub linux: Option<LinuxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfig {
    pub keyboard_device_path: Option<String>,
}

#[allow(dead_code)]
impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                db_path: "activity_log.db".to_string(),
                rotation_size_mb: 50,
                rotation_interval_days: 30,
                retention_days: 7,
            },
            engine: EngineConfig {
                idle_threshold_seconds: 180,
            },
            privacy: Arc::new(RwLock::new(PrivacyConfig {
                exclude_processes: vec!["bitwarden.exe".into(), "keepassxc".into()],
                exclude_titles: vec!["Incognito".to_string(), "Private Browsing".to_string()],
            })),
            linux: None,
        }
    }
}
