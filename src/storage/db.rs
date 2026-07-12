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
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
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
        // Sanitize inputs to prevent corruption and handle edge cases
        let app_name = entry.app_name.as_str().replace('\0', "").replace('\u{FFFD}', "");
        let window_title = entry.window_title.as_str().replace('\0', "").replace('\u{FFFD}', "");
        let buffer = entry.buffer.replace('\0', "").replace('\u{FFFD}', "");

        // Enforce length limits for schema validation
        let app_name = if app_name.len() > 256 {
            app_name.chars().take(256).collect::<String>()
        } else {
            app_name
        };
        let window_title = if window_title.len() > 1024 {
            window_title.chars().take(1024).collect::<String>()
        } else {
            window_title
        };
        let buffer = if buffer.len() > 8192 {
            buffer.chars().take(8192).collect::<String>()
        } else {
            buffer
        };

        // We use parameterized queries (?1, ?2, etc.) to safely bind data,
        // which completely mitigates the risk of SQL injection.
        self.conn.execute(
            "INSERT INTO activity_log (timestamp, app_name, window_title, buffer) VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.timestamp.to_rfc3339(),
                app_name,
                window_title,
                buffer
            ],
        )?;
        Ok(())
    }

    pub fn insert_batch(&mut self, entries: &[LogEntry]) -> rusqlite::Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO activity_log (timestamp, app_name, window_title, buffer) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for entry in entries {
                stmt.execute(params![
                    entry.timestamp.to_rfc3339(),
                    entry.app_name.as_str(),
                    entry.window_title.as_str(),
                    entry.buffer
                ])?;
            }
        }
        tx.commit()?;
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
            "SELECT SUM(CASE WHEN trim(buffer) = '' THEN 0 ELSE length(trim(buffer)) - length(replace(trim(buffer), ' ', '')) + 1 END) FROM activity_log
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
            "SELECT AVG(CASE WHEN trim(buffer) = '' THEN 0 ELSE length(trim(buffer)) - length(replace(trim(buffer), ' ', '')) + 1 END) FROM activity_log
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
    pub fn export_to_json(&self, path: &str) -> std::io::Result<()> {
        let mut stmt = self.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp ASC").map_err(|e| std::io::Error::other(e.to_string()))?;
        let log_iter = stmt
            .query_map([], |row| {
                let timestamp: String = row.get(0)?;
                let app_name: String = row.get(1)?;
                let window_title: String = row.get(2)?;
                let buffer: String = row.get(3)?;
                Ok(serde_json::json!({
                    "timestamp": timestamp,
                    "app_name": app_name,
                    "window_title": window_title,
                    "buffer": buffer
                }))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let mut entries = Vec::new();
        for entry in log_iter.flatten() {
            entries.push(entry);
        }

        let json_string = serde_json::to_string_pretty(&entries)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        use std::io::{BufWriter, Write};
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json_string.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn search_logs(
        &self,
        keyword: &str,
    ) -> rusqlite::Result<Vec<(String, String, String, String)>> {
        let keyword_param = format!("%{}%", keyword);
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, app_name, window_title, buffer FROM activity_log
             WHERE buffer LIKE ?1 OR window_title LIKE ?1 OR app_name LIKE ?1
             ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([&keyword_param], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete_app_logs(&self, app_name: &str) -> rusqlite::Result<usize> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM activity_log WHERE app_name = ?1")?;
        stmt.execute([app_name])
    }

    pub fn list_unique_apps(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT app_name FROM activity_log ORDER BY app_name ASC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_recent_logs_full(&self) -> rusqlite::Result<Vec<(String, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, app_name, window_title, buffer FROM activity_log
             ORDER BY timestamp DESC LIMIT 10",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn export_to_csv(&self, path: &str) -> std::io::Result<()> {
        let mut stmt = self.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp ASC").map_err(|e| std::io::Error::other(e.to_string()))?;
        let log_iter = stmt
            .query_map([], |row| {
                Ok(format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\"
",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?.replace("\"", "\"\""),
                    row.get::<_, String>(2)?.replace("\"", "\"\""),
                    row.get::<_, String>(3)?.replace("\"", "\"\"")
                ))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        use std::io::{BufWriter, Write};
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(
            b"Timestamp,App Name,Window Title,Buffer
",
        )?;
        for log_str in log_iter.flatten() {
            writer.write_all(log_str.as_bytes())?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn export_to_txt(&self, path: &str) -> std::io::Result<()> {
        let mut stmt = self.conn.prepare("SELECT timestamp, app_name, window_title, buffer FROM activity_log ORDER BY timestamp ASC").map_err(|e| std::io::Error::other(e.to_string()))?;
        let log_iter = stmt
            .query_map([], |row| {
                Ok(format!(
                    "[{}] {} ({}): {}
",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?.replace("\"", "\"\""),
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?
                ))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        use std::io::{BufWriter, Write};
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        for log_str in log_iter.flatten() {
            writer.write_all(log_str.as_bytes())?;
        }
        writer.flush()?;
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

                // Acquire exclusive transaction lock to prevent writes
                if let Err(e) = self.conn.execute("BEGIN EXCLUSIVE TRANSACTION", []) {
                    eprintln!("Failed to acquire exclusive lock for rotation: {}", e);
                    return false;
                }

                // Perform safe backup using Backup API
                let backup_res = (|| -> rusqlite::Result<()> {
                    let mut dest = Connection::open(&backup_path)?;
                    let backup = rusqlite::backup::Backup::new(&self.conn, &mut dest)?;
                    backup.step(-1)?;
                    Ok(())
                })();

                if let Err(e) = backup_res {
                    eprintln!("Failed to backup database during rotation: {}", e);
                    let _ = self.conn.execute("ROLLBACK", []);
                    return false;
                }

                // If there are sequence numbers for autoincrement keys, clear them too
                let _ = self.conn.execute("DELETE FROM sqlite_sequence WHERE name='activity_log'", []);

                // Clear current database state and commit
                if let Err(e) = self.conn.execute("DELETE FROM activity_log", []) {
                    eprintln!("Failed to clear database after backup: {}", e);
                    let _ = self.conn.execute("ROLLBACK", []);
                    return false;
                }

                if let Err(e) = self.conn.execute("COMMIT", []) {
                    eprintln!("Failed to commit rotation: {}", e);
                    return false;
                }

                // Vacuum to reclaim space
                if let Err(e) = self.conn.execute("VACUUM", []) {
                    eprintln!("Failed to vacuum database after rotation: {}", e);
                    // Non-fatal error, backup was successful
                }

                return true;
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create storage runtime");
            
        rt.block_on(async move {
            let mut db = Database::new(&config.storage.db_path).expect("Failed to open database");

            // Initial retention check
            enforce_retention(&config);

            let mut buffer: Vec<LogEntry> = Vec::with_capacity(100);
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            if let Err(e) = db.insert_batch(&buffer) {
                                eprintln!("Database insert_batch error: {}", e);
                            }
                            buffer.clear();
                            if db.check_rotation(&config) {
                                enforce_retention(&config);
                            }
                        }
                    }
                    cmd = rx.recv() => {
                        match cmd {
                            Some(cmd) => {
                                match cmd {
                                    StorageCommand::Store(entry) => {
                                        buffer.push(entry);
                                        if buffer.len() >= 50 {
                                            if let Err(e) = db.insert_batch(&buffer) {
                                                eprintln!("Database insert_batch error: {}", e);
                                            }
                                            buffer.clear();
                                            if db.check_rotation(&config) {
                                                enforce_retention(&config);
                                            }
                                        }
                                    }
                                    StorageCommand::QueryHistory { .. } => {
                                        // Flush buffer before querying to ensure fresh data
                                        if !buffer.is_empty() {
                                            let _ = db.insert_batch(&buffer);
                                            buffer.clear();
                                        }
                                        // Placeholder for query logic
                                    }
                                    StorageCommand::GetAnalytics { sender } => {
                                        // Flush buffer before querying
                                        if !buffer.is_empty() {
                                            let _ = db.insert_batch(&buffer);
                                            buffer.clear();
                                        }
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
                                        let _ = sender.send(AnalyticsData {
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
                                        }).await;
                                    }
                                    StorageCommand::ExportCsv {
                                        target_path,
                                        sender,
                                    } => {
                                        // Flush buffer before querying
                                        if !buffer.is_empty() {
                                            let _ = db.insert_batch(&buffer);
                                            buffer.clear();
                                        }
                                        let result = match db.export_to_csv(&target_path) {
                                            Ok(_) => Ok(()),
                                            Err(e) => Err(e.to_string()),
                                        };
                                        let _ = sender.send(result).await;
                                    }
                                    StorageCommand::ExportTxt {
                                        target_path,
                                        sender,
                                    } => {
                                        // Flush buffer before querying
                                        if !buffer.is_empty() {
                                            let _ = db.insert_batch(&buffer);
                                            buffer.clear();
                                        }
                                        let result = match db.export_to_txt(&target_path) {
                                            Ok(_) => Ok(()),
                                            Err(e) => Err(e.to_string()),
                                        };
                                        let _ = sender.send(result).await;
                                    }
                                    StorageCommand::PurgeAll { sender } => {
                                        // Flush buffer (though it will be deleted)
                                        buffer.clear();
                                        let result = match db.purge_all_data() {
                                            Ok(_) => Ok(()),
                                            Err(e) => Err(e.to_string()),
                                        };
                                        let _ = sender.send(result).await;
                                    }
                                }
                            }
                            None => {
                                // Channel closed, flush remaining and exit
                                if !buffer.is_empty() {
                                    let _ = db.insert_batch(&buffer);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        });
    });
}
