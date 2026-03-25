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

    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = DELETE;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 10000;
        "#,
    )
    .unwrap();

    let user_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    conn.execute_batch(
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
        r#"
        CREATE TABLE IF NOT EXISTS search_history (
          keyword TEXT PRIMARY KEY,
          save_time INTEGER
        );
        "#,
    )
    .expect("failed to create search_history table");

    conn.execute_batch(
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

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS video_sources (
            source_key TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            api TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            from_type TEXT NOT NULL DEFAULT 'custom',
            disabled INTEGER NOT NULL DEFAULT 0,
            is_adult INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_video_sources_sort_order
            ON video_sources(sort_order ASC, updated_at DESC);

        CREATE INDEX IF NOT EXISTS idx_video_sources_disabled
            ON video_sources(disabled, sort_order ASC);
        "#,
    )
    .expect("failed to create video_sources table");

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS source_intelligence_stats (
            source_key TEXT PRIMARY KEY REFERENCES video_sources(source_key) ON DELETE CASCADE,
            total_tests INTEGER NOT NULL DEFAULT 0,
            successful_tests INTEGER NOT NULL DEFAULT 0,
            total_response_time_ms INTEGER NOT NULL DEFAULT 0,
            last_success_time INTEGER,
            last_failure_time INTEGER,
            last_available_time INTEGER,
            consecutive_failures INTEGER NOT NULL DEFAULT 0,
            auto_degraded INTEGER NOT NULL DEFAULT 0,
            recent_results_json TEXT NOT NULL DEFAULT '[]',
            updated_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_source_intelligence_avg_time
            ON source_intelligence_stats(auto_degraded, total_response_time_ms, successful_tests);

        CREATE INDEX IF NOT EXISTS idx_source_intelligence_updated_at
            ON source_intelligence_stats(updated_at DESC);
        "#,
    )
    .expect("failed to create source_intelligence_stats table");

    if user_version < 1 {
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

            conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_access_count ON image_cache(access_count);
                CREATE INDEX IF NOT EXISTS idx_rating ON image_cache(rating);
                "#,
            )
            .expect("failed to create indexes on image_cache table");
        }

        conn.execute("PRAGMA user_version = 1", [])
            .expect("failed to update database version");
    }

    if user_version < 2 {
        conn.execute("PRAGMA user_version = 2", [])
            .expect("failed to update database version");
    }

    conn
}

#[allow(dead_code)]
pub struct Db {
    pub conn: std::sync::Mutex<rusqlite::Connection>,
}
