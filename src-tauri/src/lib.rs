mod commands;
mod db;
mod storage;

use db::db_client;
use db::db_init;
use storage::StorageManager;
use tauri::Manager;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(StorageManager::new(app.handle()));
            app.manage(commands::video::VideoCacheManager::new());
            let conn = db_init::init_db(app.handle());
            let db = db_client::Db::new(conn);
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::search::get_search_suggestions,
            // 视频
            commands::video::search,
            commands::video::get_video_detail,
            commands::video::proxy_image,
            commands::video::fetch_url,
            commands::video::fetch_binary,
            commands::video::get_douban_data,
            // 版本
            commands::version::get_current_version,
            commands::version::version_for_updates,
            commands::version_check::check_for_updates,
            // 收藏
            db::play_favorite::get_play_favorites,
            db::play_favorite::save_play_favorite,
            db::play_favorite::delete_play_favorite,
            db::play_favorite::get_play_favorites_by_title,
            db::play_favorite::clear_all_favorites,
            // 播放记录
            db::play_record::get_all_play_records,
            db::play_record::save_play_record,
            db::play_record::delete_play_record,
            db::play_record::clear_all_play_records,
            // 搜索历史
            db::search_history::add_search_history,
            db::search_history::clear_search_history,
            db::search_history::delete_search_history,
            db::search_history::get_search_history,
            // 跳过配置
            db::play_skip::get_skip_config,
            db::play_skip::save_skip_config,
            db::play_skip::delete_skip_config,
            // 导入导出清除
            db::db_handlers::export_json,
            db::db_handlers::import_json,
            db::db_handlers::clear_cache,
            // 番
            commands::bangumi::get_bangumi_calendar_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
