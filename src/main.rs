pub mod collector;
pub mod engine;
pub mod models;
pub mod os;
pub mod storage;
pub mod ui;

use crate::engine::Engine;
use crate::models::{Config, ConfigFile};
#[cfg(target_os = "linux")]
use crate::os::linux::LinuxOS;
#[cfg(windows)]
use crate::os::windows::WindowsOS;
use crate::storage::db::start_storage_thread;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

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
    let args: Vec<String> = std::env::args().collect();
    let is_daemon = args.iter().any(|arg| arg == "--daemon");
    let is_export_csv = args.iter().position(|arg| arg == "--export-csv").and_then(|i| args.get(i + 1).cloned());
    let is_export_txt = args.iter().position(|arg| arg == "--export-txt").and_then(|i| args.get(i + 1).cloned());
    let is_purge = args.iter().any(|arg| arg == "--purge");
    let is_export_json = args.iter().position(|arg| arg == "--export-json").and_then(|i| args.get(i + 1).cloned());
    let is_search = args.iter().position(|arg| arg == "--search").and_then(|i| args.get(i + 1).cloned());
    let is_delete_app = args.iter().position(|arg| arg == "--delete-app").and_then(|i| args.get(i + 1).cloned());
    let is_top_apps = args.iter().any(|arg| arg == "--top-apps");
    let is_total_words = args.iter().any(|arg| arg == "--total-words");
    let is_list_apps = args.iter().any(|arg| arg == "--list-apps");
    let is_count_entries = args.iter().any(|arg| arg == "--count-entries");
    let is_recent_logs = args.iter().any(|arg| arg == "--recent-logs");
    let is_busiest_day = args.iter().any(|arg| arg == "--busiest-day");
    let is_active_hours = args.iter().any(|arg| arg == "--active-hours");













    let mut config = Config::default();

    // Attempt to load from default config path to respect user's configured db_path
    let config_path = crate::models::get_default_config_path();
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(file_config) = toml::from_str::<ConfigFile>(&content) {
            config.storage = file_config.storage;
        }
    } else {
        // If config file doesn't exist, use default XDG path for db
        let default_dir = crate::models::get_default_data_dir();
        std::fs::create_dir_all(&default_dir).unwrap_or_default();
        config.storage.db_path = default_dir.join("activity_log.db").to_str().unwrap_or("activity_log.db").to_string();
    }

    if let Some(path) = is_export_csv {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                if let Err(e) = db.export_to_csv(&path) {
                    eprintln!("Failed to export CSV: {}", e);
                } else {
                    println!("Exported to CSV: {}", path);
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if let Some(path) = is_export_txt {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                if let Err(e) = db.export_to_txt(&path) {
                    eprintln!("Failed to export TXT: {}", e);
                } else {
                    println!("Exported to TXT: {}", path);
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_purge {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(mut db) => {
                if let Err(e) = db.purge_all_data() {
                    eprintln!("Failed to purge data: {}", e);
                } else {
                    println!("Purged all data");
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }


    if let Some(path) = is_export_json {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                if let Err(e) = db.export_to_json(&path) {
                    eprintln!("Failed to export JSON: {}", e);
                } else {
                    println!("Exported to JSON: {}", path);
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if let Some(keyword) = is_search {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.search_logs(&keyword) {
                    Ok(results) => {
                        for (timestamp, app_name, window_title, buffer) in results {
                            println!("[{}] {} ({}): {}", timestamp, app_name, window_title, buffer);
                        }
                    }
                    Err(e) => eprintln!("Search failed: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if let Some(app_name) = is_delete_app {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.delete_app_logs(&app_name) {
                    Ok(count) => println!("Deleted {} logs for app '{}'", count, app_name),
                    Err(e) => eprintln!("Failed to delete app logs: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_top_apps {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_top_apps() {
                    Ok(apps) => {
                        println!("Top Apps:");
                        for (app, count) in apps {
                            println!("- {}: {}", app, count);
                        }
                    }
                    Err(e) => eprintln!("Failed to get top apps: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_total_words {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_total_words() {
                    Ok(words) => println!("Total Words Typed: {}", words),
                    Err(e) => eprintln!("Failed to get total words: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_list_apps {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.list_unique_apps() {
                    Ok(apps) => {
                        println!("Tracked Apps:");
                        for app in apps {
                            println!("- {}", app);
                        }
                    }
                    Err(e) => eprintln!("Failed to list apps: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_count_entries {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_total_entries() {
                    Ok(count) => println!("Total Log Entries: {}", count),
                    Err(e) => eprintln!("Failed to get entry count: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_recent_logs {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_recent_logs_full() {
                    Ok(results) => {
                        println!("Recent 10 Logs:");
                        for (timestamp, app_name, window_title, buffer) in results {
                            println!("[{}] {} ({}): {}", timestamp, app_name, window_title, buffer);
                        }
                    }
                    Err(e) => eprintln!("Failed to get recent logs: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_busiest_day {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_busiest_day_of_week() {
                    Ok(day) => println!("Busiest Day: {}", day),
                    Err(e) => eprintln!("Failed to get busiest day: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if is_active_hours {
        match crate::storage::db::Database::new(&config.storage.db_path) {
            Ok(db) => {
                match db.get_hourly_activity() {
                    Ok(hours) => {
                        println!("Hourly Activity:");
                        for (hour, count) in hours {
                            if count > 0 {
                                println!("{:02}:00 - {}", hour, count);
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to get hourly activity: {}", e),
                }
            }
            Err(e) => eprintln!("Could not open database: {}", e),
        }
        return Ok(());
    }

    if !is_daemon {
        // TUI will be implemented here. For now, just start normally if no IPC requested
        // in future steps, this section would attach to IPC socket instead of starting the engine.
    }

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

    let engine = Arc::new(tokio::sync::RwLock::new(Engine::new(
        config.clone(),
        os,
        storage_tx,
    )));

    // Start Linux collector if applicable
    #[cfg(target_os = "linux")]
    {
        let engine_clone = Arc::clone(&engine);
        let device_path = config.linux.and_then(|l| l.keyboard_device_path);
        tokio::spawn(async move {
            crate::collector::keyboard::start_evdev_collector(device_path, engine_clone).await;
        });
    }

    #[cfg(windows)]
    {
        let engine_clone = Arc::clone(&engine);
        let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

        unsafe {
            crate::os::windows::GLOBAL_SENDER = Some(tx);

            std::thread::spawn(move || {
                use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowsHookExW, GetMessageW, WH_KEYBOARD_LL};
                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(crate::os::windows::keyboard_proc), 0, 0);

                let mut msg = std::mem::zeroed();
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    let _ = windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    let _ = windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                }
            });
        }

        tokio::spawn(async move {
            while let Some(ch) = rx.recv().await {
                let mut engine_lock = engine_clone.write().await;
                engine_lock.handle_key(ch).await;
            }
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
