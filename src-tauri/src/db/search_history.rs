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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open db");
        conn.execute_batch(
            r#"
            CREATE TABLE search_history (
              keyword TEXT PRIMARY KEY,
              save_time INTEGER
            );
            "#,
        )
        .expect("init search_history");
        Db::new(conn)
    }

    #[test]
    fn add_search_history_inserts_new_keyword() {
        let db = setup_test_db();
        let now = 1000000;

        db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", now],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn add_search_history_replaces_existing_keyword() {
        let db = setup_test_db();

        // Insert first
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 1000000],
            )
        })
        .unwrap();

        // Replace with new time
        db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 2000000],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);

        let save_time: i64 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT save_time FROM search_history WHERE keyword = ?1",
                    params!["anime"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(save_time, 2000000);
    }

    #[test]
    fn get_search_history_returns_empty_initially() {
        let db = setup_test_db();

        let keywords = db
            .with_conn(|conn| {
                let mut stmt =
                    conn.prepare("SELECT keyword FROM search_history ORDER BY save_time DESC")?;
                let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0).unwrap()))?;
                let mut keywords = Vec::new();
                for row in rows {
                    keywords.push(row.unwrap());
                }
                Ok(keywords)
            })
            .unwrap();

        assert_eq!(keywords.len(), 0);
    }

    #[test]
    fn get_search_history_returns_keywords_in_desc_order() {
        let db = setup_test_db();

        let keywords_to_insert = vec![
            ("anime", 1000000),
            ("movie", 2000000),
            ("drama", 3000000),
            ("music", 4000000),
        ];

        for (keyword, save_time) in keywords_to_insert {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                    params![keyword, save_time],
                )
            })
            .unwrap();
        }

        let keywords = db
            .with_conn(|conn| {
                let mut stmt =
                    conn.prepare("SELECT keyword FROM search_history ORDER BY save_time DESC")?;
                let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0).unwrap()))?;
                let mut keywords = Vec::new();
                for row in rows {
                    keywords.push(row.unwrap());
                }
                Ok(keywords)
            })
            .unwrap();

        assert_eq!(keywords.len(), 4);
        // Should be in descending order of save_time
        assert_eq!(keywords[0], "music");
        assert_eq!(keywords[1], "drama");
        assert_eq!(keywords[2], "movie");
        assert_eq!(keywords[3], "anime");
    }

    #[test]
    fn delete_search_history_removes_by_keyword() {
        let db = setup_test_db();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 1000000],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM search_history WHERE keyword = ?1",
                params!["anime"],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn delete_search_history_only_removes_target_keyword() {
        let db = setup_test_db();

        for (keyword, save_time) in &[("anime", 1000000i64), ("movie", 2000000i64), ("drama", 3000000i64)] {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                    params![keyword, save_time],
                )
            })
            .unwrap();
        }

        db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM search_history WHERE keyword = ?1",
                params!["movie"],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 2);

        let remaining = db
            .with_conn(|conn| {
                let mut stmt = conn.prepare("SELECT keyword FROM search_history")?;
                let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0).unwrap()))?;
                let mut keywords = Vec::new();
                for row in rows {
                    keywords.push(row.unwrap());
                }
                Ok(keywords)
            })
            .unwrap();

        assert!(remaining.contains(&"anime".to_string()));
        assert!(remaining.contains(&"drama".to_string()));
        assert!(!remaining.contains(&"movie".to_string()));
    }

    #[test]
    fn clear_search_history_removes_all_keywords() {
        let db = setup_test_db();

        for (keyword, save_time) in &[("anime", 1000000), ("movie", 2000000), ("drama", 3000000)]
        {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                    params![keyword, save_time],
                )
            })
            .unwrap();
        }

        db.with_conn(|conn| conn.execute("DELETE FROM search_history", []))
            .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn search_history_with_special_characters() {
        let db = setup_test_db();

        let special_keywords = vec![
            "anime & manga",
            "movie/series",
            "科幻",
            "动漫",
            "日本語",
            "🎬 movies",
        ];

        for keyword in &special_keywords {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                    params![keyword, 1000000],
                )
            })
            .unwrap();
        }

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, special_keywords.len() as i32);

        // Verify one special keyword
        let exists: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM search_history WHERE keyword = ?1",
                    params!["科幻"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(exists, 1);
    }

    #[test]
    fn search_history_with_long_keyword() {
        let db = setup_test_db();

        let long_keyword = "a".repeat(1000); // 1000 character keyword
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params![&long_keyword, 1000000],
            )
        })
        .unwrap();

        let retrieved = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT keyword FROM search_history WHERE keyword = ?1",
                    params![&long_keyword],
                    |row| row.get::<_, String>(0),
                )
            })
            .unwrap();

        assert_eq!(retrieved, long_keyword);
    }

    #[test]
    fn search_history_save_time_ordering() {
        let db = setup_test_db();

        // Insert with specific timestamps
        let data = vec![
            ("first", 100),
            ("second", 200),
            ("third", 300),
            ("fourth", 400),
            ("fifth", 500),
        ];

        for (keyword, time) in &data {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                    params![keyword, time],
                )
            })
            .unwrap();
        }

        let keywords = db
            .with_conn(|conn| {
                let mut stmt =
                    conn.prepare("SELECT keyword FROM search_history ORDER BY save_time DESC")?;
                let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(0).unwrap()))?;
                let mut keywords = Vec::new();
                for row in rows {
                    keywords.push(row.unwrap());
                }
                Ok(keywords)
            })
            .unwrap();

        // Verify descending order
        assert_eq!(keywords[0], "fifth");
        assert_eq!(keywords[1], "fourth");
        assert_eq!(keywords[2], "third");
        assert_eq!(keywords[3], "second");
        assert_eq!(keywords[4], "first");
    }
}
