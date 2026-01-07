use crate::storage::StorageManager;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_all_play_records(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.play_records)
}

#[tauri::command]
pub async fn save_play_record(
    record: Value,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    state.update_play_records(record)
}

#[tauri::command]
pub async fn get_all_favorites(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.favorites)
}

#[tauri::command]
pub async fn save_favorites(
    favorites: Value,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    state.update_favorites(favorites)
}

#[tauri::command]
pub async fn get_search_history(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.search_history)
}

#[tauri::command]
pub async fn save_search_history(
    history: Value,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    state.update_search_history(history)
}

#[tauri::command]
pub async fn get_all_skip_configs(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.skip_configs)
}

#[tauri::command]
pub async fn save_skip_configs(
    configs: Value,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    state.update_skip_configs(configs)
}
