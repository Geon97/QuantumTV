mod commands;
mod storage;

use crate::storage::StorageManager;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(StorageManager::new(app.handle()));
            app.manage(commands::video::VideoCacheManager::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::auth::login,
            commands::auth::logout,
            commands::auth::get_current_user,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::db::get_all_play_records,
            commands::db::save_play_record,
            commands::db::get_all_favorites,
            commands::db::save_favorites,
            commands::db::get_search_history,
            commands::db::save_search_history,
            commands::db::get_all_skip_configs,
            commands::db::save_skip_configs,
            commands::search::get_search_suggestions,
            commands::video::search,
            commands::video::get_video_detail,
            commands::video::proxy_image,
            commands::video::fetch_url,
            commands::video::fetch_binary,
            commands::video::get_douban_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
