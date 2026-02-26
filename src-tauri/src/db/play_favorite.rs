// 收藏
use crate::db::db_client::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize)]
pub struct TogglePlayFavoriteResponse {
    pub favorited: bool,
}
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

fn toggle_favorite_in_db(db: &Db, record: &Favorite) -> Result<bool, String> {
    let exists: i32 = db.with_conn(|conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM favorites WHERE key = ?1",
            params![record.key],
            |row| row.get(0),
        )
    })?;

    if exists > 0 {
        db.with_conn(|conn| {
            conn.execute("DELETE FROM favorites WHERE key = ?1", params![record.key])?;
            Ok(())
        })?;
        Ok(false)
    } else {
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
        })?;
        Ok(true)
    }
}

#[tauri::command]
pub fn toggle_play_favorite(
    app: AppHandle,
    db: State<'_, Db>,
    record: Favorite,
) -> Result<TogglePlayFavoriteResponse, String> {
    let favorited = toggle_favorite_in_db(&db, &record)?;
    let _ = app.emit("favoritesUpdated", ());
    Ok(TogglePlayFavoriteResponse { favorited })
}
// 清空所有收藏
#[tauri::command]
pub fn clear_all_favorites(app: AppHandle, db: State<'_, Db>) -> Result<(), String> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM favorites", [])?;
        Ok(())
    })?;
    let _ = app.emit("favoritesUpdated", ());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open db");
        conn.execute_batch(
            r#"
            CREATE TABLE favorites (
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
        .expect("init favorites");
        Db::new(conn)
    }

    fn make_favorite() -> Favorite {
        Favorite {
            key: "s1+id1".to_string(),
            title: "T".to_string(),
            source_name: "S".to_string(),
            year: "2024".to_string(),
            cover: "c".to_string(),
            episode_index: 1,
            total_episodes: 1,
            save_time: 10,
            search_title: "T".to_string(),
        }
    }

    #[test]
    fn toggle_favorite_inserts_then_deletes() {
        let db = setup_test_db();
        let record = make_favorite();

        let added = toggle_favorite_in_db(&db, &record).expect("add");
        assert!(added);

        let count: i32 = db
            .with_conn(|conn| conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0)))
            .expect("count");
        assert_eq!(count, 1);

        let removed = toggle_favorite_in_db(&db, &record).expect("remove");
        assert!(!removed);

        let count: i32 = db
            .with_conn(|conn| conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0)))
            .expect("count");
        assert_eq!(count, 0);
    }
}
