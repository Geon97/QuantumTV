use rusqlite::{params, Connection, Result};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct ImageCacheManager {
    conn: Arc<std::sync::Mutex<Connection>>,
    max_cache_size: i64,  // 最大缓存大小（字节）
    max_cache_items: i32, // 最大缓存条目数
    ttl_days: i64,        // 缓存有效期（天）
}

impl ImageCacheManager {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
            max_cache_size: 500 * 1024 * 1024, // 500MB
            max_cache_items: 1000,             // 1000 张图片
            ttl_days: 60,                      // 60 天
        }
    }

    /// 初始化图片缓存表
    pub fn init_table(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS image_cache (
                url TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                access_count INTEGER DEFAULT 1,
                size INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_last_accessed ON image_cache(last_accessed);
            CREATE INDEX IF NOT EXISTS idx_created_at ON image_cache(created_at);
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

    /// 获取缓存的图片
    pub fn get(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();

        // 检查是否过期
        let ttl_timestamp = Self::current_timestamp() - (self.ttl_days * 24 * 3600);

        let mut stmt = conn
            .prepare("SELECT data, created_at FROM image_cache WHERE url = ? AND created_at > ?")?;

        let result = stmt.query_row(params![url, ttl_timestamp], |row| {
            Ok(row.get::<_, Vec<u8>>(0)?)
        });

        match result {
            Ok(data) => {
                // 更新访问时间和访问次数
                drop(stmt);
                conn.execute(
                    "UPDATE image_cache SET last_accessed = ?, access_count = access_count + 1 WHERE url = ?",
                    params![Self::current_timestamp(), url],
                )?;
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 保存图片到缓存（带元数据）
    pub fn set_with_metadata(
        &self,
        url: &str,
        data: &[u8],
        title: Option<&str>,
        source_name: Option<&str>,
        year: Option<&str>,
        category: Option<&str>,
        rating: Option<f64>,
    ) -> Result<()> {
        let size = data.len() as i32;
        let now = Self::current_timestamp();

        let conn = self.conn.lock().unwrap();
        drop(conn);
        self.cleanup_if_needed()?;

        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO image_cache (url, data, created_at, last_accessed, access_count, size, title, source_name, year, category, rating)
             VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?)",
            params![
                url,
                data,
                now,
                now,
                size,
                title.unwrap_or(""),
                source_name.unwrap_or(""),
                year.unwrap_or(""),
                category.unwrap_or(""),
                rating.unwrap_or(0.0),
            ],
        )?;

        Ok(())
    }

    /// 保存图片到缓存
    pub fn set(&self, url: &str, data: &[u8]) -> Result<()> {
        self.set_with_metadata(url, data, None, None, None, None, None)
    }

    /// 检查并清理缓存
    fn cleanup_if_needed(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. 清理过期的缓存
        let ttl_timestamp = Self::current_timestamp() - (self.ttl_days * 24 * 3600);
        conn.execute(
            "DELETE FROM image_cache WHERE created_at < ?",
            params![ttl_timestamp],
        )?;

        // 2. 检查缓存总大小
        let total_size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM image_cache",
            [],
            |row| row.get(0),
        )?;

        if total_size > self.max_cache_size {
            // 删除最少使用的缓存（LRU）
            let to_delete = (total_size - self.max_cache_size) * 2; // 删除超出部分的 2 倍
            conn.execute(
                "DELETE FROM image_cache WHERE url IN (
                    SELECT url FROM image_cache
                    ORDER BY last_accessed ASC, access_count ASC
                    LIMIT (SELECT COUNT(*) FROM image_cache WHERE (SELECT SUM(size) FROM image_cache) > ?)
                )",
                params![to_delete],
            )?;
        }

        // 3. 检查缓存条目数
        let count: i32 =
            conn.query_row("SELECT COUNT(*) FROM image_cache", [], |row| row.get(0))?;

        if count > self.max_cache_items {
            let to_delete = count - self.max_cache_items + 100; // 多删除 100 条
            conn.execute(
                "DELETE FROM image_cache WHERE url IN (
                    SELECT url FROM image_cache
                    ORDER BY last_accessed ASC, access_count ASC
                    LIMIT ?
                )",
                params![to_delete],
            )?;
        }

        Ok(())
    }

    /// 清理过期的缓存条目，返回被删除的行数
    pub fn cleanup_expired(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let ttl_timestamp = Self::current_timestamp() - (self.ttl_days * 24 * 3600);
        let deleted = conn.execute(
            "DELETE FROM image_cache WHERE created_at < ?",
            params![ttl_timestamp],
        )?;
        Ok(deleted)
    }

    /// 清空全部图片缓存
    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM image_cache", [])?;
        Ok(())
    }
}

// Tauri 命令
#[tauri::command]
pub fn get_cached_image(
    url: String,
    cache_manager: tauri::State<ImageCacheManager>,
) -> Result<Option<Vec<u8>>, String> {
    cache_manager
        .get(&url)
        .map_err(|e| format!("Failed to get cached image: {}", e))
}

#[tauri::command]
pub fn save_cached_image(
    url: String,
    data: Vec<u8>,
    cache_manager: tauri::State<ImageCacheManager>,
) -> Result<(), String> {
    cache_manager
        .set(&url, &data)
        .map_err(|e| format!("Failed to save cached image: {}", e))
}

#[tauri::command]
pub fn clear_image_cache(
    cache_manager: tauri::State<ImageCacheManager>,
) -> Result<(), String> {
    cache_manager
        .clear_all()
        .map_err(|e| format!("Failed to clear image cache: {}", e))
}
