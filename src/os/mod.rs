pub mod windows;
pub mod linux;
pub mod macos;
pub mod ipc;

use smol_str::SmolStr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub process_name: SmolStr,
    pub title: SmolStr,
}

pub trait OSInterface {
    fn get_active_window(&self) -> Option<WindowInfo>;
}
pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".local/share/static-memory")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join("Library/Application Support/Static-Memory")
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("Static-Memory")
    }
}

pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".config/static-memory")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join("Library/Application Support/Static-Memory")
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("Static-Memory")
    }
}
