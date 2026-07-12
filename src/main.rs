#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
use chrono::Utc;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
fn setup_logging(is_daemon: bool) -> Option<WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // File appender
    let file_appender = tracing_appender::rolling::daily(
        crate::os::get_data_dir().join("logs"),
        "static-memory.log",
    );
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking_file);
    let registry = Registry::default().with(env_filter).with(file_layer);
    #[cfg(target_os = "linux")]
    {
        if is_daemon {
            if let Ok(journald_layer) = tracing_journald::layer() {
                registry.with(journald_layer).init();
                return Some(guard);
            }
        }
    }
    #[cfg(windows)]
    {
        if is_daemon {
            // Use tracing_layer_win_eventlog
            if let Ok(eventlog_layer) =
                tracing_layer_win_eventlog::EventLogLayer::new("Static-Memory")
            {
                registry.with(eventlog_layer).init();
                return Some(guard);
            }
        }
    }
    if !is_daemon {
        let stdout_layer = tracing_subscriber::fmt::layer().pretty();
        registry.with(stdout_layer).init();
    } else {
        registry.init();
    }
    Some(guard)
}
fn start_config_watcher(config: Config) {
    std::thread::spawn(move || {
        let config_dir = crate::os::get_config_dir();
        let config_path = config_dir.join("config.toml");
        let mut last_modified = fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        loop {
            std::thread::sleep(Duration::from_secs(60));
            if let Ok(metadata) = fs::metadata(&config_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_modified {
                        last_modified = modified;
                        if let Ok(content) = fs::read_to_string(&config_path) {
                            if let Ok(new_config_file) = toml::from_str::<ConfigFile>(&content) {
                                if let Ok(mut privacy) = config.privacy.write() {
                                    *privacy = new_config_file.privacy;
                                    info!("Config hot-reloaded: Privacy settings updated.");
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
    let _guard = setup_logging(is_daemon);
    let is_export_csv = args
        .iter()
        .position(|arg| arg == "--export-csv")
        .and_then(|i| args.get(i + 1).cloned());
    let is_export_txt = args
        .iter()
        .position(|arg| arg == "--export-txt")
        .and_then(|i| args.get(i + 1).cloned());
    let is_purge = args.iter().any(|arg| arg == "--purge");
    let is_export_json = args
        .iter()
        .position(|arg| arg == "--export-json")
        .and_then(|i| args.get(i + 1).cloned());
    let is_search = args
        .iter()
        .position(|arg| arg == "--search")
        .and_then(|i| args.get(i + 1).cloned());
    let is_delete_app = args
        .iter()
        .position(|arg| arg == "--delete-app")
        .and_then(|i| args.get(i + 1).cloned());
    let is_top_apps = args.iter().any(|arg| arg == "--top-apps");
    let is_total_words = args.iter().any(|arg| arg == "--total-words");
    let is_list_apps = args.iter().any(|arg| arg == "--list-apps");
    let is_count_entries = args.iter().any(|arg| arg == "--count-entries");
    let is_recent_logs = args.iter().any(|arg| arg == "--recent-logs");
    let is_busiest_day = args.iter().any(|arg| arg == "--busiest-day");
    let is_active_hours = args.iter().any(|arg| arg == "--active-hours");
    let mut config = Config::default();
    if is_daemon {
        run_daemon().await?;
    } else {
        // If config file doesn't exist, use default XDG path for db
        let default_dir = crate::os::get_data_dir();
        std::fs::create_dir_all(&default_dir).unwrap_or_default();
        config.storage.db_path = default_dir
            .join("activity_log.db")
            .to_str()
            .unwrap_or("activity_log.db")
            .to_string();
        if let Some(path) = is_export_csv {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => {
                    if let Err(e) = db.export_to_csv(&path) {
                        error!("Failed to export CSV: {}", e);
                    } else {
                        info!("Exported to CSV: {}", path);
                    }
                }
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if let Some(path) = is_export_txt {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => {
                    if let Err(e) = db.export_to_txt(&path) {
                        error!("Failed to export TXT: {}", e);
                    } else {
                        info!("Exported to TXT: {}", path);
                    }
                }
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_purge {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(mut db) => {
                    if let Err(e) = db.purge_all_data() {
                        error!("Failed to purge data: {}", e);
                    } else {
                        info!("Purged all data");
                    }
                }
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if let Some(path) = is_export_json {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => {
                    if let Err(e) = db.export_to_json(&path) {
                        error!("Failed to export JSON: {}", e);
                    } else {
                        info!("Exported to JSON: {}", path);
                    }
                }
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if let Some(keyword) = is_search {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.search_logs(&keyword) {
                    Ok(results) => {
                        for (timestamp, app_name, window_title, buffer) in results {
                            println!(
                                "[{}] {} ({}): {}",
                                timestamp, app_name, window_title, buffer
                            );
                        }
                    }
                    Err(e) => error!("Search failed: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if let Some(app_name) = is_delete_app {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.delete_app_logs(&app_name) {
                    Ok(count) => info!("Deleted {} logs for app '{}'", count, app_name),
                    Err(e) => error!("Failed to delete app logs: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_top_apps {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_top_apps() {
                    Ok(apps) => {
                        println!("Top Apps:");
                        for (app, count) in apps {
                            println!("- {}: {}", app, count);
                        }
                    }
                    Err(e) => error!("Failed to get top apps: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_total_words {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_total_words() {
                    Ok(words) => println!("Total Words Typed: {}", words),
                    Err(e) => error!("Failed to get total words: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_list_apps {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.list_unique_apps() {
                    Ok(apps) => {
                        println!("Tracked Apps:");
                        for app in apps {
                            println!("- {}", app);
                        }
                    }
                    Err(e) => error!("Failed to list apps: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_count_entries {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_total_entries() {
                    Ok(count) => println!("Total Log Entries: {}", count),
                    Err(e) => error!("Failed to get entry count: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_recent_logs {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_recent_logs_full() {
                    Ok(results) => {
                        println!("Recent 10 Logs:");
                        for (timestamp, app_name, window_title, buffer) in results {
                            println!(
                                "[{}] {} ({}): {}",
                                timestamp, app_name, window_title, buffer
                            );
                        }
                    }
                    Err(e) => error!("Failed to get recent logs: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_busiest_day {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_busiest_day_of_week() {
                    Ok(day) => println!("Busiest Day: {}", day),
                    Err(e) => error!("Failed to get busiest day: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        if is_active_hours {
            match crate::storage::db::Database::new(&config.storage.db_path) {
                Ok(db) => match db.get_hourly_activity() {
                    Ok(hours) => {
                        println!("Hourly Activity:");
                        for (hour, count) in hours {
                            if count > 0 {
                                println!("{:02}:00 - {}", hour, count);
                            }
                        }
                    }
                    Err(e) => error!("Failed to get hourly activity: {}", e),
                },
                Err(e) => error!("Could not open database: {}", e),
            }
            return Ok(());
        }
        run_client().await?;
    }
    Ok(())
}
async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal safety (even in daemon, for logs/panic)
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        error!("\n\rDaemon crashed: {:?}", panic_info);
        eprintln!("\n\rDaemon crashed: {:?}", panic_info);
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
        storage_tx.clone(),
    )));
    #[cfg(windows)]
    {
        let engine_clone = Arc::clone(&engine);
        let (tx, mut rx) = mpsc::channel(100);
        crate::os::windows::start_windows_hook(tx);
        tokio::spawn(async move {
            while let Some(key) = rx.recv().await {
                let mut engine_lock = engine_clone.write().await;
                engine_lock.handle_key(key).await;
            }
        });
    }
    // Start Linux collector if applicable
    #[cfg(all(target_os = "linux", feature = "evdev_support"))]
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
        }
    });
    info!("Static-Memory Daemon started.");
    // Start IPC Server
    let engine_ipc = Arc::clone(&engine);
    let storage_ipc = storage_tx.clone();
    #[cfg(target_os = "linux")]
    tokio::spawn(async move {
        if let Ok(listener) = crate::os::ipc::linux::listen().await {
            while let Ok((mut stream, _)) = listener.accept().await {
                let storage = storage_ipc.clone();
                let engine_conn = Arc::clone(&engine_ipc);
                tokio::spawn(async move {
                    while let Ok(msg) = crate::os::ipc::receive_message(&mut stream).await {
                        use crate::models::{IPCMessage, IPCResponse};
                        let response = match msg {
                            IPCMessage::GetAnalytics => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let _ = storage
                                    .send(crate::storage::StorageCommand::GetAnalytics {
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(data) = rx.recv().await {
                                    IPCResponse::Analytics(data)
                                } else {
                                    IPCResponse::Error("Failed to get analytics".into())
                                }
                            }
                            IPCMessage::ExportData { start, end, format } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let fmt_clone = format.clone();
                                let _ = storage
                                    .send(crate::storage::StorageCommand::Export {
                                        start,
                                        end,
                                        format,
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(data) = rx.recv().await {
                                    let data_dir = crate::os::get_data_dir();
                                    let export_dir = data_dir.join("exports");
                                    let filename = export_dir.join(format!(
                                        "export_{}.{}",
                                        Utc::now().timestamp(),
                                        fmt_clone
                                    ));
                                    let _ = std::fs::create_dir_all(&export_dir);
                                    if std::fs::write(&filename, data).is_ok() {
                                        IPCResponse::Ok
                                    } else {
                                        IPCResponse::Error("Failed to write export file".into())
                                    }
                                } else {
                                    IPCResponse::Error("Export failed".into())
                                }
                            }
                            IPCMessage::GetTimeline { limit: _ } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let _ = storage
                                    .send(crate::storage::StorageCommand::QueryHistory {
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(history) = rx.recv().await {
                                    IPCResponse::Timeline(history)
                                } else {
                                    IPCResponse::Error("Failed to get timeline".into())
                                }
                            }
                            IPCMessage::Shutdown => {
                                let _ =
                                    crate::os::ipc::send_response(&mut stream, &IPCResponse::Ok)
                                        .await;
                                std::process::exit(0);
                            }
                            IPCMessage::GetStatus => {
                                let engine_lock = engine_conn.read().await;
                                IPCResponse::Status {
                                    is_paused: false,
                                    is_idle: engine_lock.is_idle(),
                                }
                            }
                            _ => IPCResponse::Error("Not implemented".into()),
                        };
                        let _ = crate::os::ipc::send_response(&mut stream, &response).await;
                    }
                });
            }
        }
    });
    #[cfg(windows)]
    tokio::spawn(async move {
        while let Ok(mut server) = crate::os::ipc::windows::listen() {
            if server.connect().await.is_ok() {
                let storage = storage_ipc.clone();
                let engine_conn = Arc::clone(&engine_ipc);
                tokio::spawn(async move {
                    while let Ok(msg) = crate::os::ipc::receive_message(&mut server).await {
                        use crate::models::{IPCMessage, IPCResponse};
                        let response = match msg {
                            IPCMessage::GetAnalytics => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let _ = storage
                                    .send(crate::storage::StorageCommand::GetAnalytics {
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(data) = rx.recv().await {
                                    IPCResponse::Analytics(data)
                                } else {
                                    IPCResponse::Error("Failed to get analytics".into())
                                }
                            }
                            IPCMessage::ExportData { start, end, format } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let fmt_clone = format.clone();
                                let _ = storage
                                    .send(crate::storage::StorageCommand::Export {
                                        start,
                                        end,
                                        format,
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(data) = rx.recv().await {
                                    let data_dir = crate::os::get_data_dir();
                                    let export_dir = data_dir.join("exports");
                                    let filename = export_dir.join(format!(
                                        "export_{}.{}",
                                        Utc::now().timestamp(),
                                        fmt_clone
                                    ));
                                    let _ = std::fs::create_dir_all(&export_dir);
                                    if std::fs::write(&filename, data).is_ok() {
                                        IPCResponse::Ok
                                    } else {
                                        IPCResponse::Error("Failed to write export file".into())
                                    }
                                } else {
                                    IPCResponse::Error("Export failed".into())
                                }
                            }
                            IPCMessage::GetTimeline { limit: _ } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let _ = storage
                                    .send(crate::storage::StorageCommand::QueryHistory {
                                        sender: tx,
                                    })
                                    .await;
                                if let Some(history) = rx.recv().await {
                                    IPCResponse::Timeline(history)
                                } else {
                                    IPCResponse::Error("Failed to get timeline".into())
                                }
                            }
                            IPCMessage::Shutdown => {
                                let _ =
                                    crate::os::ipc::send_response(&mut server, &IPCResponse::Ok)
                                        .await;
                                std::process::exit(0);
                            }
                            IPCMessage::GetStatus => {
                                let engine_lock = engine_conn.read().await;
                                IPCResponse::Status {
                                    is_paused: false,
                                    is_idle: engine_lock.is_idle(),
                                }
                            }
                            _ => IPCResponse::Error("Not implemented".into()),
                        };
                        let _ = crate::os::ipc::send_response(&mut server, &response).await;
                    }
                });
            }
        }
    });
    println!("Static-Memory Daemon started.");
    tokio::signal::ctrl_c().await?;
    let mut engine_lock = engine.write().await;
    engine_lock.flush().await;
    Ok(())
}
async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    use crate::os::ipc::connect_with_retry;
    use tokio::time::Duration;
    // Auto-reconnect loop
    loop {
        // Attempt to connect
        let stream_res = connect_with_retry(u32::MAX, Duration::from_secs(2)).await;
        match stream_res {
            Ok(mut stream) => {
                use crate::ui::setup_app;
                use tuirealm::Update;
                let mut model = setup_app();
                let _ = model.terminal.enter_alternate_screen();
                let _ = model.terminal.enable_raw_mode();
                let mut last_sync = std::time::Instant::now() - Duration::from_secs(5);
                let mut last_tab = model.active_tab;
                while !model.quit {
                    if let Ok(events) = model.app.tick(tuirealm::PollStrategy::Once) {
                        for event in events {
                            if let Some(crate::ui::app::Msg::ExportExecuted(fmt)) =
                                model.update(Some(event))
                            {
                                let now = Utc::now();
                                let start = now - chrono::Duration::days(7);
                                if crate::os::ipc::send_message(
                                    &mut stream,
                                    &crate::models::IPCMessage::ExportData {
                                        start,
                                        end: now,
                                        format: fmt,
                                    },
                                )
                                .await
                                .is_err()
                                {
                                    break;
                                }
                                if crate::os::ipc::receive_response(&mut stream).await.is_err() {
                                    break;
                                }
                                model.update(Some(crate::ui::app::Msg::SwitchTab(
                                    crate::ui::Id::Timeline,
                                )));
                            }
                        }
                    }
                    let now = std::time::Instant::now();
                    let force_sync = model.active_tab != last_tab;
                    if force_sync || now.duration_since(last_sync) >= Duration::from_secs(2) {
                        // Sync State
                        if crate::os::ipc::send_message(
                            &mut stream,
                            &crate::models::IPCMessage::GetStatus,
                        )
                        .await
                        .is_err()
                        {
                            break;
                        }
                        match crate::os::ipc::receive_response(&mut stream).await {
                            Ok(crate::models::IPCResponse::Status { is_idle, .. }) => {
                                model.update(Some(crate::ui::app::Msg::SetIdle(is_idle)));
                            }
                            Ok(_) => {}
                            Err(_) => {
                                break;
                            }
                        }
                        // Request data from daemon
                        if model.active_tab == crate::ui::Id::Timeline {
                            if crate::os::ipc::send_message(
                                &mut stream,
                                &crate::models::IPCMessage::GetTimeline { limit: 50 },
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                            match crate::os::ipc::receive_response(&mut stream).await {
                                Ok(crate::models::IPCResponse::Timeline(history)) => {
                                    model
                                        .update(Some(crate::ui::app::Msg::UpdateTimeline(history)));
                                }
                                Ok(_) => {}
                                Err(_) => {
                                    break;
                                }
                            }
                        } else if model.active_tab == crate::ui::Id::Dashboard {
                            if crate::os::ipc::send_message(
                                &mut stream,
                                &crate::models::IPCMessage::GetAnalytics,
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                            match crate::os::ipc::receive_response(&mut stream).await {
                                Ok(crate::models::IPCResponse::Analytics(data)) => {
                                    model.update(Some(crate::ui::app::Msg::UpdateAnalytics(data)));
                                }
                                Ok(_) => {}
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                        last_sync = now;
                        last_tab = model.active_tab;
                    }
                    model.view();
                }
                let _ = model.terminal.leave_alternate_screen();
                let _ = model.terminal.disable_raw_mode();
                if model.quit {
                    break;
                }
                // If connection was lost, print status message and loop back to reconnect
                println!("Lost connection to daemon. Reconnecting in 2 seconds...");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                eprintln!("Failed to connect to daemon: {}. Retrying...", e);
            }
        }
    }
    Ok(())
}
