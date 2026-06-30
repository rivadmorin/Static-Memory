use rusqlite::{params, Connection};
use smol_str::SmolStr;
use chrono::{DateTime, Utc};
use crate::models::{LogEntry, Config};
use crate::storage::{StorageCommand, AnalyticsData};
use tokio::sync::mpsc;
use std::thread;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, Duration};

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
                entry.app_name.to_string(),
                entry.window_title.to_string(),
                entry.buffer.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn get_top_apps(&self) -> rusqlite::Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT app_name, COUNT(*) as activity FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY app_name ORDER BY activity DESC LIMIT 5"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_hourly_activity(&self) -> rusqlite::Result<Vec<(u32, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT strftime('%H', timestamp) as hour, COUNT(*) FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY hour"
        )?;
        let rows = stmt.query_map([], |row| {
            let hour_str: String = row.get(0)?;
            Ok((hour_str.parse().unwrap_or(0), row.get(1)?))
        })?;

        let mut results = vec![(0, 0); 24];
        for row in rows {
            let (hour, count) = row?;
            if (hour as usize) < 24 {
                results[hour as usize] = (hour, count);
            }
        }
        Ok(results)
    }

    pub fn get_total_words(&self) -> rusqlite::Result<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT SUM(length(buffer) - length(replace(buffer, ' ', '')) + 1) FROM activity_log
             WHERE timestamp > datetime('now', 'start of day')"
        )?;
        let total: Option<usize> = stmt.query_row([], |row| row.get(0))?;
        Ok(total.unwrap_or(0))
    }

    pub fn export_data(&self, start: DateTime<Utc>, end: DateTime<Utc>, format: &str) -> rusqlite::Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, app_name, window_title, buffer FROM activity_log
             WHERE timestamp BETWEEN ?1 AND ?2 ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map([start.to_rfc3339(), end.to_rfc3339()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
        })?;

        let mut output = String::new();
        if format == "csv" {
            output.push_str("Timestamp,App,Window,Buffer\n");
            for row in rows {
                let (ts, app, win, buf) = row?;
                output.push_str(&format!("\"{}\",\"{}\",\"{}\",\"{}\"\n", ts, app, win, buf.replace("\"", "\"\"")));
            }
        } else {
            for row in rows {
                let (ts, app, win, buf) = row?;
                output.push_str(&format!("[{}] {} ({}): {}\n", ts, app, win, buf));
            }
        }
        Ok(output)
    }

    pub fn check_rotation(&mut self, config: &Config) -> bool {
        if let Ok(metadata) = fs::metadata(&config.storage.db_path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb >= config.storage.rotation_size_mb {
                let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                let backup_path = format!("{}.{}.bak", config.storage.db_path, ts);
                if let Err(e) = fs::rename(&config.storage.db_path, &backup_path) {
                    eprintln!("Failed to rotate database: {}", e);
                    return false;
                }
                // Re-open database
                if let Ok(new_db) = Database::new(&config.storage.db_path) {
                    *self = new_db;
                    return true;
                }
            }
        }
        false
    }
}

pub fn enforce_retention(config: &Config) {
    let db_dir = Path::new(&config.storage.db_path).parent().unwrap_or(Path::new("."));
    let retention_days = config.storage.retention_days as u64;
    let cutoff = SystemTime::now() - Duration::from_secs(retention_days * 24 * 3600);

    if let Ok(entries) = fs::read_dir(db_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "bak" {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified < cutoff {
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn start_storage_thread(
    config: Config,
    mut rx: mpsc::Receiver<StorageCommand>,
) {
    thread::spawn(move || {
        let mut db = Database::new(&config.storage.db_path).expect("Failed to open database");

        // Initial retention check
        enforce_retention(&config);

        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                StorageCommand::Store(entry) => {
                    if let Err(e) = db.insert(&entry) {
                        eprintln!("Database insert error: {}", e);
                    }
                    if db.check_rotation(&config) {
                        enforce_retention(&config);
                    }
                }
                StorageCommand::GetAnalytics { sender } => {
                    let top_apps = db.get_top_apps().unwrap_or_default();
                    let hourly_activity = db.get_hourly_activity().unwrap_or_default();
                    let total_words = db.get_total_words().unwrap_or_default();
                    let _ = sender.blocking_send(AnalyticsData { top_apps, hourly_activity, total_words });
                }
                StorageCommand::Export { start, end, format, sender } => {
                    let data = db.export_data(start, end, &format).unwrap_or_else(|e| format!("Export error: {}", e));
                    let _ = sender.blocking_send(data);
                }
                StorageCommand::QueryHistory { sender } => {
                    if let Ok(mut stmt) = db.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp DESC LIMIT 50") {
                        let rows_res = stmt.query_map([], |row| {
                            let ts_str: String = row.get(0)?;
                            let timestamp: DateTime<Utc> = DateTime::parse_from_rfc3339(&ts_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now());
                            Ok(LogEntry {
                                timestamp,
                                app_name: SmolStr::new(row.get::<_, String>(1)?),
                                window_title: SmolStr::new(row.get::<_, String>(2)?),
                                buffer: SmolStr::new(row.get::<_, String>(3)?),
                            })
                        });

                        if let Ok(rows) = rows_res {
                            let history: Vec<LogEntry> = rows.filter_map(|r| r.ok()).collect();
                            let _ = sender.blocking_send(history);
                        }
                    }
                }
            }
        }
    });
}
