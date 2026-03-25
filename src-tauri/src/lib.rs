mod commands;
mod db;
mod scheduler;
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
    env_logger::init();
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
            app.manage(commands::search::SearchResultCache::new());
            app.manage(commands::search::FilterResultCache::new());
            app.manage(commands::recommendation::RecommendationEngine::new());
            app.manage(commands::analytics::AnalyticsEngine::new());
            let conn = db_init::init_db(app.handle());
            let db = db_client::Db::new(conn);
            let source_intelligence_manager =
                commands::source_intelligence::SourceIntelligenceManager::new();
            if let Err(error) = source_intelligence_manager.load_from_db(&db) {
                log::warn!("failed to load source intelligence stats: {}", error);
            }
            app.manage(source_intelligence_manager);

            // 启动时修复空元数据（回填 image_cache、推断 content_pool category）
            if let Ok(stats) = commands::recommendation::fix_empty_metadata_direct(&db) {
                if !stats.is_empty() {
                    log::info!("启动修复元数据: {}", stats);
                }
            }

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
            let _ = page_cache_manager.cleanup_expired();

            app.manage(page_cache_manager);

            // 启动所有后台任务
            scheduler::start_background_tasks(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::config::get_config,
            commands::config::parse_admin_config,
            commands::config::fetch_subscription_config,
            commands::config::parse_subscription_config,
            commands::config::pull_subscription_config,
            commands::config::save_subscription_config,
            commands::config::update_source_config,
            commands::config::update_custom_categories,
            commands::config::save_admin_config_from_json,
            commands::config::update_subscription_settings,
            commands::config::get_config_with_defaults,
            commands::config::admin_apply_source_config,
            commands::config::admin_apply_custom_category,
            commands::config::normalize_source_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::config::get_config_data,
            commands::config::get_player_config,
            commands::config::set_player_config,
            commands::config::update_player_config,
            commands::config::get_user_preferences,
            commands::config::get_runtime_config,
            commands::config::set_user_preferences,
            commands::config::update_user_preferences,
            commands::search::get_search_suggestions,
            commands::search::build_search_page_state,
            commands::search::get_search_page_bootstrap,
            commands::search::search_page_query,
            commands::search::search_page_open,
            commands::search::apply_search_filter,
            commands::search::get_search_cache_stats,
            commands::settings::get_settings_bootstrap,
            commands::config::is_adult_source,
            // 跳过片头片尾
            commands::skip::check_skip_action,
            // 预载下一集
            commands::preload::preload_next_episode_if_needed,
            // 视频
            commands::video::search,
            commands::video::get_video_detail,
            commands::video::get_video_detail_optimized,
            commands::video::change_play_source,
            commands::video::save_play_progress,
            commands::video::initialize_player_by_query,
            commands::video::initialize_player_view,
            commands::video::get_cache_stats,
            commands::video::proxy_image,
            commands::video::fetch_binary,
            commands::video::fetch_m3u8,
            commands::video::get_douban_data,
            commands::video::get_source_categories,
            commands::video::get_source_videos_by_type,
            commands::video::prefer_best_source_command,
            commands::video::test_video_source_command,
            commands::video::player_tick,
            // 版本
            commands::version::get_current_version,
            commands::version::version_for_updates,
            commands::version_check::check_for_updates,
            // 收藏
            db::play_favorite::get_play_favorites,
            db::play_favorite::get_play_favorite_statuses,
            db::play_favorite::save_play_favorite,
            db::play_favorite::delete_play_favorite,
            db::play_favorite::toggle_play_favorite,
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
            db::play_skip::apply_skip_config,
            // 导入导出清除
            db::db_handlers::export_json,
            db::db_handlers::import_json,
            db::db_handlers::import_json_bytes,
            db::db_handlers::clear_cache,
            // 图片缓存
            db::image_cache::get_cached_image,
            db::image_cache::save_cached_image,
            db::image_cache::clear_image_cache,
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
            commands::douban_client::get_douban_page_data,
            commands::douban_client::get_douban_page_state,
            commands::douban_client::load_more_douban_page,
            commands::douban_client::get_filtered_source_categories,
            commands::douban_client::get_douban_defaults,
            commands::home::get_home_data,
            commands::home::get_home_bootstrap,
            commands::home::get_favorite_cards,
            commands::home::get_continue_watching,
            // 源智能选择
            commands::source_intelligence::record_source_test,
            commands::source_intelligence::get_all_source_stats,
            commands::source_intelligence::get_source_stats,
            commands::source_intelligence::rank_sources,
            commands::source_intelligence::clear_all_source_stats,
            // 智能推荐
            commands::recommendation::get_recommendations,
            commands::recommendation::clear_recommendation_cache,
            commands::recommendation::add_to_content_pool,
            commands::recommendation::batch_add_to_content_pool,
            commands::recommendation::update_image_cache_metadata,
            commands::recommendation::fix_empty_metadata,
            // 统计分析
            commands::analytics::get_user_behavior_stats,
            commands::analytics::get_popular_items,
            commands::analytics::get_watch_trends,
            commands::analytics::get_category_stats,
            commands::analytics::get_hourly_stats,
            commands::analytics::generate_analytics_report,
            commands::analytics::clear_analytics_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
