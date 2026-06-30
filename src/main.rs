pub mod os;
pub mod models;
pub mod storage;
pub mod engine;
pub mod collector;
pub mod ui;

use crate::os::{OSInterface, windows::WindowsOS, linux::LinuxOS};
use crate::models::Config;
use crate::storage::db::start_storage_thread;
use crate::engine::Engine;
use tokio::sync::mpsc;
use std::sync::Arc;

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

    // Initialize OS interface
    #[cfg(windows)]
    let os = WindowsOS;
    #[cfg(target_os = "linux")]
    let os = LinuxOS;
    #[cfg(not(any(windows, target_os = "linux")))]
    panic!("Unsupported OS");

    let mut engine = Engine::new(config, os, storage_tx);

    println!("Static-Memory started. Press Ctrl+C to exit.");

    // This is where collectors would send events to the engine
    // For this boilerplate, we'll just run a simple loop or wait

    tokio::signal::ctrl_c().await?;
    engine.flush().await;

    Ok(())
}
