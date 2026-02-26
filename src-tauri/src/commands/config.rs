use crate::storage::StorageManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;
use quantumtv_core::adult;
use quantumtv_core::default_admin_config_value;
use quantumtv_core::merge_admin_config_with_defaults;
use quantumtv_core::normalize_source_config as normalize_source_config_core;
use quantumtv_core::parse_admin_config as parse_admin_config_core;
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSubscribtion {
    pub url: String,
    pub auto_update: bool,
    pub last_check: String,
}

impl Default for ConfigSubscribtion {
    fn default() -> Self {
        Self {
            url: String::new(),
            auto_update: false,
            last_check: String::new(),
        }
    }
}
/// 旧的 SiteConfig 结构（仅用于迁移时反序列化，不再构造）
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SiteConfig {
    pub site_name: String,
    pub announcement: String,
    pub search_downstream_max_page: u32,
    pub site_interface_cache_time: u32,
    pub douban_proxy_type: String,
    pub douban_proxy: String,
    pub douban_image_proxy_type: String,
    pub douban_image_proxy: String,
    pub disable_yellow_filter: bool,
    pub fluid_search: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    User,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub role: Role,
    pub banned: bool,
    pub enabled_apis: Vec<String>,
    pub tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Tags {
    name: String,
    enabled_apis: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub users: Vec<User>,
    pub tags: Vec<Tags>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            users: vec![],
            tags: vec![],
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum From {
    Config,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Movie,
    TV,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    pub key: String,
    pub name: String,
    pub api: String,
    pub detail: String,
    pub from: From,
    pub disabled: bool,
    pub is_adult: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomCategory {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub query: String,
    pub from: From,
    pub disabled: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminConfig {
    #[serde(default)]
    pub config_subscribtion: ConfigSubscribtion,
    #[serde(default)]
    pub config_file: String,
    #[serde(default)]
    pub user_config: UserConfig,
    pub source_config: Vec<SourceConfig>,
    #[serde(default)]
    pub custom_categories: Vec<CustomCategory>,
}
fn get_local_mode_config() -> AdminConfig {
    AdminConfig {
        config_file: "".to_string(),
        config_subscribtion: ConfigSubscribtion {
            url: "".to_string(),
            auto_update: false,
            last_check: "".to_string(),
        },
        user_config: UserConfig {
            users: vec![User {
                username: "admin".to_string(),
                role: Role::Owner,
                banned: false,
                enabled_apis: vec![],
                tags: vec![],
            }],
            tags: vec![],
        },
        source_config: vec![],
        custom_categories: vec![],
    }
}

#[tauri::command]
pub fn get_config_data() -> &'static AdminConfig {
    static CONFIG: OnceLock<AdminConfig> = OnceLock::new();
    CONFIG.get_or_init(get_local_mode_config)
}

#[tauri::command]
pub async fn get_config(state: State<'_, StorageManager>) -> Result<Value, String> {
    let data = state.get_data()?;
    Ok(data.config)
}

/// 解析管理端订阅配置（Rust 端统一解析逻辑）
#[tauri::command]
pub async fn parse_admin_config(raw_json: String) -> Result<Value, String> {
    parse_admin_config_core(&raw_json)
}

fn validate_subscription_json(raw_json: &str) -> Result<(), String> {
    serde_json::from_str::<Value>(raw_json)
        .map(|_| ())
        .map_err(|_| "返回内容不是有效的 JSON 格式".to_string())
}

fn format_rfc3339_utc_from_secs(secs: i64, nanos: u32) -> String {
    let days = secs.div_euclid(86_400);
    let seconds_of_day = secs.rem_euclid(86_400);
    let hour = (seconds_of_day / 3_600) as u32;
    let minute = ((seconds_of_day % 3_600) / 60) as u32;
    let second = (seconds_of_day % 60) as u32;
    let (year, month, day) = civil_from_days(days);
    let millis = nanos / 1_000_000;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hour, minute, second, millis
    )
}

fn format_rfc3339_utc_now() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_rfc3339_utc_from_secs(duration.as_secs() as i64, duration.subsec_nanos())
}

// Gregorian calendar conversion (days since Unix epoch -> YYYY-MM-DD)
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}

async fn fetch_subscription_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    response.text().await.map_err(|e| e.to_string())
}

