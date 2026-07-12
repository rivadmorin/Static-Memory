use static_memory::engine::Engine;
use static_memory::models::Config;
use static_memory::os::{OSInterface, WindowInfo};
use static_memory::storage::{db::Database, StorageCommand};

use std::fs;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Clone)]
struct MockOS {
    active_window: Arc<RwLock<Option<WindowInfo>>>,
}

impl MockOS {
    fn new() -> Self {
        Self {
            active_window: Arc::new(RwLock::new(None)),
        }
    }

    fn set_window(&self, process_name: &str, title: &str) {
        let mut lock = self.active_window.write().unwrap();
        *lock = Some(WindowInfo {
            process_name: smol_str::SmolStr::new(process_name),
            title: smol_str::SmolStr::new(title),
        });
    }
}

impl OSInterface for MockOS {
    fn get_active_window(&self) -> Option<WindowInfo> {
        self.active_window.read().unwrap().clone()
    }
}

#[tokio::test]
async fn test_window_filtering() {
    let config = Config::default();

    let mock_os = MockOS::new();
    let (storage_tx, mut storage_rx) = mpsc::channel(100);

    let mut engine = Engine::new(config.clone(), mock_os.clone(), storage_tx);

    // Test excluded window (by title)
    mock_os.set_window("chrome.exe", "Private Browsing");

    engine.handle_key('a').await;
    engine.handle_key('b').await;
    engine.handle_key('c').await;

    engine.flush().await;

    assert!(
        storage_rx.try_recv().is_err(),
        "Expected no messages for excluded window"
    );

    // Test excluded window (by process)
    mock_os.set_window("bitwarden.exe", "Bitwarden Password Manager");

    engine.handle_key('1').await;
    engine.handle_key('2').await;

    engine.flush().await;

    assert!(
        storage_rx.try_recv().is_err(),
        "Expected no messages for excluded process"
    );

    // Test allowed window
    mock_os.set_window("code.exe", "project - Visual Studio Code");

    engine.handle_key('h').await;
    engine.handle_key('e').await;
    engine.handle_key('l').await;
    engine.handle_key('l').await;
    engine.handle_key('o').await;

    engine.flush().await;

    if let Ok(StorageCommand::Store(entry)) = storage_rx.try_recv() {
        assert_eq!(entry.buffer.as_str(), "hello");
        assert_eq!(entry.app_name.as_str(), "code.exe");
        assert_eq!(entry.window_title.as_str(), "project - Visual Studio Code");
    } else {
        panic!("Expected Store command for allowed window");
    }
}

#[tokio::test]
async fn test_window_switch_flush() {
    let config = Config::default();

    let mock_os = MockOS::new();
    let (storage_tx, mut storage_rx) = mpsc::channel(100);

    let mut engine = Engine::new(config.clone(), mock_os.clone(), storage_tx);

    mock_os.set_window("app1.exe", "Window 1");
    engine.handle_key('a').await;
    engine.handle_key('b').await;

    // Changing window should cause immediate flush
    mock_os.set_window("app2.exe", "Window 2");

    engine.handle_key('c').await; // this triggers check_window_switch

    if let Ok(StorageCommand::Store(entry)) = storage_rx.try_recv() {
        assert_eq!(entry.buffer.as_str(), "ab");
        assert_eq!(entry.app_name.as_str(), "app1.exe");
    } else {
        panic!("Expected Store command on window switch");
    }
}

#[test]
fn test_database_rotation() {
    let mut config = Config::default();
    let db_path = "test_rotation.db";
    config.storage.db_path = db_path.to_string();
    config.storage.rotation_size_mb = 0; // Rotate immediately on any size > 0

    // Clean up before test
    let _ = fs::remove_file(db_path);
    for entry in fs::read_dir(".").unwrap().flatten() {
        if let Some(ext) = entry.path().extension() {
            if ext == "bak" && entry.path().to_str().unwrap().contains("test_rotation.db") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let mut db = Database::new(db_path).unwrap();

    // Write something to ensure file exists and size > 0
    let entry = static_memory::models::LogEntry {
        timestamp: chrono::Utc::now(),
        app_name: smol_str::SmolStr::new("test"),
        window_title: smol_str::SmolStr::new("test"),
        buffer: "test data".into(),
    };
    db.insert(&entry).unwrap();

    let rotated = db.check_rotation(&config);
    assert!(rotated, "Database should have been rotated");

    // Verify .bak file exists
    let mut found_bak = false;
    for entry in fs::read_dir(".").unwrap().flatten() {
        if let Some(ext) = entry.path().extension() {
            if ext == "bak" && entry.path().to_str().unwrap().contains("test_rotation.db") {
                found_bak = true;
                break;
            }
        }
    }
    assert!(found_bak, "Backup file should exist");

    // Clean up
    let _ = fs::remove_file(db_path);
    for entry in fs::read_dir(".").unwrap().flatten() {
        if let Some(ext) = entry.path().extension() {
            if ext == "bak" && entry.path().to_str().unwrap().contains("test_rotation.db") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[tokio::test]
async fn test_ipc_crash_resilience() {
    let config = Config::default();

    let mock_os = MockOS::new();
    let (storage_tx, storage_rx) = mpsc::channel(100);

    let mut engine = Engine::new(config.clone(), mock_os.clone(), storage_tx);

    mock_os.set_window("resilient.exe", "Test Window");

    // Drop the receiver to simulate a crash in the storage thread or IPC client
    drop(storage_rx);

    // Simulate high-frequency input that triggers a flush
    for i in 0..20 {
        // We shouldn't panic even though the receiver is dropped and flush() attempts to send
        engine
            .handle_key(char::from_digit(i % 10, 10).unwrap())
            .await;
    }
}
