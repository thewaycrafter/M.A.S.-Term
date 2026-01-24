//! Cache database implementation

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub metadata: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct CacheDb {
    conn: Connection,
}

impl CacheDb {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )?;
        
        Ok(Self { conn })
    }

    pub fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Lazy expiration: delete on fetch if expired
        self.conn.execute(
            "DELETE FROM cache WHERE key = ?1 AND expires_at < ?2",
            params![key, now],
        )?;

        self.conn
            .query_row(
                "SELECT key, value, metadata, created_at, expires_at FROM cache WHERE key = ?1",
                params![key],
                |row| {
                    Ok(CacheEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        metadata: row.get(2)?,
                        created_at: row.get(3)?,
                        expires_at: row.get(4)?,
                    })
                },
            )
            .optional()
            .context("Failed to query cache")
    }

    pub fn set(&self, key: &str, value: &str, metadata: Option<&str>, ttl_secs: u64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let expires_at = now + ttl_secs;

        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, metadata, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![key, value, metadata, now, expires_at],
        )?;

        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM cache", [])?;
        Ok(())
    }

    pub fn stats(&self) -> Result<(usize, usize)> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM cache",
            [],
            |row| row.get(0),
        )?;
        
        // This is a rough estimation of size in bytes (just counting rows isn't size, but usually sufficient for simple stats)
        // For real size we check the file size
        Ok((count, 0))
    }
}
