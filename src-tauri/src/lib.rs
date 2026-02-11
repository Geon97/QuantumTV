mod commands;
mod db;
mod storage;

use db::db_client;
use db::db_init;
use db::image_cache::ImageCacheManager;
use db::page_cache::PageCacheManager;
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
            // 注册老板键
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let boss_key_shortcut =
                    // Ctrl+Alt+X
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyX);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |app, shortcut, event| {
                            if shortcut == &boss_key_shortcut
                                && event.state() == ShortcutState::Pressed
                            {
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(true) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(boss_key_shortcut)?;
            }
            app.manage(StorageManager::new(app.handle()));
            app.manage(commands::video::VideoCacheManager::new());
            app.manage(commands::video::SearchCacheManager::new());
            let conn = db_init::init_db(app.handle());
            let db = db_client::Db::new(conn);
            app.manage(db);

            // 初始化图片缓存管理器
            let cache_conn = db_init::init_db(app.handle());
            let image_cache_manager = ImageCacheManager::new(cache_conn);
            image_cache_manager
                .init_table()
                .expect("failed to init image cache table");
            app.manage(image_cache_manager);

            // 初始化页面缓存管理器
            let page_cache_conn = db_init::init_db(app.handle());
            let page_cache_manager = PageCacheManager::new(page_cache_conn);
            page_cache_manager
                .init_table()
                .expect("failed to init page cache table");
            app.manage(page_cache_manager);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::config::get_config_data,
            commands::config::get_fluid_search,
            commands::config::set_fluid_search,
            commands::search::get_search_suggestions,
            // 视频
            commands::video::search,
            commands::video::get_video_detail,
            commands::video::get_video_detail_optimized,
            commands::video::proxy_image,
            commands::video::fetch_url,
            commands::video::fetch_binary,
            commands::video::get_douban_data,
            commands::video::prefer_best_source_command,
            commands::video::test_video_source_command,
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
            // 图片缓存
            db::image_cache::get_cached_image,
            db::image_cache::save_cached_image,
            // 页面缓存
            db::page_cache::get_page_cache,
            db::page_cache::set_page_cache,
            db::page_cache::delete_page_cache,
            db::page_cache::cleanup_expired_page_cache,
            db::page_cache::clear_all_page_cache,
            db::page_cache::get_page_cache_stats,
            // 番
            commands::bangumi::get_bangumi_calendar_data,
            // douban
            commands::douban_client::get_douban_categories,
            commands::douban_client::fetch_douban_list,
            commands::douban_client::get_douban_recommends,
            commands::douban_client::get_douban_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