async fn resolve_subscription_json(
    subscription_url: Option<&str>,
    raw_json: Option<&str>,
) -> Result<String, String> {
    if let Some(raw) = raw_json {
        if !raw.trim().is_empty() {
            return Ok(raw.to_string());
        }
    }

    if let Some(url) = subscription_url {
        if !url.trim().is_empty() {
            return fetch_subscription_text(url).await;
        }
    }

    Err("配置内容不能为空".to_string())
}

/// 订阅拉取（Rust 端完成 HTTP 拉取 + JSON 校验）
#[tauri::command]
pub async fn fetch_subscription_config(subscription_url: String) -> Result<String, String> {
    let url = subscription_url.trim();
    if url.is_empty() {
        return Err("请输入订阅URL".to_string());
    }

    let text = fetch_subscription_text(url).await?;
    validate_subscription_json(&text)?;

    Ok(text)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionParseResponse {
    pub raw_json: String,
    pub parsed_config: Value,
}

/// 解析订阅配置（支持 URL 或 JSON）
#[tauri::command]
pub async fn parse_subscription_config(
    subscription_url: Option<String>,
    raw_json: Option<String>,
) -> Result<SubscriptionParseResponse, String> {
    let raw_json = resolve_subscription_json(subscription_url.as_deref(), raw_json.as_deref())
        .await?;

    let parsed_config = parse_admin_config_core(&raw_json)?;
    Ok(SubscriptionParseResponse {
        raw_json,
        parsed_config,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPullResponse {
    pub raw_json: String,
    pub config: Value,
}

/// 拉取订阅配置并写入 ConfigFile/LastCheck（不解析 SourceConfig）
#[tauri::command]
pub async fn pull_subscription_config(
    subscription_url: String,
    state: State<'_, StorageManager>,
) -> Result<SubscriptionPullResponse, String> {
    let url = subscription_url.trim();
    if url.is_empty() {
        return Err("请输入订阅URL".to_string());
    }

    let text = fetch_subscription_text(url).await?;
    validate_subscription_json(&text)?;

    let data = state.get_data()?;
    let mut config = merge_admin_config_with_defaults(&data.config);

    if let Some(obj) = config.as_object_mut() {
        obj.insert("ConfigFile".to_string(), Value::String(text.clone()));
        let sub = obj
            .entry("ConfigSubscribtion".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !sub.is_object() {
            *sub = serde_json::json!({});
        }
        let sub_obj = sub.as_object_mut().unwrap();
        sub_obj.insert("URL".to_string(), Value::String(url.to_string()));
        sub_obj.insert(
            "LastCheck".to_string(),
            Value::String(format_rfc3339_utc_now()),
        );
    }

    state.update_config(config.clone())?;

    Ok(SubscriptionPullResponse {
        raw_json: text,
        config,
    })
}

/// 保存订阅配置（解析并更新 SourceConfig/CustomCategories）
#[tauri::command]
pub async fn save_subscription_config(
    subscription_url: Option<String>,
    raw_json: Option<String>,
    auto_update: Option<bool>,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let raw_json =
        resolve_subscription_json(subscription_url.as_deref(), raw_json.as_deref()).await?;

    let parsed_config = parse_admin_config_core(&raw_json)?;
    let sources = parsed_config
        .get("SourceConfig")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone())
        .unwrap_or_default();
    let categories = parsed_config
        .get("CustomCategories")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone())
        .unwrap_or_default();

    let data = state.get_data()?;
    let mut config = merge_admin_config_with_defaults(&data.config);

    if let Some(obj) = config.as_object_mut() {
        obj.insert("ConfigFile".to_string(), Value::String(raw_json));
        obj.insert("SourceConfig".to_string(), Value::Array(sources));
        obj.insert("CustomCategories".to_string(), Value::Array(categories));

        let sub = obj
            .entry("ConfigSubscribtion".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !sub.is_object() {
            *sub = serde_json::json!({});
        }
        let sub_obj = sub.as_object_mut().unwrap();
        if let Some(url) = subscription_url {
            sub_obj.insert("URL".to_string(), Value::String(url));
        }
        if let Some(auto_update) = auto_update {
            sub_obj.insert("AutoUpdate".to_string(), Value::Bool(auto_update));
        }
    }

    state.update_config(config.clone())?;
    Ok(config)
}

/// 更新视频源配置
#[tauri::command]
pub async fn update_source_config(
    sources: Vec<Value>,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let data = state.get_data()?;
    let mut config = merge_admin_config_with_defaults(&data.config);

    if let Some(obj) = config.as_object_mut() {
        obj.insert("SourceConfig".to_string(), Value::Array(sources));
    }

    state.update_config(config.clone())?;
    Ok(config)
}

/// 更新自定义分类配置
#[tauri::command]
pub async fn update_custom_categories(
    categories: Vec<Value>,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let data = state.get_data()?;
    let mut config = merge_admin_config_with_defaults(&data.config);

    if let Some(obj) = config.as_object_mut() {
        obj.insert("CustomCategories".to_string(), Value::Array(categories));
    }

    state.update_config(config.clone())?;
    Ok(config)
}

/// 从 JSON 导入完整配置并保存
#[tauri::command]
pub async fn save_admin_config_from_json(
    raw_json: String,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let parsed_config = parse_admin_config_core(&raw_json)?;
    state.update_config(parsed_config.clone())?;
    Ok(parsed_config)
}

/// 更新订阅设置（仅 URL/AutoUpdate）
#[tauri::command]
pub async fn update_subscription_settings(
    subscription_url: Option<String>,
    auto_update: bool,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let data = state.get_data()?;
    let mut config = merge_admin_config_with_defaults(&data.config);

    if let Some(obj) = config.as_object_mut() {
        let sub = obj
            .entry("ConfigSubscribtion".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !sub.is_object() {
            *sub = serde_json::json!({});
        }
        let sub_obj = sub.as_object_mut().unwrap();
        if let Some(url) = subscription_url {
            sub_obj.insert("URL".to_string(), Value::String(url));
        }
        sub_obj.insert(
            "AutoUpdate".to_string(),
            Value::Bool(auto_update),
        );
    }

    state.update_config(config.clone())?;
    Ok(config)
}

/// 获取配置（自动补全默认字段）
#[tauri::command]
pub async fn get_config_with_defaults(state: State<'_, StorageManager>) -> Result<Value, String> {
    match state.get_data() {
        Ok(data) => Ok(merge_admin_config_with_defaults(&data.config)),
        Err(_) => Ok(default_admin_config_value()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum SourceConfigAction {
    Reorder { active_key: String, over_key: String },
    Toggle { key: String },
    Delete { key: String },
    Add { source: Value },
    Edit { source: Value },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum CustomCategoryAction {
    Toggle { index: usize },
    Delete { index: usize },
    Add { category: Value },
}

fn extract_non_empty_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn apply_source_config_action(config: &Value, action: SourceConfigAction) -> Result<Value, String> {
    let mut merged = merge_admin_config_with_defaults(config);
    let mut sources = merged
        .get("SourceConfig")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match action {
        SourceConfigAction::Reorder { active_key, over_key } => {
            let from_index = sources.iter().position(|item| {
                item.get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == active_key)
                    .unwrap_or(false)
            });
            let to_index = sources.iter().position(|item| {
                item.get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == over_key)
                    .unwrap_or(false)
            });

            if let (Some(from), Some(to)) = (from_index, to_index) {
                if from != to {
                    let item = sources.remove(from);
                    sources.insert(to, item);
                }
            }
        }
        SourceConfigAction::Toggle { key } => {
            for item in &mut sources {
                if item
                    .get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == key)
                    .unwrap_or(false)
                {
                    let disabled = item
                        .get("disabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert("disabled".to_string(), Value::Bool(!disabled));
                    }
                    break;
                }
            }
        }
        SourceConfigAction::Delete { key } => {
            sources.retain(|item| {
                item.get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v != key)
                    .unwrap_or(true)
            });
        }
        SourceConfigAction::Add { source } => {
            if extract_non_empty_string(&source, "key").is_none()
                || extract_non_empty_string(&source, "name").is_none()
                || extract_non_empty_string(&source, "api").is_none()
            {
                return Err("请填写完整信息".to_string());
            }

            let key = extract_non_empty_string(&source, "key").unwrap_or_default();
            if sources.iter().any(|item| {
                item.get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == key)
                    .unwrap_or(false)
            }) {
                return Err("源标识已存在".to_string());
            }

            let mut normalized = normalize_source_config_core(&source, "custom")?;
            if let Some(obj) = normalized.as_object_mut() {
                obj.insert("disabled".to_string(), Value::Bool(false));
            }
            sources.push(normalized);
        }
        SourceConfigAction::Edit { source } => {
            let key = extract_non_empty_string(&source, "key")
                .ok_or_else(|| "缺少源标识".to_string())?;
            let default_from = source
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("custom");

            let mut normalized = normalize_source_config_core(&source, default_from)?;

            if let Some(existing) = sources.iter().find(|item| {
                item.get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == key)
                    .unwrap_or(false)
            }) {
                if let (Some(obj), Some(existing_disabled)) = (
                    normalized.as_object_mut(),
                    existing.get("disabled").and_then(|v| v.as_bool()),
                ) {
                    obj.entry("disabled".to_string())
                        .or_insert_with(|| Value::Bool(existing_disabled));
                }
            }

            let mut replaced = false;
            for item in &mut sources {
                if item
                    .get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v == key)
                    .unwrap_or(false)
                {
                    *item = normalized.clone();
                    replaced = true;
                    break;
                }
            }

            if !replaced {
                return Err("源不存在".to_string());
            }
        }
    }

    if let Some(obj) = merged.as_object_mut() {
        obj.insert("SourceConfig".to_string(), Value::Array(sources));
    }
    Ok(merged)
}

fn apply_custom_category_action(
    config: &Value,
    action: CustomCategoryAction,
) -> Result<Value, String> {
    let mut merged = merge_admin_config_with_defaults(config);
    let mut categories = merged
        .get("CustomCategories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match action {
        CustomCategoryAction::Toggle { index } => {
            if let Some(item) = categories.get_mut(index) {
                let disabled = item
                    .get("disabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("disabled".to_string(), Value::Bool(!disabled));
                }
            }
        }
        CustomCategoryAction::Delete { index } => {
            if index < categories.len() {
                categories.remove(index);
            }
        }
        CustomCategoryAction::Add { category } => {
            if extract_non_empty_string(&category, "name").is_none()
                || extract_non_empty_string(&category, "query").is_none()
            {
                return Err("请填写完整信息".to_string());
            }

            let mut obj = category.as_object().cloned().unwrap_or_default();
            obj.insert("from".to_string(), Value::String("custom".to_string()));
            obj.insert("disabled".to_string(), Value::Bool(false));
            categories.push(Value::Object(obj));
        }
    }

    if let Some(obj) = merged.as_object_mut() {
        obj.insert("CustomCategories".to_string(), Value::Array(categories));
    }
    Ok(merged)
}

/// Admin: apply source config action in Rust, update storage and return config.
#[tauri::command]
pub async fn admin_apply_source_config(
    action: SourceConfigAction,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let data = state.get_data()?;
    let updated = apply_source_config_action(&data.config, action)?;
    state.update_config(updated.clone())?;
    Ok(updated)
}

/// Admin: apply custom category action in Rust, update storage and return config.
#[tauri::command]
pub async fn admin_apply_custom_category(
    action: CustomCategoryAction,
    state: State<'_, StorageManager>,
) -> Result<Value, String> {
    let data = state.get_data()?;
    let updated = apply_custom_category_action(&data.config, action)?;
    state.update_config(updated.clone())?;
    Ok(updated)
}

/// 规范化单个视频源配置（Rust 端统一成人检测与字段补全）
#[tauri::command]
pub async fn normalize_source_config(
    source: Value,
    default_from: Option<String>,
) -> Result<Value, String> {
    let from = default_from.as_deref().unwrap_or("custom");
    normalize_source_config_core(&source, from)
}

#[tauri::command]
pub async fn save_config(config: Value, state: State<'_, StorageManager>) -> Result<(), String> {
    state.update_config(config)
}

#[tauri::command]
pub async fn reset_config(state: State<'_, StorageManager>) -> Result<(), String> {
    state.reset_config()
}

fn player_config_from_config(config: &Value) -> PlayerConfig {
    if let Some(player_config) = config.get("PlayerConfig") {
        if let Ok(config) = serde_json::from_value::<PlayerConfig>(player_config.clone()) {
            return config;
        }
    }

    PlayerConfig::default()
}

/// 获取播放器配置（去广告、优选等）
#[tauri::command]
pub async fn get_player_config(state: State<'_, StorageManager>) -> Result<PlayerConfig, String> {
    let data = state.get_data()?;

    Ok(player_config_from_config(&data.config))
}

/// 保存播放器配置
#[tauri::command]
pub async fn set_player_config(
    config: PlayerConfig,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    let mut data = state.get_data()?;

    // 确保配置结构存在
    if !data.config.is_object() {
        data.config = serde_json::json!({});
    }

    let config_obj = data.config.as_object_mut().unwrap();

    // 保存播放器配置
    config_obj.insert(
        "PlayerConfig".to_string(),
        serde_json::to_value(config).map_err(|e| e.to_string())?,
    );

    state.update_config(data.config)
}

/// 播放器配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerConfig {
    /// 去广告开关
    pub block_ad_enabled: bool,
    /// 优选开关
    pub optimization_enabled: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            block_ad_enabled: true,
            optimization_enabled: true,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PlayerConfigPatch {
    pub block_ad_enabled: Option<bool>,
    pub optimization_enabled: Option<bool>,
}

fn apply_player_config_patch(mut config: PlayerConfig, patch: PlayerConfigPatch) -> PlayerConfig {
    if let Some(value) = patch.block_ad_enabled {
        config.block_ad_enabled = value;
    }
    if let Some(value) = patch.optimization_enabled {
        config.optimization_enabled = value;
    }
    config
}

/// Update player config (partial).
#[tauri::command]
pub async fn update_player_config(
    config: PlayerConfigPatch,
    state: State<'_, StorageManager>,
) -> Result<PlayerConfig, String> {
    let mut data = state.get_data()?;
    let current = player_config_from_config(&data.config);
    let updated = apply_player_config_patch(current, config);

    if !data.config.is_object() {
        data.config = serde_json::json!({});
    }

    let config_obj = data.config.as_object_mut().unwrap();
    config_obj.insert(
        "PlayerConfig".to_string(),
        serde_json::to_value(updated.clone()).map_err(|e| e.to_string())?,
    );

    state.update_config(data.config)?;
    Ok(updated)
}

/// 用户偏好配置结构（统一配置，包含原 SiteConfig 字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UserPreferences {
    // 应用基础设置（原 SiteConfig）
    /// 站点名称
    pub site_name: String,
    /// 公告内容
    pub announcement: String,
    /// 搜索下游最大页数
    pub search_downstream_max_page: u32,
    /// 站点接口缓存时间（秒）
    pub site_interface_cache_time: u32,
    /// 是否禁用黄色过滤
    pub disable_yellow_filter: bool,

    // 豆瓣设置
    /// 豆瓣数据源类型
    pub douban_data_source: String,
    /// 豆瓣代理URL
    pub douban_proxy_url: String,
    /// 豆瓣图片代理类型
    pub douban_image_proxy_type: String,
    /// 豆瓣图片代理URL
    pub douban_image_proxy_url: String,

    // 用户偏好设置
    /// 是否启用优选和测速
    pub enable_optimization: bool,
    /// 是否启用流式搜索
    pub fluid_search: bool,
    /// 播放器缓冲模式
    pub player_buffer_mode: String,
    /// 已查看的公告内容
    pub has_seen_announcement: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            // 应用基础设置默认值
            site_name: "QuantumTV".to_string(),
            announcement: "本应用仅提供影视信息搜索服务，所有内容均来自第三方网站。".to_string(),
            search_downstream_max_page: 5,
            site_interface_cache_time: 7200,
            disable_yellow_filter: false,

            // 豆瓣设置默认值
            douban_data_source: "cmliussss-cdn-tencent".to_string(),
            douban_proxy_url: String::new(),
            douban_image_proxy_type: "cmliussss-cdn-tencent".to_string(),
            douban_image_proxy_url: String::new(),

            // 用户偏好设置默认值
            enable_optimization: true,
            fluid_search: true,
            player_buffer_mode: "standard".to_string(),
            has_seen_announcement: String::new(),
        }
    }
}
#[derive(Debug, Deserialize, Default)]
pub struct UserPreferencesPatch {
    pub site_name: Option<String>,
    pub announcement: Option<String>,
    pub search_downstream_max_page: Option<u32>,
    pub site_interface_cache_time: Option<u32>,
    pub disable_yellow_filter: Option<bool>,
    pub douban_data_source: Option<String>,
    pub douban_proxy_url: Option<String>,
    pub douban_image_proxy_type: Option<String>,
    pub douban_image_proxy_url: Option<String>,
    pub enable_optimization: Option<bool>,
    pub fluid_search: Option<bool>,
    pub player_buffer_mode: Option<String>,
    pub has_seen_announcement: Option<String>,
}

