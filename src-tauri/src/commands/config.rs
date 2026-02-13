use crate::storage::StorageManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use tauri::State;
use quantumtv_core::adult;
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

#[tauri::command]
pub async fn save_config(config: Value, state: State<'_, StorageManager>) -> Result<(), String> {
    state.update_config(config)
}

#[tauri::command]
pub async fn reset_config(state: State<'_, StorageManager>) -> Result<(), String> {
    state.reset_config()
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

/// 获取用户偏好配置（统一配置，自动从 SiteConfig 迁移）
#[tauri::command]
pub async fn get_user_preferences(state: State<'_, StorageManager>) -> Result<UserPreferences, String> {
    let data = state.get_data()?;

    // 尝试从配置中获取用户偏好
    if let Some(user_prefs) = data.config.get("UserPreferences") {
        if let Ok(prefs) = serde_json::from_value::<UserPreferences>(user_prefs.clone()) {
            return Ok(prefs);
        }
    }

    // 如果 UserPreferences 不存在或解析失败，
    let mut prefs = UserPreferences::default();

    if let Some(site_config) = data.config.get("SiteConfig") {
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

    Ok(prefs)
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

// 是否为成人源 批量判断
#[tauri::command]
pub async fn is_adult_source(names: Vec<String>) -> Vec<bool> {
    names.iter().map(|n| adult::is_adult_source(n)).collect()
}