use crate::models::{Config, LogEntry};
use crate::storage::{AnalyticsData, StorageCommand};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;

        // Optimizations
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -2000;
        ",
        )?;

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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON activity_log (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_name ON activity_log (app_name)",
            [],
        )?;

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

    pub fn get_top_apps(&self) -> rusqlite::Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT app_name, COUNT(*) as activity FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY app_name ORDER BY activity DESC LIMIT 5",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

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
             GROUP BY hour",
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
             WHERE timestamp > datetime('now', 'start of day')",
        )?;
        let total: Option<usize> = stmt.query_row([], |row| row.get(0))?;
        Ok(total.unwrap_or(0))
    }

    pub fn get_daily_activity_trend(&self) -> rusqlite::Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT date(timestamp) as day, COUNT(*) FROM activity_log
             WHERE timestamp > datetime('now', '-7 days')
             GROUP BY day ORDER BY day ASC",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_most_active_window_titles(&self) -> rusqlite::Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT window_title, COUNT(*) as activity FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY window_title ORDER BY activity DESC LIMIT 5",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_total_characters(&self) -> rusqlite::Result<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT SUM(length(buffer)) FROM activity_log
             WHERE timestamp > datetime('now', 'start of day')",
        )?;
        let total: Option<usize> = stmt.query_row([], |row| row.get(0))?;
        Ok(total.unwrap_or(0))
    }

    pub fn get_most_productive_hour(&self) -> rusqlite::Result<u32> {
        let mut stmt = self.conn.prepare(
            "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count FROM activity_log
             WHERE timestamp > datetime('now', 'start of day')
             GROUP BY hour ORDER BY count DESC LIMIT 1"
        )?;
        let hour: Option<u32> = stmt.query_row([], |row| row.get(0)).ok();
        Ok(hour.unwrap_or(0))
    }

    pub fn get_average_words_per_entry(&self) -> rusqlite::Result<f64> {
        let mut stmt = self.conn.prepare(
            "SELECT AVG(length(buffer) - length(replace(buffer, ' ', '')) + 1) FROM activity_log
             WHERE timestamp > datetime('now', 'start of day')",
        )?;
        let avg: Option<f64> = stmt.query_row([], |row| row.get(0))?;
        Ok(avg.unwrap_or(0.0))
    }

    pub fn get_longest_active_session(&self) -> rusqlite::Result<usize> {
        // Approximate: Count of entries today. A better way would be analyzing time diffs but complex for SQLite.
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM activity_log WHERE timestamp > datetime('now', 'start of day')",
        )?;
        let count: Option<usize> = stmt.query_row([], |row| row.get(0))?;
        Ok(count.unwrap_or(0))
    }

    pub fn get_busiest_day_of_week(&self) -> rusqlite::Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT case cast(strftime('%w', timestamp) as integer)
                when 0 then 'Sunday' when 1 then 'Monday' when 2 then 'Tuesday'
                when 3 then 'Wednesday' when 4 then 'Thursday' when 5 then 'Friday'
                else 'Saturday' end as day_name, COUNT(*) as count FROM activity_log
             WHERE timestamp > datetime('now', '-7 days')
             GROUP BY day_name ORDER BY count DESC LIMIT 1",
        )?;
        let day: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        Ok(day.unwrap_or_else(|| "N/A".to_string()))
    }

    pub fn get_most_used_app_heatmap(&self) -> rusqlite::Result<Vec<(String, u32, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT app_name, CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY app_name, hour ORDER BY count DESC LIMIT 5"
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_recent_apps(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT app_name FROM activity_log
             WHERE timestamp > datetime('now', '-1 day')
             GROUP BY app_name ORDER BY MAX(timestamp) DESC LIMIT 5",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_total_entries(&self) -> rusqlite::Result<usize> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM activity_log")?;
        let count: Option<usize> = stmt.query_row([], |row| row.get(0))?;
        Ok(count.unwrap_or(0))
    }

    pub fn export_to_csv(&self, path: &str) -> std::io::Result<()> {
        let mut stmt = self.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp ASC").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let log_iter = stmt.query_map([], |row| {
            Ok(format!("\"{}\",\"{}\",\"{}\",\"{}\"
",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?.replace("\"", "\"\""),
                row.get::<_, String>(2)?.replace("\"", "\"\""),
                row.get::<_, String>(3)?.replace("\"", "\"\"")
            ))
        }).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        file.write_all(b"Timestamp,App Name,Window Title,Buffer
")?;
        for log in log_iter {
            if let Ok(log_str) = log {
                file.write_all(log_str.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn export_to_txt(&self, path: &str) -> std::io::Result<()> {
        let mut stmt = self.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp ASC").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let log_iter = stmt.query_map([], |row| {
            Ok(format!("[{}] {} ({}): {}
",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?.replace("\"", "\"\""),
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?
            ))
        }).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        for log in log_iter {
            if let Ok(log_str) = log {
                file.write_all(log_str.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn purge_all_data(&mut self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM activity_log", [])?;
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    pub fn check_rotation(&mut self, config: &Config) -> bool {
        if let Ok(metadata) = fs::metadata(&config.storage.db_path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb >= config.storage.rotation_size_mb {
                let backup_path = format!(
                    "{}.{}.bak",
                    config.storage.db_path,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
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
    let db_dir = Path::new(&config.storage.db_path)
        .parent()
        .unwrap_or(Path::new("."));
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

pub fn start_storage_thread(config: Config, mut rx: mpsc::Receiver<StorageCommand>) {
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
                StorageCommand::QueryHistory { .. } => {
                    // Placeholder for query logic
                }
                StorageCommand::GetAnalytics { sender } => {
                    let top_apps = db.get_top_apps().unwrap_or_default();
                    let hourly_activity = db.get_hourly_activity().unwrap_or_default();
                    let total_words = db.get_total_words().unwrap_or_default();
                    let daily_activity_trend = db.get_daily_activity_trend().unwrap_or_default();
                    let most_active_window_titles =
                        db.get_most_active_window_titles().unwrap_or_default();
                    let total_characters = db.get_total_characters().unwrap_or_default();
                    let most_productive_hour = db.get_most_productive_hour().unwrap_or_default();
                    let average_words_per_entry =
                        db.get_average_words_per_entry().unwrap_or_default();
                    let longest_active_session =
                        db.get_longest_active_session().unwrap_or_default();
                    let busiest_day_of_week = db
                        .get_busiest_day_of_week()
                        .unwrap_or_else(|_| "N/A".to_string());
                    let most_used_app_heatmap = db.get_most_used_app_heatmap().unwrap_or_default();
                    let recent_apps = db.get_recent_apps().unwrap_or_default();
                    let total_entries = db.get_total_entries().unwrap_or_default();
                    let _ = sender.blocking_send(AnalyticsData {
                        top_apps,
                        hourly_activity,
                        total_words,
                        daily_activity_trend,
                        most_active_window_titles,
                        total_characters,
                        most_productive_hour,
                        average_words_per_entry,
                        longest_active_session,
                        busiest_day_of_week,
                        most_used_app_heatmap,
                        recent_apps,
                        total_entries,
                    });
                }
                StorageCommand::ExportCsv { target_path, sender } => {
                    let result = match db.export_to_csv(&target_path) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = sender.blocking_send(result);
                }
                StorageCommand::ExportTxt { target_path, sender } => {
                    let result = match db.export_to_txt(&target_path) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = sender.blocking_send(result);
                }
                StorageCommand::PurgeAll { sender } => {
                    let result = match db.purge_all_data() {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = sender.blocking_send(result);
                }
            }
        }
    });
}
