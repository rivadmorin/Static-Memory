use rusqlite::{params, Connection};
use crate::models::{LogEntry, Config};
use crate::storage::StorageCommand;
use tokio::sync::mpsc;
use std::thread;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;

        // Optimizations
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -2000;
        ")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS activity_log (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                app_name TEXT NOT NULL,
                window_title TEXT NOT NULL,
                buffer TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_timestamp ON activity_log (timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_app_name ON activity_log (app_name)", [])?;

        Ok(Self { conn })
    }

    pub fn insert(&mut self, entry: &LogEntry) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO activity_log (timestamp, app_name, window_title, buffer) VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.app_name.as_str(),
                entry.window_title.as_str(),
                entry.buffer
            ],
        )?;
        Ok(())
    }
}

pub fn start_storage_thread(
    config: Config,
    mut rx: mpsc::Receiver<StorageCommand>,
) {
    thread::spawn(move || {
        let mut db = Database::new(&config.storage.db_path).expect("Failed to open database");

        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                StorageCommand::Store(entry) => {
                    if let Err(e) = db.insert(&entry) {
                        eprintln!("Database insert error: {}", e);
                    }
                    // TODO: Implement rotation check here
                }
                StorageCommand::QueryHistory { sender } => {
                    // Placeholder for query logic
                }
            }
        }
    });
}