fn apply_user_preferences_patch(
    mut preferences: UserPreferences,
    patch: UserPreferencesPatch,
) -> UserPreferences {
    if let Some(value) = patch.site_name {
        preferences.site_name = value;
    }
    if let Some(value) = patch.announcement {
        preferences.announcement = value;
    }
    if let Some(value) = patch.search_downstream_max_page {
        preferences.search_downstream_max_page = value;
    }
    if let Some(value) = patch.site_interface_cache_time {
        preferences.site_interface_cache_time = value;
    }
    if let Some(value) = patch.disable_yellow_filter {
        preferences.disable_yellow_filter = value;
    }
    if let Some(value) = patch.douban_data_source {
        preferences.douban_data_source = value;
    }
    if let Some(value) = patch.douban_proxy_url {
        preferences.douban_proxy_url = value;
    }
    if let Some(value) = patch.douban_image_proxy_type {
        preferences.douban_image_proxy_type = value;
    }
    if let Some(value) = patch.douban_image_proxy_url {
        preferences.douban_image_proxy_url = value;
    }
    if let Some(value) = patch.enable_optimization {
        preferences.enable_optimization = value;
    }
    if let Some(value) = patch.fluid_search {
        preferences.fluid_search = value;
    }
    if let Some(value) = patch.player_buffer_mode {
        preferences.player_buffer_mode = value;
    }
    if let Some(value) = patch.has_seen_announcement {
        preferences.has_seen_announcement = value;
    }
    preferences
}

