// 搜索历史
use crate::db::db_client::Db;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn add_search_history(
    app: AppHandle,
    db: State<'_, Db>,
    keyword: String,
) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO search_history (keyword, save_time) VALUES (?1, ?2)",
            params![keyword, now],
        )?;
        Ok(())
    })?;
    let _ = app.emit("searchHistoryUpdated", ());
    Ok(())
}
#[tauri::command]
pub fn clear_search_history(app: AppHandle, db: State<'_, Db>) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM search_history", [])?;
        Ok(())
    })?;
    let _ = app.emit("searchHistoryUpdated", ());
    Ok(())
}
#[tauri::command]
pub fn delete_search_history(
    app: AppHandle,
    db: State<'_, Db>,
    keyword: String,
) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute(
            "DELETE FROM search_history WHERE keyword = ?1",
            params![keyword],
        )?;
        Ok(())
    })?;
    let _ = app.emit("searchHistoryUpdated", ());
    Ok(())
}
#[tauri::command]
pub fn get_search_history(db: State<'_, Db>) -> Result<Vec<String>, String> {
    db.with_conn(|conn| {
        let mut stmt =
            conn.prepare("SELECT keyword FROM search_history ORDER BY save_time DESC")?;
        let rows = stmt.query_map([], |row| Ok(row.get(0).unwrap()))?;
        let mut keywords = Vec::new();
        for row in rows {
            keywords.push(row.unwrap());
        }
        Ok(keywords)
    })
}
