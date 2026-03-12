use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
pub struct PlayRecord {
    pub key: String,
    pub title: String,
    pub source_name: String,
    pub year: String,
    pub cover: String,
    pub episode_index: i32,
    pub total_episodes: i32,
    pub play_time: i32,
    pub total_time: i32,
    pub save_time: i32,
    pub search_title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Favorite {
    pub key: String,
    pub title: String,
    pub source_name: String,
    pub year: String,
    pub cover: String,
    pub episode_index: i32,
    pub total_episodes: i32,
    pub save_time: i32,
    pub search_title: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchHistory {
    pub keyword: String,
    pub save_time: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SkipConfig {
    pub key: String,
    pub enable: i32,
    pub intro_time: f64,
    pub outro_time: f64,
}

pub fn init_db(app: &tauri::AppHandle) -> Connection {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("failed to create app dir");

    let db_path = app_dir.join("quantumtv.db");

    let conn = Connection::open(db_path).expect("failed to open database");

    // 打开外键支持
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    // 检查数据库版本并执行迁移
    let user_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    // 创建基础表（如果不存在）
    conn.execute_batch(
        // 播放记录表
        r#"
        CREATE TABLE IF NOT EXISTS play_records (
          key TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          source_name TEXT NOT NULL,
          year TEXT,
          cover TEXT,
          episode_index INTEGER,
          total_episodes INTEGER,
          play_time INTEGER,
          total_time INTEGER,
          save_time INTEGER,
          search_title TEXT
        );
        "#,
    )
    .expect("failed to create play_records table");

    conn.execute_batch(
        // 收藏表
        r#"
        CREATE TABLE IF NOT EXISTS favorites (
          key TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          source_name TEXT NOT NULL,
          year TEXT,
          cover TEXT,
          episode_index INTEGER,
          total_episodes INTEGER,
          save_time INTEGER,
          search_title TEXT
        );
        "#,
    )
    .expect("failed to create favorites table");

    conn.execute_batch(
        // 搜索历史表
        r#"
        CREATE TABLE IF NOT EXISTS search_history (
          keyword TEXT PRIMARY KEY,
          save_time INTEGER
        );
        "#,
    )
    .expect("failed to create search_history table");

    conn.execute_batch(
        // 跳过片头片尾配置表
        r#"
        CREATE TABLE IF NOT EXISTS skip_configs (
          key TEXT PRIMARY KEY,
          enable INTEGER DEFAULT 0,
          intro_time REAL DEFAULT 0,
          outro_time REAL DEFAULT 0
        );
        "#,
    )
    .expect("failed to create skip_configs table");

    conn.execute_batch(
        // 图片缓存表
        r#"
        CREATE TABLE IF NOT EXISTS image_cache (
            url TEXT PRIMARY KEY,
            data BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            last_accessed INTEGER NOT NULL,
            access_count INTEGER DEFAULT 1,
            size INTEGER NOT NULL,
            title TEXT,
            source_name TEXT,
            year TEXT,
            category TEXT,
            rating REAL
        );

        CREATE INDEX IF NOT EXISTS idx_last_accessed ON image_cache(last_accessed);
        CREATE INDEX IF NOT EXISTS idx_created_at ON image_cache(created_at);
        CREATE INDEX IF NOT EXISTS idx_access_count ON image_cache(access_count);
        CREATE INDEX IF NOT EXISTS idx_rating ON image_cache(rating);
        "#,
    )
    .expect("failed to create image_cache table");

    conn.execute_batch(
        // 内容池表：存储全局可推荐的内容
        r#"
        CREATE TABLE IF NOT EXISTS content_pool (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            source_name TEXT NOT NULL,
            year TEXT,
            cover TEXT,
            category TEXT,
            rating REAL DEFAULT 0.0,
            description TEXT,
            tags TEXT,
            popularity_score REAL DEFAULT 0.0,
            created_at INTEGER NOT NULL,
            last_updated INTEGER NOT NULL,
            UNIQUE(title, source_name)
        );

        CREATE INDEX IF NOT EXISTS idx_content_category ON content_pool(category);
        CREATE INDEX IF NOT EXISTS idx_content_rating ON content_pool(rating);
        CREATE INDEX IF NOT EXISTS idx_content_popularity ON content_pool(popularity_score);
        CREATE INDEX IF NOT EXISTS idx_content_year ON content_pool(year);
        "#,
    )
    .expect("failed to create content_pool table");

    // 数据库迁移：为 image_cache 表添加新字段（如果不存在）
    if user_version < 1 {
        // 检查 image_cache 表是否有新字段
        let has_title_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('image_cache') WHERE name='title'",
                [],
                |row| {
                    let count: i32 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if !has_title_column {
            // 添加新字段
            conn.execute_batch(
                r#"
                ALTER TABLE image_cache ADD COLUMN title TEXT;
                ALTER TABLE image_cache ADD COLUMN source_name TEXT;
                ALTER TABLE image_cache ADD COLUMN year TEXT;
                ALTER TABLE image_cache ADD COLUMN category TEXT;
                ALTER TABLE image_cache ADD COLUMN rating REAL;
                "#,
            )
            .expect("failed to add columns to image_cache table");

            // 创建新索引
            conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_access_count ON image_cache(access_count);
                CREATE INDEX IF NOT EXISTS idx_rating ON image_cache(rating);
                "#,
            )
            .expect("failed to create indexes on image_cache table");
        }

        // 更新数据库版本
        conn.execute("PRAGMA user_version = 1", [])
            .expect("failed to update database version");
    }

    conn
}

#[allow(dead_code)]
pub struct Db {
    pub conn: std::sync::Mutex<rusqlite::Connection>,
}
