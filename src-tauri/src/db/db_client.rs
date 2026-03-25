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
    video_sources: Vec<VideoSourceExport>,
    source_intelligence_stats: Vec<SourceIntelligenceStatsExport>,
}

#[derive(Serialize, Deserialize)]
struct VideoSourceExport {
    source_key: String,
    name: String,
    api: String,
    detail: String,
    from_type: String,
    disabled: i32,
    is_adult: i32,
    sort_order: i32,
    created_at: i64,
    updated_at: i64,
}

#[derive(Serialize, Deserialize)]
struct SourceIntelligenceStatsExport {
    source_key: String,
    total_tests: i64,
    successful_tests: i64,
    total_response_time_ms: i64,
    last_success_time: Option<i64>,
    last_failure_time: Option<i64>,
    last_available_time: Option<i64>,
    consecutive_failures: i32,
    auto_degraded: i32,
    recent_results_json: String,
    updated_at: i64,
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
                let mut video_sources = conn.prepare(
                    "SELECT source_key, name, api, detail, from_type, disabled, is_adult, sort_order, created_at, updated_at
                     FROM video_sources
                     ORDER BY sort_order ASC, updated_at DESC, source_key ASC",
                )?;
                let mut source_stats = conn.prepare(
                    "SELECT source_key, total_tests, successful_tests, total_response_time_ms, last_success_time, last_failure_time,
                            last_available_time, consecutive_failures, auto_degraded, recent_results_json, updated_at
                     FROM source_intelligence_stats",
                )?;
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
                let video_sources_iter = video_sources.query_map([], |row| {
                    Ok(VideoSourceExport {
                        source_key: row.get(0)?,
                        name: row.get(1)?,
                        api: row.get(2)?,
                        detail: row.get(3)?,
                        from_type: row.get(4)?,
                        disabled: row.get(5)?,
                        is_adult: row.get(6)?,
                        sort_order: row.get(7)?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                })?;
                let source_stats_iter = source_stats.query_map([], |row| {
                    Ok(SourceIntelligenceStatsExport {
                        source_key: row.get(0)?,
                        total_tests: row.get(1)?,
                        successful_tests: row.get(2)?,
                        total_response_time_ms: row.get(3)?,
                        last_success_time: row.get(4)?,
                        last_failure_time: row.get(5)?,
                        last_available_time: row.get(6)?,
                        consecutive_failures: row.get(7)?,
                        auto_degraded: row.get(8)?,
                        recent_results_json: row.get(9)?,
                        updated_at: row.get(10)?,
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
                let video_sources = video_sources_iter
                    .collect::<Result<Vec<VideoSourceExport>, rusqlite::Error>>()?;
                let source_intelligence_stats = source_stats_iter
                    .collect::<Result<Vec<SourceIntelligenceStatsExport>, rusqlite::Error>>()?;
                Ok(ExportData {
                    play_records,
                    favorites,
                    search_history,
                    skip_configs,
                    video_sources,
                    source_intelligence_stats,
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
        let video_sources_data = export_data.video_sources;
        let source_stats_data = export_data.source_intelligence_stats;
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA busy_timeout = 10000;")?;
            let tx = conn.unchecked_transaction()?;

            {
                let mut source_stmt = tx.prepare(
                    "INSERT OR REPLACE INTO video_sources
                     (source_key, name, api, detail, from_type, disabled, is_adult, sort_order, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                )?;
                for source in video_sources_data {
                    source_stmt.execute(params![
                        source.source_key,
                        source.name,
                        source.api,
                        source.detail,
                        source.from_type,
                        source.disabled,
                        source.is_adult,
                        source.sort_order,
                        source.created_at,
                        source.updated_at,
                    ])?;
                }
            }

            {
                let mut stmt = tx.prepare("INSERT OR IGNORE INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")?;
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
            }

            {
                let mut stmt = tx.prepare("INSERT OR IGNORE INTO favorites (key, title, source_name, year, cover, episode_index, total_episodes, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")?;
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
            }

            {
                let mut stmt = tx.prepare(
                    "INSERT OR IGNORE INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                )?;
                for record in search_history_data {
                    stmt.execute(params![record.keyword, record.save_time,])?;
                }
            }

            {
                let mut stmt = tx.prepare("INSERT OR IGNORE INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)")?;
                for record in skip_configs_data {
                    stmt.execute(params![
                        record.key,
                        record.enable,
                        record.intro_time,
                        record.outro_time,
                    ])?;
                }
            }

            {
                let mut stats_stmt = tx.prepare(
                    "INSERT OR REPLACE INTO source_intelligence_stats
                     (source_key, total_tests, successful_tests, total_response_time_ms, last_success_time, last_failure_time,
                      last_available_time, consecutive_failures, auto_degraded, recent_results_json, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )?;
                for stat in source_stats_data {
                    stats_stmt.execute(params![
                        stat.source_key,
                        stat.total_tests,
                        stat.successful_tests,
                        stat.total_response_time_ms,
                        stat.last_success_time,
                        stat.last_failure_time,
                        stat.last_available_time,
                        stat.consecutive_failures,
                        stat.auto_degraded,
                        stat.recent_results_json,
                        stat.updated_at,
                    ])?;
                }
            }

            tx.commit()?;
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
            conn.execute("DELETE FROM content_pool", [])?;
            conn.execute("DELETE FROM source_intelligence_stats", [])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open in-memory db");

        // Create tables
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

            CREATE TABLE search_history (
              keyword TEXT PRIMARY KEY,
              save_time INTEGER
            );

            CREATE TABLE skip_configs (
              key TEXT PRIMARY KEY,
              enable INTEGER DEFAULT 0,
              intro_time REAL DEFAULT 0,
              outro_time REAL DEFAULT 0
            );

            CREATE TABLE video_sources (
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

            CREATE TABLE source_intelligence_stats (
              source_key TEXT PRIMARY KEY,
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

            CREATE TABLE content_pool (
              id TEXT PRIMARY KEY,
              data TEXT
            );
            "#,
        )
        .expect("init schema");

        Db::new(conn)
    }

    #[test]
    fn db_new_creates_instance() {
        let conn = Connection::open_in_memory().expect("open db");
        let db = Db::new(conn);

        // Verify we can access it
        let result = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM sqlite_master", [], |row| {
                row.get::<_, i32>(0)
            })
        });

        assert!(result.is_ok());
    }

    #[test]
    fn with_conn_executes_closure_successfully() {
        let db = setup_test_db();

        let result = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, save_time) VALUES (?1, ?2, ?3, ?4)",
                params!["key1", "title1", "source1", 1000000],
            )
        });

        assert!(result.is_ok());
    }

    #[test]
    fn with_conn_returns_result_from_closure() {
        let db = setup_test_db();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, save_time) VALUES (?1, ?2, ?3, ?4)",
                params!["key1", "title1", "source1", 1000000],
            )
        })
        .unwrap();

        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn export_json_empty_database() {
        let db = setup_test_db();

        let export_result = db.export_json();
        assert!(export_result.is_ok());

        let json_data = export_result.unwrap();
        let json_str = String::from_utf8(json_data).unwrap();
        let export_data: ExportData = serde_json::from_str(&json_str).unwrap();

        assert_eq!(export_data.play_records.len(), 0);
        assert_eq!(export_data.favorites.len(), 0);
        assert_eq!(export_data.search_history.len(), 0);
        assert_eq!(export_data.skip_configs.len(), 0);
        assert_eq!(export_data.video_sources.len(), 0);
        assert_eq!(export_data.source_intelligence_stats.len(), 0);
    }

