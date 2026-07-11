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
    pub flush_threshold_chars: usize,
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

pub fn get_default_data_dir() -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    {
        let mut path =
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
        path.push(".local");
        path.push("share");
        path.push("static-memory");
        path
    }
    #[cfg(windows)]
    {
        let mut path = std::path::PathBuf::from(
            std::env::var("APPDATA").unwrap_or_else(|_| String::from("C:\\ProgramData")),
        );
        path.push("Static-Memory");
        path
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        std::path::PathBuf::from(".")
    }
}

pub fn get_default_config_path() -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    {
        let mut path =
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~")));
        path.push(".config");
        path.push("static-memory");
        path.push("config.toml");
        path
    }
    #[cfg(windows)]
    {
        let mut path = get_default_data_dir();
        path.push("config.toml");
        path
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        std::path::PathBuf::from("config.toml")
    }
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
                flush_threshold_chars: 512,
            },
            privacy: Arc::new(RwLock::new(PrivacyConfig {
                exclude_processes: vec!["bitwarden.exe".into(), "keepassxc".into()],
                exclude_titles: vec!["Incognito".to_string(), "Private Browsing".to_string()],
            })),
            linux: None,
        }
    }
}
