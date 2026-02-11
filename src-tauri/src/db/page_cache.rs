use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct PageCacheManager {
    conn: Arc<std::sync::Mutex<Connection>>,
    ttl_seconds: i64, // 缓存有效期（秒）
}

impl PageCacheManager {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
            ttl_seconds: 6 * 3600, // 6 小时
        }
    }

    /// 初始化页面缓存表
    pub fn init_table(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS page_cache (
                page_key TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_expires_at ON page_cache(expires_at);
            "#,
        )?;
        Ok(())
    }

    /// 获取当前时间戳（秒）
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// 获取缓存的页面数据
    pub fn get(&self, page_key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let now = Self::current_timestamp();

        let mut stmt = conn.prepare(
            "SELECT data FROM page_cache WHERE page_key = ? AND expires_at > ?",
        )?;

        let result = stmt.query_row(params![page_key, now], |row| {
            Ok(row.get::<_, String>(0)?)
        });

        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 保存页面数据到缓存
    pub fn set(&self, page_key: &str, data: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Self::current_timestamp();
        let expires_at = now + self.ttl_seconds;

        conn.execute(
            "INSERT OR REPLACE INTO page_cache (page_key, data, cached_at, expires_at) VALUES (?, ?, ?, ?)",
            params![page_key, data, now, expires_at],
        )?;

        Ok(())
    }

    /// 删除指定页面的缓存
    pub fn delete(&self, page_key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM page_cache WHERE page_key = ?", params![page_key])?;
        Ok(())
    }

    /// 清理所有过期的缓存
    pub fn cleanup_expired(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = Self::current_timestamp();
        let deleted = conn.execute("DELETE FROM page_cache WHERE expires_at <= ?", params![now])?;
        Ok(deleted)
    }

    /// 清空所有缓存
    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM page_cache", [])?;
        Ok(())
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock().unwrap();
        let now = Self::current_timestamp();

        let total: i32 = conn.query_row("SELECT COUNT(*) FROM page_cache", [], |row| {
            row.get(0)
        })?;

        let valid: i32 = conn.query_row(
            "SELECT COUNT(*) FROM page_cache WHERE expires_at > ?",
            params![now],
            |row| row.get(0),
        )?;

        let expired = total - valid;

        Ok(CacheStats {
            total,
            valid,
            expired,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total: i32,
    pub valid: i32,
    pub expired: i32,
}

// Tauri 命令
#[tauri::command]
pub async fn get_page_cache(
    page_key: String,
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<Option<String>, String> {
    cache.get(&page_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_page_cache(
    page_key: String,
    data: String,
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<(), String> {
    cache.set(&page_key, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_page_cache(
    page_key: String,
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<(), String> {
    cache.delete(&page_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_expired_page_cache(
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<usize, String> {
    cache.cleanup_expired().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_all_page_cache(
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<(), String> {
    cache.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_page_cache_stats(
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<CacheStats, String> {
    cache.get_stats().map_err(|e| e.to_string())
}
