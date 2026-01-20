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

    conn
}

#[allow(dead_code)]
pub struct Db {
    pub conn: std::sync::Mutex<rusqlite::Connection>,
}
