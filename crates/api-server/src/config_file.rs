use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    pub config: Config,
    pub play_records: HashMap<String, serde_json::Value>,
    pub favorites: HashMap<String, serde_json::Value>,
    pub search_history: Vec<serde_json::Value>,
    pub skip_configs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "ConfigFile")]
    pub config_file: String,
    #[serde(rename = "ConfigSubscribtion")]
    pub config_subscribtion: ConfigSubscribtion,
    #[serde(rename = "CustomCategories")]
    pub custom_categories: Vec<serde_json::Value>,
    #[serde(rename = "SourceConfig")]
    pub source_config: Vec<SourceConfig>,
    #[serde(rename = "UserConfig")]
    pub user_config: UserConfig,
    #[serde(rename = "UserPreferences")]
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSubscribtion {
    #[serde(rename = "AutoUpdate")]
    pub auto_update: bool,
    #[serde(rename = "LastCheck")]
    pub last_check: String,
    #[serde(rename = "URL")]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    pub api: String,
    pub detail: String,
    pub disabled: bool,
    pub from: String,
    pub is_adult: bool,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(rename = "Users")]
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub banned: bool,
    pub role: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    pub announcement: String,
    pub disable_yellow_filter: bool,
    pub douban_data_source: String,
    pub douban_image_proxy_type: String,
    pub douban_image_proxy_url: String,
    pub douban_proxy_url: String,
    pub enable_optimization: bool,
    pub fluid_search: bool,
    pub has_seen_announcement: String,
    pub player_buffer_mode: String,
    pub search_downstream_max_page: u32,
    pub site_interface_cache_time: u32,
    pub site_name: String,
}

pub static PARSES_FILE: LazyLock<String> =
    LazyLock::new(|| std::env::var("PARSES_FILE").unwrap_or_else(|_| "data.json".to_string()));

/// 读取文件
pub async fn load_parses_from_file() -> Result<Root, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(&*PARSES_FILE);
    if !file_path.exists() {
        return Err("文件不存在".into());
    }
    let content = fs::read_to_string(file_path).await?;
    let parses: Root = serde_json::from_str(&content)?;
    Ok(parses)
}

/// 获取 source_config
pub async fn load_source_configs_from_file() -> Result<Vec<SourceConfig>, Box<dyn std::error::Error>>
{
    let parses = load_parses_from_file().await?;
    Ok(parses.config.source_config)
}

/// 过滤 成人
pub async fn filter_adult_source_configs() -> Result<Vec<SourceConfig>, Box<dyn std::error::Error>>
{
    let source_configs = load_source_configs_from_file().await?;
    let filtered_source_configs = source_configs
        .into_iter()
        .filter(|source_config| !source_config.is_adult)
        .collect();
    Ok(filtered_source_configs)
}
