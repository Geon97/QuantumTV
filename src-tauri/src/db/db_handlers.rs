use crate::db::db_client::Db;
use tauri::State;

#[tauri::command]
pub fn export_json(db: State<'_, Db>) -> Result<Vec<u8>, String> {
    db.export_json()
}

#[tauri::command]
pub fn import_json(db: State<'_, Db>, data: String) -> Result<(), String> {
    db.import_json(data)
}

#[tauri::command]
pub fn clear_cache(db: State<'_, Db>) -> Result<(), String> {
    db.clear_cache()
}
