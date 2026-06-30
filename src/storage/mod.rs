use rusqlite::{params, Connection};
use crate::models::ActivityRecord;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

enum StorageMsg {
    Save(ActivityRecord),
    GetRecent(usize, mpsc::Sender<Vec<ActivityRecord>>),
    Clear,
}

pub struct Storage {
    tx: mpsc::Sender<StorageMsg>,
}

impl Storage {
    pub fn new<P: AsRef<Path> + Send + 'static>(path: P) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let conn = Connection::open(path).expect("Failed to open database");

            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA cache_size = -2000;
                 PRAGMA temp_store = MEMORY;",
            ).expect("Failed to set PRAGMAs");

            conn.execute(
                "CREATE TABLE IF NOT EXISTS activity_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp INTEGER NOT NULL,
                    app_name TEXT NOT NULL,
                    window_title TEXT NOT NULL,
                    content TEXT NOT NULL
                )",
                [],
            ).expect("Failed to create table");

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_timestamp ON activity_log (timestamp)",
                [],
            ).ok();

            while let Ok(msg) = rx.recv() {
                match msg {
                    StorageMsg::Save(record) => {
                        let _ = conn.execute(
                            "INSERT INTO activity_log (timestamp, app_name, window_title, content)
                             VALUES (?1, ?2, ?3, ?4)",
                            params![
                                record.timestamp.timestamp(),
                                record.app_name.as_str(),
                                record.window_title.as_str(),
                                record.buffer,
                            ],
                        );
                    }
                    StorageMsg::GetRecent(limit, reply_tx) => {
                        let mut stmt = conn.prepare(
                            "SELECT timestamp, app_name, window_title, content
                             FROM activity_log
                             ORDER BY timestamp DESC
                             LIMIT ?1",
                        ).unwrap();

                        let rows = stmt.query_map(params![limit], |row| {
                            let timestamp_secs: i64 = row.get(0)?;
                            let dt = chrono::DateTime::from_timestamp(timestamp_secs, 0)
                                .unwrap_or_default();

                            Ok(ActivityRecord {
                                timestamp: dt,
                                app_name: row.get::<_, String>(1)?.into(),
                                window_title: row.get::<_, String>(2)?.into(),
                                buffer: row.get(3)?,
                            })
                        }).unwrap();

                        let results: Vec<ActivityRecord> = rows.filter_map(|r| r.ok()).collect();
                        let _ = reply_tx.send(results);
                    }
                    StorageMsg::Clear => {
                        let _ = conn.execute("DELETE FROM activity_log", []);
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    pub fn save_record(&self, record: ActivityRecord) -> anyhow::Result<()> {
        self.tx.send(StorageMsg::Save(record))?;
        Ok(())
    }

    pub fn get_recent_logs(&self, limit: usize) -> anyhow::Result<Vec<ActivityRecord>> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx.send(StorageMsg::GetRecent(limit, reply_tx))?;
        Ok(reply_rx.recv()?)
    }

    pub fn clear_logs(&self) -> anyhow::Result<()> {
        self.tx.send(StorageMsg::Clear)?;
        Ok(())
    }
}