    #[test]
    fn export_json_with_play_records() {
        let db = setup_test_db();

        // Insert play records
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params!["k1", "Title 1", "source1", "2024", "cover1", 1, 12, 1800, 3600, 1000000, "search1"],
            )
        })
        .unwrap();

        let json_data = db.export_json().unwrap();
        let json_str = String::from_utf8(json_data).unwrap();
        let export_data: ExportData = serde_json::from_str(&json_str).unwrap();

        assert_eq!(export_data.play_records.len(), 1);
        assert_eq!(export_data.play_records[0].key, "k1");
        assert_eq!(export_data.play_records[0].title, "Title 1");
    }

    #[test]
    fn export_json_with_all_data_types() {
        let db = setup_test_db();

        // Insert all types of data with proper field values
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params!["pk1", "pTitle", "psource", "2024", "pcover", 1, 12, 1800, 3600, 1000000, "search"],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO favorites (key, title, source_name, year, cover, episode_index, total_episodes, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params!["fk1", "fTitle", "fsource", "2024", "fcover", 1, 12, 2000000, "fsearch"],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 3000000],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)",
                params!["sk1", 1, 10.5, -5.0],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO video_sources (source_key, name, api, detail, from_type, disabled, is_adult, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '', 'custom', 0, 0, 0, 1, 1)",
                params!["vs1", "Source 1", "https://source1.example.com"],
            )?;
            conn.execute(
                "INSERT INTO source_intelligence_stats (source_key, total_tests, successful_tests, total_response_time_ms, updated_at)
                 VALUES (?1, 3, 2, 1500, 1)",
                params!["vs1"],
            )?;
            Ok(())
        })
        .unwrap();

        let json_data = db.export_json().unwrap();
        let json_str = String::from_utf8(json_data).unwrap();
        let export_data: ExportData = serde_json::from_str(&json_str).unwrap();

        assert_eq!(export_data.play_records.len(), 1);
        assert_eq!(export_data.favorites.len(), 1);
        assert_eq!(export_data.search_history.len(), 1);
        assert_eq!(export_data.skip_configs.len(), 1);
        assert_eq!(export_data.video_sources.len(), 1);
        assert_eq!(export_data.source_intelligence_stats.len(), 1);
    }

    #[test]
    fn import_json_inserts_all_records() {
        let db = setup_test_db();

        let json_data = r#"{
            "play_records": [
                {
                    "key": "pk1",
                    "title": "pTitle",
                    "source_name": "psource",
                    "year": "2024",
                    "cover": "pcover",
                    "episode_index": 1,
                    "total_episodes": 12,
                    "play_time": 1800,
                    "total_time": 3600,
                    "save_time": 1000000,
                    "search_title": "search"
                }
            ],
            "favorites": [
                {
                    "key": "fk1",
                    "title": "fTitle",
                    "source_name": "fsource",
                    "year": "2024",
                    "cover": "fcover",
                    "episode_index": 2,
                    "total_episodes": 13,
                    "save_time": 2000000,
                    "search_title": "fsearch"
                }
            ],
            "search_history": [
                {
                    "keyword": "anime",
                    "save_time": 3000000
                }
            ],
            "skip_configs": [
                {
                    "key": "sk1",
                    "enable": 1,
                    "intro_time": 10.5,
                    "outro_time": -5.0
                }
            ],
            "video_sources": [
                {
                    "source_key": "vs1",
                    "name": "Source 1",
                    "api": "https://source1.example.com",
                    "detail": "",
                    "from_type": "custom",
                    "disabled": 0,
                    "is_adult": 0,
                    "sort_order": 0,
                    "created_at": 1,
                    "updated_at": 1
                }
            ],
            "source_intelligence_stats": [
                {
                    "source_key": "vs1",
                    "total_tests": 3,
                    "successful_tests": 2,
                    "total_response_time_ms": 1500,
                    "last_success_time": null,
                    "last_failure_time": null,
                    "last_available_time": null,
                    "consecutive_failures": 0,
                    "auto_degraded": 0,
                    "recent_results_json": "[]",
                    "updated_at": 1
                }
            ]
        }"#;

        let import_result = db.import_json(json_data.to_string());
        assert!(import_result.is_ok());

        // Verify records were imported
        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);

        let fav_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(fav_count, 1);

        let hist_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(hist_count, 1);

        let skip_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM skip_configs", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(skip_count, 1);

        let source_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM video_sources", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(source_count, 1);

        let stats_count: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM source_intelligence_stats",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(stats_count, 1);
    }

    #[test]
    fn import_json_skips_duplicates() {
        let db = setup_test_db();

        // Insert initial record
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, save_time) VALUES (?1, ?2, ?3, ?4)",
                params!["pk1", "Original", "source", 1000000],
            )
        })
        .unwrap();

        // Import with same key (should be ignored due to INSERT OR IGNORE)
        let json_data = r#"{
            "play_records": [
                {
                    "key": "pk1",
                    "title": "Imported",
                    "source_name": "newsource",
                    "year": "2024",
                    "cover": "cover",
                    "episode_index": 1,
                    "total_episodes": 12,
                    "play_time": 1800,
                    "total_time": 3600,
                    "save_time": 2000000,
                    "search_title": "search"
                }
            ],
            "favorites": [],
            "search_history": [],
            "skip_configs": [],
            "video_sources": [],
            "source_intelligence_stats": []
        }"#;

        db.import_json(json_data.to_string()).unwrap();

        // Verify original record is unchanged
        let title: String = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT title FROM play_records WHERE key = ?1",
                    params!["pk1"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(title, "Original");
    }

    #[test]
    fn clear_cache_removes_all_tables() {
        let db = setup_test_db();

        // Insert data into all tables
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, save_time) VALUES (?1, ?2, ?3, ?4)",
                params!["pk1", "pTitle", "psource", 1000000],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO favorites (key, title, source_name, save_time) VALUES (?1, ?2, ?3, ?4)",
                params!["fk1", "fTitle", "fsource", 2000000],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 3000000],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)",
                params!["sk1", 1, 10.5, -5.0],
            )
        })
        .unwrap();

        // Clear cache
        let clear_result = db.clear_cache();
        assert!(clear_result.is_ok());

        // Verify all tables are empty
        let p_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(p_count, 0);

        let f_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(f_count, 0);

        let h_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(h_count, 0);

        let s_count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM skip_configs", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(s_count, 0);
    }

    #[test]
    fn export_and_reimport_preserves_data() {
        let db = setup_test_db();

        // Insert original data
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO play_records (key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params!["pk1", "Title 1", "source1", "2024", "cover1", 1, 12, 1800, 3600, 1000000, "search1"],
            )
        })
        .unwrap();

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO search_history (keyword, save_time) VALUES (?1, ?2)",
                params!["anime", 3000000],
            )
        })
        .unwrap();

        // Export
        let json_data = db.export_json().unwrap();

        // Clear and reimport
        db.clear_cache().unwrap();
        let json_str = String::from_utf8(json_data).unwrap();
        db.import_json(json_str).unwrap();

        // Verify data was restored
        let title: String = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT title FROM play_records WHERE key = ?1",
                    params!["pk1"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(title, "Title 1");

        let keyword: String = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT keyword FROM search_history WHERE keyword = ?1",
                    params!["anime"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(keyword, "anime");
    }

    #[test]
    fn import_json_with_invalid_json() {
        let db = setup_test_db();

        let result = db.import_json("not valid json".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn import_json_with_empty_arrays() {
        let db = setup_test_db();

        let json_data = r#"{
            "play_records": [],
            "favorites": [],
            "search_history": [],
            "skip_configs": [],
            "video_sources": [],
            "source_intelligence_stats": []
        }"#;

        let result = db.import_json(json_data.to_string());
        assert!(result.is_ok());

        // Verify nothing was inserted
        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
            })
            .unwrap();
        assert_eq!(count, 0);
    }
}
