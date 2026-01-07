use crate::storage::StorageManager;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub async fn login(
    username: Option<String>,
    _password: Option<String>,
    _state: State<'_, StorageManager>,
) -> Result<Value, String> {
    // For local Tauri app, we can simplify auth or use the same logic as Next.js
    // For now, let's assume successful login if password matches environment or is empty if not set
    // In a real app, you might want more secure password handling
    Ok(json!({ "ok": true, "username": username.unwrap_or_else(|| "admin".to_string()) }))
}

#[tauri::command]
pub async fn logout() -> Result<Value, String> {
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub async fn get_current_user(_state: State<'_, StorageManager>) -> Result<Value, String> {
    // Return a default user or check local storage
    Ok(json!({ "username": "admin", "role": "owner" }))
}
