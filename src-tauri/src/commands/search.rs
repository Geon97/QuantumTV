use crate::db::db_client::Db;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_search_suggestions(db: State<'_, Db>, query: String) -> Result<Vec<String>, String> {
    // 获取数据库数据
    db.with_conn(|conn| {
        let like_query = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT keyword FROM search_history WHERE keyword LIKE ?1 ORDER BY save_time DESC LIMIT 10",
            )?;
        let rows = stmt
            .query_map(params![like_query], |row| row.get(0))?;
        let suggestions = rows
            .collect::<Result<Vec<String>, _>>()?;
        Ok(suggestions)
    })
}
