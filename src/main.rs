#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
use chrono::Utc;
use crate::models::{Config, ConfigFile};
use crate::storage::db::start_storage_thread;
use crate::engine::Engine;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::fs;

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
    let is_daemon = args.contains(&"--daemon".to_string());

    if is_daemon {
        run_daemon().await
    } else {
        run_client().await
    }
}

async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal safety (even in daemon, for logs/panic)
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
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

    let engine = Arc::new(tokio::sync::RwLock::new(Engine::new(config.clone(), os, storage_tx.clone())));

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
        }
    });

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
                                let _ = storage.send(crate::storage::StorageCommand::GetAnalytics { sender: tx }).await;
                                if let Some(data) = rx.recv().await {
                                    IPCResponse::Analytics(data)
                                } else {
                                    IPCResponse::Error("Failed to get analytics".into())
                                }
                            }
                            IPCMessage::ExportData { start, end, format } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let fmt_clone = format.clone();
                                let _ = storage.send(crate::storage::StorageCommand::Export { start, end, format, sender: tx }).await;
                                if let Some(data) = rx.recv().await {
                                    let data_dir = crate::os::get_data_dir();
                                    let export_dir = data_dir.join("exports");
                                    let filename = export_dir.join(format!("export_{}.{}", Utc::now().timestamp(), fmt_clone));
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
                                let _ = storage.send(crate::storage::StorageCommand::QueryHistory { sender: tx }).await;
                                if let Some(history) = rx.recv().await {
                                    IPCResponse::Timeline(history)
                                } else {
                                    IPCResponse::Error("Failed to get timeline".into())
                                }
                            }
                            IPCMessage::Shutdown => {
                                let _ = crate::os::ipc::send_response(&mut stream, &IPCResponse::Ok).await;
                                std::process::exit(0);
                            }
                            IPCMessage::GetStatus => {
                                let engine_lock = engine_conn.read().await;
                                IPCResponse::Status { is_paused: false, is_idle: engine_lock.is_idle() }
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
                                let _ = storage.send(crate::storage::StorageCommand::GetAnalytics { sender: tx }).await;
                                if let Some(data) = rx.recv().await {
                                    IPCResponse::Analytics(data)
                                } else {
                                    IPCResponse::Error("Failed to get analytics".into())
                                }
                            }
                            IPCMessage::ExportData { start, end, format } => {
                                let (tx, mut rx) = mpsc::channel(1);
                                let fmt_clone = format.clone();
                                let _ = storage.send(crate::storage::StorageCommand::Export { start, end, format, sender: tx }).await;
                                if let Some(data) = rx.recv().await {
                                    let data_dir = crate::os::get_data_dir();
                                    let export_dir = data_dir.join("exports");
                                    let filename = export_dir.join(format!("export_{}.{}", Utc::now().timestamp(), fmt_clone));
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
                                let _ = storage.send(crate::storage::StorageCommand::QueryHistory { sender: tx }).await;
                                if let Some(history) = rx.recv().await {
                                    IPCResponse::Timeline(history)
                                } else {
                                    IPCResponse::Error("Failed to get timeline".into())
                                }
                            }
                            IPCMessage::Shutdown => {
                                let _ = crate::os::ipc::send_response(&mut server, &IPCResponse::Ok).await;
                                std::process::exit(0);
                            }
                            IPCMessage::GetStatus => {
                                let engine_lock = engine_conn.read().await;
                                IPCResponse::Status { is_paused: false, is_idle: engine_lock.is_idle() }
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
    #[cfg(target_os = "linux")]
    let conn_res = crate::os::ipc::linux::connect().await;
    #[cfg(windows)]
    let conn_res = crate::os::ipc::windows::connect().await;

    #[cfg(any(target_os = "linux", windows))]
    match conn_res {
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
                        if let Some(crate::ui::app::Msg::ExportExecuted(fmt)) = model.update(Some(event)) {
                            let now = Utc::now();
                            let start = now - chrono::Duration::days(7);
                            let _ = crate::os::ipc::send_message(&mut stream, &crate::models::IPCMessage::ExportData { start, end: now, format: fmt }).await;
                            let _ = crate::os::ipc::receive_response(&mut stream).await;
                            model.update(Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline)));
                        }
                    }
                }

                let now = std::time::Instant::now();
                let force_sync = model.active_tab != last_tab;

                if force_sync || now.duration_since(last_sync) >= Duration::from_secs(2) {
                    // Sync State
                    let _ = crate::os::ipc::send_message(&mut stream, &crate::models::IPCMessage::GetStatus).await;
                    if let Ok(crate::models::IPCResponse::Status { is_idle, .. }) = crate::os::ipc::receive_response(&mut stream).await {
                        model.update(Some(crate::ui::app::Msg::SetIdle(is_idle)));
                    }

                    // Request data from daemon
                    if model.active_tab == crate::ui::Id::Timeline {
                        let _ = crate::os::ipc::send_message(&mut stream, &crate::models::IPCMessage::GetTimeline { limit: 50 }).await;
                        if let Ok(crate::models::IPCResponse::Timeline(history)) = crate::os::ipc::receive_response(&mut stream).await {
                            model.update(Some(crate::ui::app::Msg::UpdateTimeline(history)));
                        }
                    } else if model.active_tab == crate::ui::Id::Dashboard {
                        let _ = crate::os::ipc::send_message(&mut stream, &crate::models::IPCMessage::GetAnalytics).await;
                        if let Ok(crate::models::IPCResponse::Analytics(data)) = crate::os::ipc::receive_response(&mut stream).await {
                            model.update(Some(crate::ui::app::Msg::UpdateAnalytics(data)));
                        }
                    }
                    last_sync = now;
                    last_tab = model.active_tab;
                }

                model.view();
            }

            let _ = model.terminal.leave_alternate_screen();
            let _ = model.terminal.disable_raw_mode();
            Ok(())
        }
        Err(_) => {
            eprintln!("Daemon is not running. Please start the service first or run with --daemon flag.");
            std::process::exit(1);
        }
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        eprintln!("TUI Client not yet supported on this OS");
        Ok(())
    }
}
