use crate::storage::StorageManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSubscribtion {
    pub url: String,
    pub auto_update: bool,
    pub last_check: String,
}
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
    pub config_subscribtion: ConfigSubscribtion,
    pub config_file: String,
    pub site_config: SiteConfig,
    pub user_config: UserConfig,
    pub source_config: Vec<SourceConfig>,
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
        site_config: SiteConfig {
            site_name: "QuantumTV".to_string(),
            announcement: "本网站仅提供影视信息搜索服务，所有内容均来自第三方网站。本站不存储任何视频资源，不对任何内容的准确性、合法性、完整性负责。".to_string(),
            search_downstream_max_page: 5,
            site_interface_cache_time: 7200,
            douban_proxy_type: "cmliussss-cdn-tencent".to_string(),
            douban_proxy: "".to_string(),
            douban_image_proxy_type: "cmliussss-cdn-tencent".to_string(),
            douban_image_proxy: "".to_string(),
            disable_yellow_filter: false,
            fluid_search: true,
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

#[tauri::command]
pub async fn save_config(config: Value, state: State<'_, StorageManager>) -> Result<(), String> {
    state.update_config(config)
}

#[tauri::command]
pub async fn reset_config(state: State<'_, StorageManager>) -> Result<(), String> {
    state.reset_config()
}

#[tauri::command]
pub async fn get_fluid_search(state: State<'_, StorageManager>) -> Result<bool, String> {
    let data = state.get_data()?;

    // Try to get FluidSearch from config
    if let Some(site_config) = data.config.get("SiteConfig") {
        if let Some(fluid_search) = site_config.get("FluidSearch") {
            if let Some(value) = fluid_search.as_bool() {
                return Ok(value);
            }
        }
    }

    // Default to true
    Ok(true)
}

#[tauri::command]
pub async fn set_fluid_search(
    enabled: bool,
    state: State<'_, StorageManager>,
) -> Result<(), String> {
    let mut data = state.get_data()?;

    // Ensure config structure exists
    if !data.config.is_object() {
        data.config = serde_json::json!({});
    }

    let config_obj = data.config.as_object_mut().unwrap();

    // Ensure SiteConfig exists
    if !config_obj.contains_key("SiteConfig") {
        config_obj.insert("SiteConfig".to_string(), serde_json::json!({}));
    }

    let site_config = config_obj.get_mut("SiteConfig").unwrap();
    if let Some(site_config_obj) = site_config.as_object_mut() {
        site_config_obj.insert("FluidSearch".to_string(), serde_json::json!(enabled));
    }

    state.update_config(data.config)
}

/// 获取播放器配置（去广告、优选等）
#[tauri::command]
pub async fn get_player_config(state: State<'_, StorageManager>) -> Result<PlayerConfig, String> {
    let data = state.get_data()?;

    // 尝试从配置中获取播放器配置
    if let Some(player_config) = data.config.get("PlayerConfig") {
        if let Ok(config) = serde_json::from_value::<PlayerConfig>(player_config.clone()) {
            return Ok(config);
        }
    }

    // 返回默认配置
    Ok(PlayerConfig::default())
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
