pub mod os;
pub mod models;
pub mod storage;
pub mod engine;
pub mod collector;
pub mod ui;

#[cfg(windows)]
use crate::os::windows::WindowsOS;
#[cfg(target_os = "linux")]
use crate::os::linux::LinuxOS;
use crate::models::{Config, ConfigFile};
use crate::storage::db::start_storage_thread;
use crate::engine::Engine;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::fs;

fn start_config_watcher(config: Config) {
    std::thread::spawn(move || {
        let config_path = "config.toml";
        let mut last_modified = fs::metadata(config_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        loop {
            std::thread::sleep(Duration::from_secs(60));

            if let Ok(metadata) = fs::metadata(config_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_modified {
                        last_modified = modified;
                        if let Ok(content) = fs::read_to_string(config_path) {
                            if let Ok(new_config_file) = toml::from_str::<ConfigFile>(&content) {
                                if let Ok(mut privacy) = config.privacy.write() {
                                    *privacy = new_config_file.privacy;
                                    println!("Config hot-reloaded: Privacy settings updated.");
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal safety
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        eprintln!("\n\rApplication crashed: {:?}", panic_info);
    }));

    let config = Config::default();
    let (storage_tx, storage_rx) = mpsc::channel(100);

    // Start storage
    start_storage_thread(config.clone(), storage_rx);

    // Start config watcher
    start_config_watcher(config.clone());

    // Initialize OS interface
    #[cfg(windows)]
    let os = WindowsOS;
    #[cfg(target_os = "linux")]
    let os = LinuxOS;
    #[cfg(not(any(windows, target_os = "linux")))]
    panic!("Unsupported OS");

    let engine = Arc::new(tokio::sync::RwLock::new(Engine::new(config.clone(), os, storage_tx)));

    // Start Linux collector if applicable
    #[cfg(target_os = "linux")]
    {
        let engine_clone = Arc::clone(&engine);
        let device_path = config.linux.and_then(|l| l.keyboard_device_path);
        tokio::spawn(async move {
            crate::collector::keyboard::start_evdev_collector(device_path, engine_clone).await;
        });
    }

    // Idle check loop
    let engine_clone = Arc::clone(&engine);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let mut engine_lock = engine_clone.write().await;
            engine_lock.check_idle().await;
            // TODO: In a real app, communicate idle status to UI via a channel or shared state
        }
    });

    println!("Static-Memory started. Press Ctrl+C to exit.");

    // This is where collectors would send events to the engine
    // For this boilerplate, we'll just run a simple loop or wait

    tokio::signal::ctrl_c().await?;
    let mut engine_lock = engine.write().await;
    engine_lock.flush().await;

    Ok(())
}
