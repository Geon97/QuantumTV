use crate::storage::StorageManager;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_config(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.config)
}

#[tauri::command]
pub async fn save_config(config: Value, state: State<'_, StorageManager>) -> Result<(), String> {
    state.update_config(config)
}

#[tauri::command]
pub async fn reset_config(state: State<'_, StorageManager>) -> Result<(), String> {
    state.reset_config()
}
