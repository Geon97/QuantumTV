// 播放记录
use crate::db::db_client::Db;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayRecord {
    key: String,
    title: String,
    source_name: String,
    year: String,
    cover: String,
    episode_index: i32,
    total_episodes: i32,
    play_time: i32,
    total_time: i32,
    save_time: i32,
    search_title: String,
}

#[tauri::command]
pub fn get_all_play_records(db: State<'_, Db>) -> Result<Vec<PlayRecord>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT * FROM play_records")?;
        let rows = stmt.query_map([], |row| {
            Ok(PlayRecord {
                key: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                source_name: row.get(2).unwrap(),
                year: row.get(3).unwrap(),
                cover: row.get(4).unwrap(),
                episode_index: row.get(5).unwrap(),
                total_episodes: row.get(6).unwrap(),
                play_time: row.get(7).unwrap(),
                total_time: row.get(8).unwrap(),
                save_time: row.get(9).unwrap(),
                search_title: row.get(10).unwrap(),
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.unwrap());
        }
        Ok(records)
    })
}

#[tauri::command]
pub fn save_play_record(db: State<'_, Db>, record: PlayRecord) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
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
    Ok(())
})
}

#[tauri::command]
pub fn delete_play_record(
    app: AppHandle,
    db: State<'_, Db>,
    key: String,
) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM play_records WHERE key = ?1", params![key])?;
        Ok(())
    })?;
    let _ = app.emit("playRecordsUpdated", ());
    Ok(())
}

#[tauri::command]
pub fn clear_all_play_records(app: AppHandle, db: State<'_, Db>) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM play_records", [])?;
        Ok(())
    })?;
    let _ = app.emit("playRecordsUpdated", ());
    Ok(())
}
