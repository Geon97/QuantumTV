// 跳过片头片尾
use crate::db::db_client::Db;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SkipConfig {
    pub key: String,
    pub enable: bool,
    pub intro_time: f64,
    pub outro_time: f64,
}

#[tauri::command]
pub fn get_skip_config(db: State<'_, Db>, key: String) -> Result<Option<SkipConfig>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, enable, intro_time, outro_time FROM skip_configs WHERE key = ?1",
        )?;

        let result = stmt.query_row(params![key], |row| {
            Ok(SkipConfig {
                key: row.get(0)?,
                enable: row.get::<_, i32>(1)? != 0,
                intro_time: row.get(2)?,
                outro_time: row.get(3)?,
            })
        });

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

#[tauri::command]
pub fn delete_skip_config(db: State<'_, Db>, key: String) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM skip_configs WHERE key = ?1", params![key])?;
        Ok(())
    })
}

#[tauri::command]
pub fn save_skip_config(db: State<'_, Db>, config: SkipConfig) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)",
            params![
                config.key,
                if config.enable { 1 } else { 0 },
                config.intro_time,
                config.outro_time,
            ],
        )?;
        Ok(())
    })
}
