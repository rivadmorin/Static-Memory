use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Configure SQLite for performance and reliability
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        let storage = Self { conn };
        storage.bootstrap()?;

        Ok(storage)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    fn bootstrap(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                content TEXT NOT NULL,
                context TEXT
            )",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_initialization() {
        let storage = Storage::new(":memory:").unwrap();
        let mut stmt = storage.connection().prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='events'").unwrap();
        let exists = stmt.exists([]).unwrap();
        assert!(exists);
    }
}
