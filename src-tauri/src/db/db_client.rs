use crate::db::db_init::{Favorite, PlayRecord, SearchHistory, SkipConfig};
use rusqlite::params;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct ExportData {
    play_records: Vec<PlayRecord>,
    favorites: Vec<Favorite>,
    search_history: Vec<SearchHistory>,
    skip_configs: Vec<SkipConfig>,
}

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    // 现有的方法创建示例
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
    // 访问数据库
    pub fn with_conn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("数据库锁失败: {}", e))?;
        f(&conn).map_err(|e| e.to_string())
    }
    // 导出
    pub fn export_json(&self) -> Result<Vec<u8>, String> {
        let export_data = self
            .with_conn(|conn| {
                let mut play_records = conn.prepare("SELECT * FROM play_records")?;
                let mut favorites = conn.prepare("SELECT * FROM favorites")?;
                let mut search_history = conn.prepare("SELECT * FROM search_history")?;
                let mut skip_configs = conn.prepare("SELECT * FROM skip_configs")?;
                let play_records_iter = play_records.query_map([], |row| {
                    Ok(PlayRecord {
                        key: row.get(0)?,
                        title: row.get(1)?,
                        source_name: row.get(2)?,
                        year: row.get(3)?,
                        cover: row.get(4)?,
                        episode_index: row.get(5)?,
                        total_episodes: row.get(6)?,
                        play_time: row.get(7)?,
                        total_time: row.get(8)?,
                        save_time: row.get(9)?,
                        search_title: row.get(10)?,
                    })
                })?;
                let favorites_iter = favorites.query_map([], |row| {
                    Ok(Favorite {
                        key: row.get(0)?,
                        title: row.get(1)?,
                        source_name: row.get(2)?,
                        year: row.get(3)?,
                        cover: row.get(4)?,
                        episode_index: row.get(5)?,
                        total_episodes: row.get(6)?,
                        save_time: row.get(7)?,
                        search_title: row.get(8)?,
                    })
                })?;
                let search_history_iter = search_history.query_map([], |row| {
                    Ok(SearchHistory {
                        keyword: row.get(0)?,
                        save_time: row.get(1)?,
                    })
                })?;
                let skip_configs_iter = skip_configs.query_map([], |row| {
                    Ok(SkipConfig {
                        key: row.get(0)?,
                        enable: row.get(1)?,
                        intro_time: row.get(2)?,
                        outro_time: row.get(3)?,
                    })
                })?;
                let play_records =
                    play_records_iter.collect::<Result<Vec<PlayRecord>, rusqlite::Error>>()?;
                let favorites =
                    favorites_iter.collect::<Result<Vec<Favorite>, rusqlite::Error>>()?;
                let search_history =
                    search_history_iter.collect::<Result<Vec<SearchHistory>, rusqlite::Error>>()?;
                let skip_configs =
                    skip_configs_iter.collect::<Result<Vec<SkipConfig>, rusqlite::Error>>()?;
                Ok(ExportData {
                    play_records,
                    favorites,
                    search_history,
                    skip_configs,
                })
            })
            .map_err(|e| e.to_string())?;

        let export_data_json = serde_json::to_vec(&export_data).map_err(|e| e.to_string())?;

        Ok(export_data_json)
    }
    // 导入
    pub fn import_json(&self, data: String) -> Result<(), String> {
        let export_data: ExportData = serde_json::from_str(&data).map_err(|e| e.to_string())?;

        let play_record_data = export_data.play_records;
        let favorites_data = export_data.favorites;
        let search_history_data = export_data.search_history;
        let skip_configs_data = export_data.skip_configs;
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")?;
            for record in play_record_data {
                stmt.execute(params![
                    record.key,
                    record.title,
                    record.source_name,
                    record.year,
                    record.cover,
                    record.episode_index,
                    record.total_episodes,
                    record.play_time,
                    record.total_time,
                    record.save_time,
                    record.search_title,
                ])?;
            }
            Ok(())
        })?;
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO favorites (key, title, source_name, year, cover, episode_index, total_episodes, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")?;
            for record in favorites_data {
                stmt.execute(params![
                    record.key,
                    record.title,
                    record.source_name,
                    record.year,
                    record.cover,
                    record.episode_index,
                    record.total_episodes,
                    record.save_time,
                    record.search_title,
                ])?;
            }
            Ok(())
        })?;
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "INSERT OR IGNORE INTO search_history (keyword, save_time) VALUES (?1, ?2)",
            )?;
            for record in search_history_data {
                stmt.execute(params![record.keyword, record.save_time,])?;
            }
            Ok(())
        })?;
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)")?;
            for record in skip_configs_data {
                stmt.execute(params![
                    record.key,
                    record.enable,
                    record.intro_time,
                    record.outro_time,
                ])?;
            }
            Ok(())
        })?;
        Ok(())
    }
    // 清空缓存
    pub fn clear_cache(&self) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM play_records", [])?;
            conn.execute("DELETE FROM favorites", [])?;
            conn.execute("DELETE FROM search_history", [])?;
            conn.execute("DELETE FROM skip_configs", [])?;
            Ok(())
        })
    }
}
