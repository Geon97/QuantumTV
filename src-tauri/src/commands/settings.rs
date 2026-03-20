use crate::commands::config::{get_user_preferences, UserPreferences};
use crate::commands::version::get_current_version;
use crate::commands::version_check::{check_for_updates, VersionCheckResult};
use crate::db::page_cache::{CacheStats, PageCacheManager};
use crate::storage::StorageManager;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBootstrapResponse {
    pub user_preferences: UserPreferences,
    pub current_version: String,
    pub version_check: VersionCheckResult,
    pub page_cache_stats: CacheStats,
}

#[tauri::command]
pub async fn get_settings_bootstrap(
    storage: State<'_, StorageManager>,
    cache: State<'_, PageCacheManager>,
) -> Result<SettingsBootstrapResponse, String> {
    let user_preferences = get_user_preferences(storage).await?;
    let current_version = get_current_version();
    let version_check = check_for_updates().await;
    let page_cache_stats = cache.get_stats().map_err(|e| e.to_string())?;

    Ok(SettingsBootstrapResponse {
        user_preferences,
        current_version,
        version_check,
        page_cache_stats,
    })
}
