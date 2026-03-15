use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::types::Biobrick;

#[derive(Clone)]
pub struct SqliteCache {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteCache {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
        }

        let connection = Connection::open(path)?;
        let cache = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        cache.init()?;
        Ok(cache)
    }

    fn init(&self) -> Result<(), rusqlite::Error> {
        let connection = self.connection.lock().unwrap();
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS parts_cache (
                id_normalized TEXT PRIMARY KEY,
                biobrick_json TEXT NOT NULL,
                cached_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn get_part(&self, id_normalized: &str) -> Option<Biobrick> {
        let connection = self.connection.lock().unwrap();
        let payload: Option<String> = connection
            .query_row(
                "SELECT biobrick_json FROM parts_cache WHERE id_normalized = ?1",
                params![id_normalized],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten();

        payload.and_then(|json| serde_json::from_str(&json).ok())
    }

    pub fn put_part(&self, id_normalized: &str, biobrick: &Biobrick) -> Result<(), rusqlite::Error> {
        let payload = serde_json::to_string(biobrick)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let cached_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let connection = self.connection.lock().unwrap();
        connection.execute(
            "
            INSERT INTO parts_cache (id_normalized, biobrick_json, cached_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id_normalized) DO UPDATE SET
                biobrick_json = excluded.biobrick_json,
                cached_at = excluded.cached_at
            ",
            params![id_normalized, payload, cached_at],
        )?;
        Ok(())
    }

    pub fn stats_entries(&self) -> Result<i64, rusqlite::Error> {
        let connection = self.connection.lock().unwrap();
        connection.query_row(
            "SELECT COUNT(*) FROM parts_cache",
            [],
            |row| row.get(0),
        )
    }

    pub fn list_parts(&self) -> Result<Vec<Biobrick>, rusqlite::Error> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection.prepare("SELECT biobrick_json FROM parts_cache")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

        let parts = rows
            .filter_map(|r| r.ok())
            .filter_map(|json| serde_json::from_str::<Biobrick>(&json).ok())
            .collect::<Vec<_>>();

        Ok(parts)
    }
}
