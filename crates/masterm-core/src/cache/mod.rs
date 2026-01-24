//! Cache subsystem

pub mod db;

use anyhow::Result;
use db::CacheDb;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

static CACHE_INSTANCE: Lazy<Mutex<Option<CacheManager>>> = Lazy::new(|| Mutex::new(None));

pub struct CacheManager {
    pub db: CacheDb,
}

impl CacheManager {
    pub fn init() -> Result<()> {
        let cache_dir = dirs::home_dir()
            .map(|h| h.join(".masterm/cache.db"))
            .unwrap_or_else(|| PathBuf::from(".masterm_cache.db"));
            
        let db = CacheDb::new(cache_dir)?;
        let manager = Self { db };
        
        let mut instance = CACHE_INSTANCE.lock().unwrap();
        *instance = Some(manager);
        
        Ok(())
    }

    pub fn get_instance() -> Option<std::sync::MutexGuard<'static, Option<CacheManager>>> {
        CACHE_INSTANCE.lock().ok()
    }

    pub fn get(key: &str) -> Option<String> {
        let guard = CACHE_INSTANCE.lock().unwrap();
        if let Some(manager) = guard.as_ref() {
            match manager.db.get(key) {
                Ok(Some(entry)) => return Some(entry.value),
                _ => return None,
            }
        }
        None
    }

    pub fn set(key: &str, value: &str, ttl_secs: u64) {
        let guard = CACHE_INSTANCE.lock().unwrap();
        if let Some(manager) = guard.as_ref() {
            let _ = manager.db.set(key, value, None, ttl_secs);
        }
    }
    
    pub fn clear() -> Result<()> {
        let guard = CACHE_INSTANCE.lock().unwrap();
        if let Some(manager) = guard.as_ref() {
            manager.db.clear()?;
        }
        Ok(())
    }
    
    pub fn stats() -> Result<(usize, usize)> {
        let guard = CACHE_INSTANCE.lock().unwrap();
        if let Some(manager) = guard.as_ref() {
            return manager.db.stats();
        }
        Ok((0, 0))
    }
}
