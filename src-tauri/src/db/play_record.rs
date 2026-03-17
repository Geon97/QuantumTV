// 播放记录
use crate::db::db_client::Db;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
pub fn delete_play_record(app: AppHandle, db: State<'_, Db>, key: String) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open db");
        conn.execute_batch(
            r#"
            CREATE TABLE play_records (
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
        .expect("init play_records");
        Db::new(conn)
    }

    fn make_play_record(key: &str, title: &str) -> PlayRecord {
        PlayRecord {
            key: key.to_string(),
            title: title.to_string(),
            source_name: "source1".to_string(),
            year: "2024".to_string(),
            cover: "cover_url".to_string(),
            episode_index: 1,
            total_episodes: 12,
            play_time: 1800,
            total_time: 3600,
            save_time: 1000000,
            search_title: "search_title".to_string(),
        }
    }

    #[test]
    fn save_play_record_inserts_new_record() {
        let db = setup_test_db();
        let record = make_play_record("key1", "Title 1");

        let result = db.with_conn(|conn| {
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
                ],
            )
        });

        assert!(result.is_ok());

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn save_play_record_updates_existing_record() {
        let db = setup_test_db();
        let mut record = make_play_record("key1", "Title 1");

        // Insert first
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                ],
            )
        })
        .unwrap();

        // Update
        record.play_time = 2700;
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
                ],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 1); // Still only 1 record

        let updated_play_time: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT play_time FROM play_records WHERE key = ?1",
                    params!["key1"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(updated_play_time, 2700);
    }

    #[test]
    fn get_all_play_records_returns_empty_list_initially() {
        let db = setup_test_db();

        let records = db
            .with_conn(|conn| {
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
            .unwrap();

        assert_eq!(records.len(), 0);
    }

    #[test]
    fn get_all_play_records_returns_multiple_records() {
        let db = setup_test_db();

        // Insert 3 records
        for i in 1..=3 {
            let record = make_play_record(&format!("key{}", i), &format!("Title {}", i));
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    ],
                )
            })
            .unwrap();
        }

        let records = db
            .with_conn(|conn| {
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
            .unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].title, "Title 1");
        assert_eq!(records[1].title, "Title 2");
        assert_eq!(records[2].title, "Title 3");
    }

    #[test]
    fn delete_play_record_removes_by_key() {
        let db = setup_test_db();
        let record = make_play_record("key1", "Title 1");

        // Insert
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                ],
            )
        })
        .unwrap();

        // Delete
        db.with_conn(|conn| {
            conn.execute("DELETE FROM play_records WHERE key = ?1", params!["key1"])
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn delete_play_record_only_removes_target_record() {
        let db = setup_test_db();

        // Insert 3 records
        for i in 1..=3 {
            let record = make_play_record(&format!("key{}", i), &format!("Title {}", i));
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    ],
                )
            })
            .unwrap();
        }

        // Delete only key2
        db.with_conn(|conn| {
            conn.execute("DELETE FROM play_records WHERE key = ?1", params!["key2"])
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 2);

        // Verify key2 is gone
        let exists: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM play_records WHERE key = ?1",
                    params!["key2"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(exists, 0);
    }

    #[test]
    fn clear_all_play_records_removes_all_records() {
        let db = setup_test_db();

        // Insert 3 records
        for i in 1..=3 {
            let record = make_play_record(&format!("key{}", i), &format!("Title {}", i));
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    ],
                )
            })
            .unwrap();
        }

        // Clear all
        db.with_conn(|conn| conn.execute("DELETE FROM play_records", []))
            .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn play_record_with_various_episode_values() {
        let db = setup_test_db();

        let records = vec![
            make_play_record("k1", "T1"), // episode_index: 1
            PlayRecord {
                key: "k2".to_string(),
                title: "T2".to_string(),
                source_name: "s2".to_string(),
                year: "2023".to_string(),
                cover: "c2".to_string(),
                episode_index: 0,
                total_episodes: 10,
                play_time: 0,
                total_time: 3600,
                save_time: 2000000,
                search_title: "t2".to_string(),
            },
            PlayRecord {
                key: "k3".to_string(),
                title: "T3".to_string(),
                source_name: "s3".to_string(),
                year: "2022".to_string(),
                cover: "c3".to_string(),
                episode_index: 100,
                total_episodes: 120,
                play_time: 3600,
                total_time: 3600,
                save_time: 3000000,
                search_title: "t3".to_string(),
            },
        ];

        for record in &records {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    ],
                )
            })
            .unwrap();
        }

        // Verify episode values
        let ep_index: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT episode_index FROM play_records WHERE key = ?1",
                    params!["k3"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(ep_index, 100);

        let total_ep: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT total_episodes FROM play_records WHERE key = ?1",
                    params!["k3"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(total_ep, 120);
    }

    #[test]
    fn play_record_with_null_optional_fields() {
        let db = setup_test_db();

        // Insert record with null/empty optional fields
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    "key1",
                    "Title",
                    "source",
                    "",      // empty year
                    "",      // empty cover
                    0,
                    0,
                    0,
                    0,
                    1000000,
                    ""       // empty search_title
                ],
            )
        })
        .unwrap();

        let record = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title FROM play_records WHERE key = ?1",
                    params!["key1"],
                    |row| {
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
                    },
                )
            })
            .unwrap();

        assert_eq!(record.year, "");
        assert_eq!(record.cover, "");
    }
}