fn user_preferences_from_config(config: &Value) -> UserPreferences {
    if let Some(user_prefs) = config.get("UserPreferences") {
        if let Ok(prefs) = serde_json::from_value::<UserPreferences>(user_prefs.clone()) {
            return prefs;
        }
    }

    let mut prefs = UserPreferences::default();

    if let Some(site_config) = config.get("SiteConfig") {
        if let Some(site_name) = site_config.get("SiteName").and_then(|v| v.as_str()) {
            prefs.site_name = site_name.to_string();
        }
        if let Some(announcement) = site_config.get("Announcement").and_then(|v| v.as_str()) {
            prefs.announcement = announcement.to_string();
        }
        if let Some(max_page) = site_config.get("SearchDownstreamMaxPage").and_then(|v| v.as_u64()) {
            prefs.search_downstream_max_page = max_page as u32;
        }
        if let Some(cache_time) = site_config.get("SiteInterfaceCacheTime").and_then(|v| v.as_u64()) {
            prefs.site_interface_cache_time = cache_time as u32;
        }
        if let Some(disable_filter) = site_config.get("DisableYellowFilter").and_then(|v| v.as_bool()) {
            prefs.disable_yellow_filter = disable_filter;
        }
        if let Some(douban_type) = site_config.get("DoubanProxyType").and_then(|v| v.as_str()) {
            prefs.douban_data_source = douban_type.to_string();
        }
        if let Some(douban_proxy) = site_config.get("DoubanProxy").and_then(|v| v.as_str()) {
            prefs.douban_proxy_url = douban_proxy.to_string();
        }
        if let Some(image_type) = site_config.get("DoubanImageProxyType").and_then(|v| v.as_str()) {
            prefs.douban_image_proxy_type = image_type.to_string();
        }
        if let Some(image_proxy) = site_config.get("DoubanImageProxy").and_then(|v| v.as_str()) {
            prefs.douban_image_proxy_url = image_proxy.to_string();
        }
        if let Some(fluid_search) = site_config.get("FluidSearch").and_then(|v| v.as_bool()) {
            prefs.fluid_search = fluid_search;
        }
    }

    prefs
}



