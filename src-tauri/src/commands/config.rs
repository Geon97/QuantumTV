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
