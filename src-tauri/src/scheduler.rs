use crate::commands::config::{
    fetch_subscription_text, format_rfc3339_utc_now, validate_subscription_json,
};
use crate::commands::recommendation::RecommendationEngine;
use crate::db::db_client::Db;
use crate::db::image_cache::ImageCacheManager;
use crate::db::page_cache::PageCacheManager;
use crate::storage::StorageManager;
use quantumtv_core::{
    merge_admin_config_with_defaults, parse_admin_config as parse_admin_config_core,
};
use serde_json::Value;
use std::time::Duration;
use tauri::{Emitter, Manager};

/// Spawn all background interval tasks. Call once from `.setup()`.
pub fn start_background_tasks(app: tauri::AppHandle) {
    eprintln!("[调度器] 已启动: 配置订阅(24h), 图像缓存(24h), 页面缓存(24h), 推荐预热(12h)");

    spawn_subscription_auto_update(app.clone());
    spawn_image_cache_cleanup(app.clone());
    spawn_page_cache_cleanup(app.clone());
    spawn_recommendation_preheat(app);
}

/// 任务 1 — 订阅自动更新（每 24 小时）
fn spawn_subscription_auto_update(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        interval.tick().await;

        loop {
            interval.tick().await;
            if let Err(e) = run_subscription_update(&app).await {
                log::warn!("[调度器:配置订阅] error: {}", e);
            }
        }
    });
}

async fn run_subscription_update(app: &tauri::AppHandle) -> Result<(), String> {
    let storage = app.state::<StorageManager>();
    let data = storage.get_data()?;
    let config = merge_admin_config_with_defaults(&data.config);

    let sub = config
        .get("ConfigSubscribtion")
        .cloned()
        .unwrap_or(Value::Null);

    let auto_update = sub
        .get("AutoUpdate")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !auto_update {
        log::debug!("[调度器:配置订阅] 自动更新已禁用，跳过");
        return Ok(());
    }

    let url = sub
        .get("URL")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if url.trim().is_empty() {
        log::debug!("[调度器:配置订阅] 没有配置订阅 URL，跳过");
        return Ok(());
    }

    log::info!("[调度器:配置订阅] 正在获取 {}", url);
    let text = fetch_subscription_text(&url).await?;
    validate_subscription_json(&text)?;

    let parsed = parse_admin_config_core(&text)?;

    let sources = parsed
        .get("SourceConfig")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let categories = parsed
        .get("CustomCategories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut new_config = merge_admin_config_with_defaults(&data.config);
    if let Some(obj) = new_config.as_object_mut() {
        obj.insert("ConfigFile".to_string(), Value::String(text));
        obj.insert("SourceConfig".to_string(), Value::Array(sources));
        obj.insert("CustomCategories".to_string(), Value::Array(categories));

        let sub = obj
            .entry("ConfigSubscribtion".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !sub.is_object() {
            *sub = serde_json::json!({});
        }
        if let Some(sub_obj) = sub.as_object_mut() {
            sub_obj.insert(
                "LastCheck".to_string(),
                Value::String(format_rfc3339_utc_now()),
            );
        }
    }

    storage.update_config(new_config)?;
    let _ = app.emit("configUpdated", ());
    log::info!("[调度器:配置订阅] 配置更新成功");
    Ok(())
}

/// 任务 2 — 图像缓存清理（每 24 小时）
fn spawn_image_cache_cleanup(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        interval.tick().await;

        loop {
            interval.tick().await;
            let manager = app.state::<ImageCacheManager>();
            match manager.cleanup_expired() {
                Ok(deleted) => {
                    log::info!("[调度器:图像缓存] 清理了 {} 个过期的条目", deleted)
                }
                Err(e) => log::warn!("[调度器:图像缓存] 错误: {}", e),
            }
        }
    });
}

/// 任务 3 — 页面缓存清理（每 24 小时）
fn spawn_page_cache_cleanup(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        interval.tick().await;

        loop {
            interval.tick().await;
            let manager = app.state::<PageCacheManager>();
            match manager.cleanup_expired() {
                Ok(deleted) => {
                    log::info!("[调度器:页面缓存] 清理了 {} 个过期的条目", deleted)
                }
                Err(e) => log::warn!("[调度器:页面缓存] 错误: {}", e),
            }
        }
    });
}

/// 任务 4 — 推荐预热（每 12 小时）
fn spawn_recommendation_preheat(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(12 * 3600));
        interval.tick().await;

        loop {
            interval.tick().await;
            let engine = app.state::<RecommendationEngine>();
            let db = app.state::<Db>();
            match engine.get_recommendations(&db).await {
                Ok(items) => {
                    log::info!("[调度器:推荐预热] 预热了 {} 个推荐", items.len())
                }
                Err(e) => log::warn!("[调度器:推荐预热] 错误: {}", e),
            }
        }
    });
}