/// 获取用户偏好配置（统一配置，自动从 SiteConfig 迁移）
#[tauri::command]
pub async fn get_user_preferences(state: State<'_, StorageManager>) -> Result<UserPreferences, String> {
    let data = state.get_data()?;
    Ok(user_preferences_from_config(&data.config))
}

/// 保存用户偏好配置
#[tauri::command]
pub async fn set_user_preferences(
    preferences: UserPreferences,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    let mut data = state.get_data()?;

    // 确保配置结构存在
    if !data.config.is_object() {
        data.config = serde_json::json!({});
    }

    let config_obj = data.config.as_object_mut().unwrap();

    // 保存用户偏好配置
    config_obj.insert(
        "UserPreferences".to_string(),
        serde_json::to_value(preferences).map_err(|e| e.to_string())?,
    );

    state.update_config(data.config)
}

/// Update user preferences (partial).
#[tauri::command]
pub async fn update_user_preferences(
    preferences: UserPreferencesPatch,
    state: State<'_, StorageManager>,
) -> Result<UserPreferences, String> {
    let mut data = state.get_data()?;
    let current = user_preferences_from_config(&data.config);
    let updated = apply_user_preferences_patch(current, preferences);

    if !data.config.is_object() {
        data.config = serde_json::json!({});
    }

    let config_obj = data.config.as_object_mut().unwrap();
    config_obj.insert(
        "UserPreferences".to_string(),
        serde_json::to_value(updated.clone()).map_err(|e| e.to_string())?,
    );

    state.update_config(data.config)?;
    Ok(updated)
}

