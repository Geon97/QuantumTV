// 收藏
use crate::db::db_client::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
#[derive(Debug, Serialize, Deserialize)]
pub struct Favorite {
    key: String,
    title: String,
    source_name: String,
    year: String,
    cover: String,
    episode_index: i32,
    total_episodes: i32,
    save_time: i32,
    search_title: String,
}
// 删除收藏
#[tauri::command]
pub fn delete_play_favorite(db: State<'_, Db>, key: String) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute(
            "DELETE FROM favorites WHERE key = ?1",
            rusqlite::params![key],
        )?;
        Ok(())
    })
}
// 获取收藏
#[tauri::command]
pub fn get_play_favorites(db: State<'_, Db>) -> Result<Vec<Favorite>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT * FROM favorites")?;
        let favorites_iter = stmt.query_map([], |row| {
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
        let favorites = favorites_iter.collect::<Result<Vec<Favorite>, _>>()?;

        Ok(favorites)
    })
}
// 获取某个是否已经收藏
#[tauri::command]
pub fn get_play_favorites_by_title(
    db: State<'_, Db>,
    title: String,
) -> Result<Vec<Favorite>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT * FROM favorites WHERE title = ?1")?;
        let favorites_iter = stmt.query_map([title], |row| {
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
        let favorites = favorites_iter.collect::<Result<Vec<Favorite>, _>>()?;
        Ok(favorites)
    })
}

#[tauri::command]
// 添加收藏记录
pub fn save_play_favorite(db: State<'_, Db>, record: Favorite) -> Result<(), String> {
    db.with_conn(|conn| {
    conn.execute(
        "INSERT OR REPLACE INTO favorites (key, title, source_name, year, cover, episode_index, total_episodes, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.key,
            record.title,
            record.source_name,
            record.year,
            record.cover,
            record.episode_index,
            record.total_episodes,
            record.save_time,
            record.search_title,
        ],
    )?;
    Ok(())
    })
}
// 清空所有收藏
#[tauri::command]
pub fn clear_all_favorites(db: State<'_, Db>) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM favorites", [])?;
        Ok(())
    })
}
