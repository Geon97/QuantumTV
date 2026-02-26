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

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplySkipConfigRequest {
    pub source: String,
    pub id: String,
    pub enable: bool,
    pub intro_time: f64,
    pub outro_time: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySkipConfigResponse {
    pub deleted: bool,
}

fn should_delete_skip_config(enable: bool, intro_time: f64, outro_time: f64) -> bool {
    !enable && intro_time == 0.0 && outro_time == 0.0
}

fn apply_skip_config_inner(
    db: &Db,
    request: ApplySkipConfigRequest,
) -> Result<ApplySkipConfigResponse, String> {
    let key = format!("{}+{}", request.source, request.id);
    if should_delete_skip_config(request.enable, request.intro_time, request.outro_time) {
        db.with_conn(|conn| {
            conn.execute("DELETE FROM skip_configs WHERE key = ?1", params![key])?;
            Ok(())
        })?;
        return Ok(ApplySkipConfigResponse { deleted: true });
    }

    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)",
            params![
                key,
                if request.enable { 1 } else { 0 },
                request.intro_time,
                request.outro_time,
            ],
        )?;
        Ok(())
    })?;

    Ok(ApplySkipConfigResponse { deleted: false })
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

#[tauri::command]
pub fn apply_skip_config(
    request: ApplySkipConfigRequest,
    db: State<'_, Db>,
) -> Result<ApplySkipConfigResponse, String> {
    apply_skip_config_inner(&db, request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE skip_configs (
                key TEXT PRIMARY KEY,
                enable INTEGER DEFAULT 0,
                intro_time REAL DEFAULT 0,
                outro_time REAL DEFAULT 0
            );
            "#,
        )
        .expect("init schema");
        Db::new(conn)
    }

    #[test]
    fn should_delete_skip_config_when_disabled_and_empty() {
        assert!(should_delete_skip_config(false, 0.0, 0.0));
        assert!(!should_delete_skip_config(true, 0.0, 0.0));
        assert!(!should_delete_skip_config(false, 1.0, 0.0));
    }

    #[test]
    fn apply_skip_config_deletes_when_empty() {
        let db = setup_test_db();
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skip_configs (key, enable, intro_time, outro_time) VALUES (?1, ?2, ?3, ?4)",
                params!["s1+1", 1, 10.0, -5.0],
            )?;
            Ok(())
        })
        .unwrap();

        let response = apply_skip_config_inner(
            &db,
            ApplySkipConfigRequest {
                source: "s1".to_string(),
                id: "1".to_string(),
                enable: false,
                intro_time: 0.0,
                outro_time: 0.0,
            },
        )
        .unwrap();

        assert!(response.deleted);
        let count: i32 = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM skip_configs WHERE key = ?1",
                    params!["s1+1"],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn apply_skip_config_saves_when_enabled_or_times_present() {
        let db = setup_test_db();
        let response = apply_skip_config_inner(
            &db,
            ApplySkipConfigRequest {
                source: "s1".to_string(),
                id: "1".to_string(),
                enable: true,
                intro_time: 12.0,
                outro_time: -5.0,
            },
        )
        .unwrap();

        assert!(!response.deleted);
        let (enable, intro, outro): (i32, f64, f64) = db
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT enable, intro_time, outro_time FROM skip_configs WHERE key = ?1",
                    params!["s1+1"],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
            })
            .unwrap();
        assert_eq!(enable, 1);
        assert!((intro - 12.0).abs() < 0.01);
        assert!((outro + 5.0).abs() < 0.01);
    }
}