// 是否为成人源 批量判断
#[tauri::command]
pub async fn is_adult_source(names: Vec<String>) -> Vec<bool> {
    names.iter().map(|n| adult::is_adult_source(n)).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        apply_custom_category_action,
        apply_source_config_action,
        apply_player_config_patch,
        apply_user_preferences_patch,
        format_rfc3339_utc_from_secs,
        player_config_from_config,
        resolve_subscription_json,
        CustomCategoryAction,
        SourceConfigAction,
        user_preferences_from_config,
        validate_subscription_json,
        PlayerConfig,
        PlayerConfigPatch,
        UserPreferences,
        UserPreferencesPatch,
    };
    use serde_json::json;



    #[test]
    fn validate_subscription_json_accepts_valid_json() {
        let result = validate_subscription_json(r#"{ "ok": true }"#);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_subscription_json_rejects_invalid_json() {
        let result = validate_subscription_json("{");
        assert!(result.is_err());
    }

    #[test]
    fn format_rfc3339_utc_epoch() {
        let formatted = format_rfc3339_utc_from_secs(0, 0);
        assert_eq!(formatted, "1970-01-01T00:00:00.000Z");
    }

    #[tokio::test]
    async fn resolve_subscription_json_prefers_raw_json() {
        let result =
            resolve_subscription_json(Some("http://example.com"), Some("{\"ok\":true}"))
                .await
                .unwrap();
        assert_eq!(result, "{\"ok\":true}");
    }

    #[test]
    fn player_config_from_config_prefers_saved_config() {
        let config = json!({
            "PlayerConfig": {
                "block_ad_enabled": false,
                "optimization_enabled": false
            }
        });
        let prefs = player_config_from_config(&config);
        assert!(!prefs.block_ad_enabled);
        assert!(!prefs.optimization_enabled);
    }

    #[test]
    fn apply_player_config_patch_updates_only_fields() {
        let base = PlayerConfig {
            block_ad_enabled: true,
            optimization_enabled: true,
        };
        let patch = PlayerConfigPatch {
            block_ad_enabled: Some(false),
            optimization_enabled: None,
        };
        let updated = apply_player_config_patch(base, patch);
        assert!(!updated.block_ad_enabled);
        assert!(updated.optimization_enabled);
    }

    #[test]
    fn user_preferences_from_config_prefers_user_preferences() {
        let config = json!({
            "UserPreferences": {
                "site_name": "TestSite",
                "enable_optimization": false,
                "fluid_search": false
            }
        });
        let prefs = user_preferences_from_config(&config);
        assert_eq!(prefs.site_name, "TestSite");
        assert!(!prefs.enable_optimization);
        assert!(!prefs.fluid_search);
    }

    #[test]
    fn user_preferences_from_config_falls_back_to_site_config() {
        let config = json!({
            "SiteConfig": {
                "SiteName": "LegacySite",
                "Announcement": "Hello",
                "SearchDownstreamMaxPage": 9,
                "SiteInterfaceCacheTime": 3600,
                "DisableYellowFilter": true,
                "DoubanProxyType": "direct",
                "DoubanProxy": "https://proxy.example.com",
                "DoubanImageProxyType": "direct",
                "DoubanImageProxy": "https://img.example.com",
                "FluidSearch": false
            }
        });
        let prefs = user_preferences_from_config(&config);
        assert_eq!(prefs.site_name, "LegacySite");
        assert_eq!(prefs.announcement, "Hello");
        assert_eq!(prefs.search_downstream_max_page, 9);
        assert_eq!(prefs.site_interface_cache_time, 3600);
        assert!(prefs.disable_yellow_filter);
        assert_eq!(prefs.douban_data_source, "direct");
        assert_eq!(prefs.douban_proxy_url, "https://proxy.example.com");
        assert_eq!(prefs.douban_image_proxy_type, "direct");
        assert_eq!(prefs.douban_image_proxy_url, "https://img.example.com");
        assert!(!prefs.fluid_search);
    }

    #[test]
    fn apply_user_preferences_patch_updates_only_fields() {
        let base = UserPreferences::default();
        let patch = UserPreferencesPatch {
            enable_optimization: Some(false),
            player_buffer_mode: Some("max".to_string()),
            ..Default::default()
        };
        let updated = apply_user_preferences_patch(base.clone(), patch);
        assert!(!updated.enable_optimization);
        assert_eq!(updated.player_buffer_mode, "max");
        assert_eq!(updated.site_name, base.site_name);
    }

    #[test]
    fn source_action_toggle_updates_disabled() {
        let config = json!({
            "SourceConfig": [
                { "key": "s1", "name": "A", "api": "http://a", "disabled": false }
            ]
        });

        let updated =
            apply_source_config_action(&config, SourceConfigAction::Toggle { key: "s1".to_string() })
                .unwrap();
        let sources = updated.get("SourceConfig").unwrap().as_array().unwrap();
        let disabled = sources[0].get("disabled").and_then(|v| v.as_bool()).unwrap();
        assert!(disabled);
    }

    #[test]
    fn source_action_reorder_swaps_items() {
        let config = json!({
            "SourceConfig": [
                { "key": "s1", "name": "A", "api": "http://a" },
                { "key": "s2", "name": "B", "api": "http://b" }
            ]
        });

        let updated = apply_source_config_action(
            &config,
            SourceConfigAction::Reorder {
                active_key: "s1".to_string(),
                over_key: "s2".to_string(),
            },
        )
        .unwrap();
        let sources = updated.get("SourceConfig").unwrap().as_array().unwrap();
        let first_key = sources[0].get("key").and_then(|v| v.as_str()).unwrap();
        assert_eq!(first_key, "s2");
    }

    #[test]
    fn source_action_add_validates_required_fields() {
        let config = json!({ "SourceConfig": [] });
        let err = apply_source_config_action(
            &config,
            SourceConfigAction::Add { source: json!({ "name": "A", "api": "http://a" }) },
        )
        .unwrap_err();
        assert_eq!(err, "请填写完整信息");
    }

    #[test]
    fn source_action_add_rejects_duplicate_key() {
        let config = json!({
            "SourceConfig": [
                { "key": "s1", "name": "A", "api": "http://a" }
            ]
        });
        let err = apply_source_config_action(
            &config,
            SourceConfigAction::Add {
                source: json!({ "key": "s1", "name": "B", "api": "http://b" }),
            },
        )
        .unwrap_err();
        assert_eq!(err, "源标识已存在");
    }

    #[test]
    fn category_action_add_sets_defaults() {
        let config = json!({ "CustomCategories": [] });
        let updated = apply_custom_category_action(
            &config,
            CustomCategoryAction::Add {
                category: json!({ "name": "Marvel", "query": "漫威", "type": "movie" }),
            },
        )
        .unwrap();
        let categories = updated.get("CustomCategories").unwrap().as_array().unwrap();
        let first = categories[0].as_object().unwrap();
        assert_eq!(first.get("from").unwrap(), "custom");
        assert_eq!(first.get("disabled").unwrap(), false);
    }

    #[test]
    fn category_action_toggle_flips_disabled() {
        let config = json!({
            "CustomCategories": [
                { "name": "Marvel", "query": "漫威", "type": "movie", "disabled": false }
            ]
        });
        let updated = apply_custom_category_action(
            &config,
            CustomCategoryAction::Toggle { index: 0 },
        )
        .unwrap();
        let categories = updated.get("CustomCategories").unwrap().as_array().unwrap();
        let disabled = categories[0].get("disabled").and_then(|v| v.as_bool()).unwrap();
        assert!(disabled);
    }
}
