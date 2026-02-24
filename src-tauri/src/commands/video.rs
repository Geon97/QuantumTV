use crate::storage::StorageManager;
use image::{GenericImageView, ImageOutputFormat};
use moka::future::Cache;
use regex::Regex;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, RANGE, REFERER, USER_AGENT,
};
use rusqlite::params;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, OnceLock};
use tauri::{Emitter, Manager, State};
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use quantumtv_core::types::SearchResult;
use quantumtv_core::{prefer_best_source, test_video_source, SourceTestResult};

pub struct VideoCacheManager {
    pub cache: Cache<String, Vec<u8>>,
}

impl VideoCacheManager {
    pub fn new() -> Self {
        // 优化缓存配置：
        // - 最大 500 条目（从300增加到500，支持更多预加载）
        // - TTL 15分钟（从10分钟增加到15分钟，减少重复下载）
        // 假设每个 ts 片段约 1-2MB，500个约 500MB-1GB
        let cache = Cache::builder()
            .max_capacity(500)
            .time_to_live(std::time::Duration::from_secs(900))
            .build();
        Self { cache }
    }

    pub async fn get(&self, url: &str) -> Option<Vec<u8>> {
        self.cache.get(url).await
    }

    pub async fn set(&self, url: String, data: Vec<u8>) {
        self.cache.insert(url, data).await;
    }
}

pub struct SearchCacheManager {
    pub cache: Cache<String, Vec<SearchResult>>,
}

impl SearchCacheManager {
    pub fn new() -> Self {
        // 最大 1000 条搜索结果缓存，TTL 3600 秒（1 小时）
        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(3600))
            .build();
        Self { cache }
    }

    pub async fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let key = Self::normalize_key(query);
        self.cache.get(&key).await
    }

    pub async fn set(&self, query: String, results: Vec<SearchResult>) {
        let key = Self::normalize_key(&query);
        self.cache.insert(key, results).await;
    }

    fn normalize_key(query: &str) -> String {
        query.trim().to_lowercase()
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetVideoDetailOptimizedResponse {
    pub detail: SearchResult,
    pub other_sources: Vec<SearchResult>,
}

/// 播放器初始化状态响应
/// 包含播放器启动所需的所有数据，减少 IPC 通信次数
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInitialState {
    /// 视频详情
    pub detail: SearchResult,
    /// 其他可用源
    pub other_sources: Vec<SearchResult>,
    /// 播放记录（集数索引和播放时间）
    pub play_record: Option<PlayRecordInfo>,
    /// 是否已收藏
    pub is_favorited: bool,
    /// 跳过配置
    pub skip_config: Option<SkipConfigInfo>,
    /// 去广告开关
    pub block_ad_enabled: bool,
    /// 优选开关
    pub optimization_enabled: bool,
}

/// 播放记录信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayRecordInfo {
    pub episode_index: i32,
    pub play_time: i32,
}

/// 跳过配置信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkipConfigInfo {
    pub enable: bool,
    pub intro_time: i32,
    pub outro_time: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchStreamEvent {
    pub results: Vec<SearchResult>,
    pub source: String,
    pub source_name: String,
    pub total_sources: i32,
    pub completed_sources: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSite {
    pub key: String,
    pub api: String,
    pub name: String,
    pub detail: Option<String>,
    pub is_adult: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSearchItem {
    pub vod_id: Value, // Can be int or string
    pub vod_name: String,
    pub vod_pic: String,
    pub vod_remarks: Option<String>,
    pub vod_play_url: Option<String>,
    pub vod_class: Option<String>,
    pub vod_year: Option<String>,
    pub vod_content: Option<String>,
    pub vod_douban_id: Option<Value>,
    pub type_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSearchResponse {
    pub list: Vec<ApiSearchItem>,
    pub pagecount: Option<i32>,
}

// Douban Related Structs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanCelebrity {
    pub id: String,
    pub name: String,
    pub alt: Option<String>,
    pub avatars: Option<DoubanAvatars>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanAvatars {
    pub small: String,
    pub medium: String,
    pub large: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanRating {
    pub max: f32,
    pub average: f32,
    pub stars: String,
    pub min: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanMovieDetail {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub alt: Option<String>,
    pub rating: Option<DoubanRating>,
    pub ratings_count: Option<i32>,
    pub images: Option<DoubanAvatars>,
    pub subtype: Option<String>,
    pub directors: Option<Vec<DoubanCelebrity>>,
    pub casts: Option<Vec<DoubanCelebrity>>,
    pub writers: Option<Vec<DoubanCelebrity>>,
    pub pubdates: Option<Vec<String>>,
    pub year: Option<String>,
    pub genres: Option<Vec<String>>,
    pub countries: Option<Vec<String>>,
    pub mainland_pubdate: Option<String>,
    pub aka: Option<Vec<String>>,
    pub summary: Option<String>,
    pub durations: Option<Vec<String>>,
    pub seasons_count: Option<i32>,
    pub episodes_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanAuthor {
    pub id: String,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub alt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanComment {
    pub id: String,
    pub created_at: String,
    pub content: String,
    pub useful_count: i32,
    pub rating: Option<DoubanRatingShort>,
    pub author: DoubanAuthor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanRatingShort {
    pub max: i32,
    pub value: f32,
    pub min: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoubanCommentsResponse {
    pub start: i32,
    pub count: i32,
    pub total: i32,
    pub comments: Vec<DoubanComment>,
}
static VIDEO_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_video_client() -> &'static reqwest::Client {
    VIDEO_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            // 大幅增加连接池大小，允许更高并发
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(std::time::Duration::from_secs(120))
            // 开启 TCP_NODELAY，减少小包延迟
            .tcp_nodelay(true)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            // 开启自适应窗口，解决跨国高延迟下的吞吐量瓶颈
            .http2_adaptive_window(true)
            // 保持 H2 连接活跃，防止中间设备切断
            .http2_keep_alive_interval(std::time::Duration::from_secs(30))
            // 增加超时时间，适应跨国慢速网络
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            // 烂证书 野鸡CDN 连接问题
            .danger_accept_invalid_certs(true) // 忽略证书无效/过期/自签名
            .danger_accept_invalid_hostnames(true) // 忽略域名不匹配
            .no_proxy() // (可选) 避免被系统代理设置干扰，直连
            .build()
            .expect("Failed to create global video client")
    })
}
fn is_playable_m3u8(url: &str) -> bool {
    url.to_lowercase().contains(".m3u8")
}

fn clean_html_tags(html: &str) -> String {
    // Basic cleaning, more advanced can be added if needed
    html.replace("<p>", "")
        .replace("</p>", "")
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<div>", "")
        .replace("</div>", "")
}

const YELLOW_WORDS: &[&str] = &[
    "伦理片",
    "福利",
    "里番动漫",
    "门事件",
    "萝莉少女",
    "制服诱惑",
    "国产传媒",
    "cosplay",
    "黑丝诱惑",
    "无码",
    "日本无码",
    "有码",
    "日本有码",
    "SWAG",
    "网红主播",
    "色情片",
    "同性片",
    "福利视频",
    "福利片",
    "写真热舞",
    "倫理片",
    "理论片",
    "韩国伦理",
    "港台三级",
    "电影解说",
    "伦理",
    "日本伦理",
];

fn parse_episodes(play_url: &str) -> (Vec<String>, Vec<String>) {
    let mut episodes = Vec::new();
    let mut titles = Vec::new();

    let groups = play_url.split("$$$");
    for group in groups {
        let mut group_episodes = Vec::new();
        let mut group_titles = Vec::new();
        let items = group.split('#');
        for item in items {
            let parts: Vec<&str> = item.split('$').collect();
            if parts.len() == 2 && is_playable_m3u8(parts[1]) {
                group_titles.push(parts[0].to_string());
                group_episodes.push(parts[1].to_string());
            } else if parts.len() == 1 && is_playable_m3u8(parts[0]) {
                group_titles.push((group_episodes.len() + 1).to_string());
                group_episodes.push(parts[0].to_string());
            }
        }
        if group_episodes.len() > episodes.len() {
            episodes = group_episodes;
            titles = group_titles;
        }
    }
    (episodes, titles)
}

#[tauri::command]
pub async fn search(
    query: String,
    app_handle: tauri::AppHandle,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
) -> Result<Vec<SearchResult>, String> {
    // 首先尝试从缓存获取结果
    if let Some(cached_results) = cache.get(&query).await {
        return Ok(cached_results);
    }

    let data = storage.get_data()?;
    let config = &data.config;

    // 读取 FluidSearch 配置，判断是否启用流式搜索（从 UserPreferences 读取）
    let fluid_search = config
        .get("UserPreferences")
        .and_then(|v| v.get("fluid_search"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // 仅在启用 FluidSearch 时才使用流式输出
    let use_streaming = fluid_search;

    let mut sites = if let Some(source_config) = config.get("SourceConfig").and_then(|v| v.as_array()) {
        source_config
            .iter()
            .filter_map(|s| {
                if s.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false) {
                    return None;
                }
                Some(ApiSite {
                    key: s.get("key")?.as_str()?.to_string(),
                    api: s.get("api")?.as_str()?.to_string(),
                    name: s.get("name")?.as_str()?.to_string(),
                    detail: s
                        .get("detail")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    is_adult: s.get("is_adult").and_then(|v| v.as_bool()),
                })
            })
            .collect::<Vec<ApiSite>>()
    } else {
        vec![]
    };

    if sites.is_empty() {
        return Ok(vec![]);
    }

    // 读取过滤配置
    let disable_yellow_filter = config
        .get("UserPreferences")
        .and_then(|v| v.get("disable_yellow_filter"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // 如果启用过滤（disable_yellow_filter=false），在搜索前就过滤掉18+的源
    if !disable_yellow_filter {
        sites.retain(|site| !site.is_adult.unwrap_or(false));
    }

    // 过滤后如果没有源了，直接返回
    if sites.is_empty() {
        return Ok(vec![]);
    }

    let total_sources = sites.len() as i32;

    // 限制并发数：最多同时请求 20 个源，充分利用并发
    let semaphore = Arc::new(Semaphore::new(20));
    let client = get_video_client();
    let completed = Arc::new(tokio::sync::Mutex::new(0i32));

    let mut handles = Vec::new();
    for site in &sites {
        let semaphore = semaphore.clone();
        let client = client.clone();
        let query = query.clone();
        let site_clone = site.clone();
        // 仅在启用流式搜索时才传递 app_handle
        let app_handle_opt = if use_streaming {
            Some(app_handle.clone())
        } else {
            None
        };
        let completed = completed.clone();
        // 克隆过滤配置到闭包中
        let disable_filter = disable_yellow_filter;

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.ok()?;
            let url = format!(
                "{}?ac=videolist&wd={}",
                site_clone.api,
                urlencoding::encode(&query)
            );

            // 单个源请求超时 6 秒
            let resp = match timeout(Duration::from_secs(6), client.get(&url).send()).await {
                Ok(Ok(res)) if res.status().is_success() => res,
                _ => {
                    // 如果启用了流式搜索，即使失败也要发送事件
                    if let Some(app_handle) = &app_handle_opt {
                        // 尝试获取窗口 - 兼容桌面端和移动端
                        let window = app_handle.get_webview_window("main")
                            .or_else(|| app_handle.webview_windows().values().next().cloned());

                        if let Some(window) = window {
                            let mut count = completed.lock().await;
                            *count += 1;
                            let _ = window.emit(
                                "search-stream-result",
                                SearchStreamEvent {
                                    results: vec![],
                                    source: site_clone.key.clone(),
                                    source_name: site_clone.name.clone(),
                                    total_sources,
                                    completed_sources: *count,
                                },
                            );
                        }
                    }
                    return Some(vec![]);
                }
            };

            let body = match timeout(Duration::from_secs(5), resp.text()).await {
                Ok(Ok(text)) => text,
                _ => {
                    if let Some(app_handle) = &app_handle_opt {
                        // 尝试获取窗口 - 兼容桌面端和移动端
                        let window = app_handle.get_webview_window("main")
                            .or_else(|| app_handle.webview_windows().values().next().cloned());

                        if let Some(window) = window {
                            let mut count = completed.lock().await;
                            *count += 1;
                            let _ = window.emit(
                                "search-stream-result",
                                SearchStreamEvent {
                                    results: vec![],
                                    source: site_clone.key.clone(),
                                    source_name: site_clone.name.clone(),
                                    total_sources,
                                    completed_sources: *count,
                                },
                            );
                        }
                    }
                    return Some(vec![]);
                }
            };

            let mut source_results = Vec::new();
            if let Ok(search_res) = serde_json::from_str::<ApiSearchResponse>(&body) {
                source_results = search_res
                    .list
                    .into_iter()
                    .map(|item| {
                        let (episodes, episodes_titles) =
                            parse_episodes(item.vod_play_url.as_deref().unwrap_or(""));
                        SearchResult {
                            id: match item.vod_id {
                                Value::String(s) => s,
                                Value::Number(n) => n.to_string(),
                                _ => "".to_string(),
                            },
                            title: item.vod_name.trim().to_string(),
                            poster: item.vod_pic,
                            episodes,
                            episodes_titles,
                            source: site_clone.key.clone(),
                            source_name: site_clone.name.clone(),
                            class: item.vod_class,
                            year: item.vod_year,
                            desc: item.vod_content.map(|c| clean_html_tags(&c)),
                            type_name: item.type_name,
                            douban_id: item
                                .vod_douban_id
                                .and_then(|v| v.as_i64())
                                .map(|v| v as i32),
                        }
                    })
                    .collect::<Vec<SearchResult>>();
            }

            // 在流式输出前进行内容关键词过滤（源已经在搜索前过滤了）
            if !disable_filter {
                source_results.retain(|res| {
                    let type_name = res.type_name.as_deref().unwrap_or("");
                    // 只需要检查关键词，因为18+源已经在搜索前被过滤掉了
                    !YELLOW_WORDS.iter().any(|w| type_name.contains(w))
                });
            }

            // 如果启用了流式搜索，立即发送该源的搜索结果给前端
            if let Some(app_handle) = &app_handle_opt {
                // 尝试获取窗口 - 兼容桌面端和移动端
                let window = app_handle.get_webview_window("main")
                    .or_else(|| app_handle.webview_windows().values().next().cloned());

                if let Some(window) = window {
                    let mut count = completed.lock().await;
                    *count += 1;
                    let _ = window.emit(
                        "search-stream-result",
                        SearchStreamEvent {
                            results: source_results.clone(),
                            source: site_clone.key.clone(),
                            source_name: site_clone.name.clone(),
                            total_sources,
                            completed_sources: *count,
                        },
                    );
                }
            }

            Some(source_results)
        });
        handles.push(handle);
    }

    let mut all_results = Vec::new();
    for handle in handles {
        if let Ok(Some(results)) = handle.await {
            all_results.extend(results);
        }
    }

    // Filter duplicates
    let mut unique_results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for res in all_results {
        let key = format!("{}|{}", res.source, res.id);
        if seen.insert(key) {
            // 按关键词筛选成人内容
            if !disable_yellow_filter {
                let type_name = res.type_name.as_deref().unwrap_or("");
                if YELLOW_WORDS.iter().any(|w| type_name.contains(w)) {
                    continue;
                }
            }
            if !res.episodes.is_empty() {
                unique_results.push(res);
            }
        }
    }

    // Basic ranking
    unique_results.sort_by(|a, b| {
        let a_match = a.title.contains(&query);
        let b_match = b.title.contains(&query);
        if a_match && !b_match {
            std::cmp::Ordering::Less
        } else if !a_match && b_match {
            std::cmp::Ordering::Greater
        } else {
            a.title.len().cmp(&b.title.len())
        }
    });

    // 如果启用了流式搜索，发送搜索完成事件
    if use_streaming {
        // 尝试获取窗口 - 兼容桌面端和移动端
        let window = app_handle.get_webview_window("main")
            .or_else(|| app_handle.webview_windows().values().next().cloned());

        if let Some(window) = window {
            let _ = window.emit(
                "search-stream-completed",
                serde_json::json!({
                    "total": unique_results.len(),
                    "query": query
                }),
            );
        }
    }

    // 缓存搜索结果
    cache.set(query, unique_results.clone()).await;

    Ok(unique_results)
}

#[tauri::command]
pub async fn get_video_detail(
    source: String,
    id: String,
    storage: State<'_, StorageManager>,
) -> Result<SearchResult, String> {
    let data = storage.get_data()?;
    let config = &data.config;

    let api_site =
        if let Some(source_config) = config.get("SourceConfig").and_then(|v| v.as_array()) {
            source_config
                .iter()
                .find(|s| s.get("key").and_then(|v| v.as_str()) == Some(&source))
                .and_then(|s| {
                    Some(ApiSite {
                        key: s.get("key")?.as_str()?.to_string(),
                        api: s.get("api")?.as_str()?.to_string(),
                        name: s.get("name")?.as_str()?.to_string(),
                        detail: s
                            .get("detail")
                            .and_then(|v| v.as_str())
                            .map(|v| v.to_string()),
                        is_adult: s.get("is_adult").and_then(|v| v.as_bool()),
                    })
                })
        } else {
            None
        };

    let site = api_site.ok_or_else(|| "Source not found".to_string())?;
    let client = get_video_client();

    let url = format!("{}?ac=videolist&ids={}", site.api, id);

    // 添加超时控制：8秒
    let resp = match timeout(Duration::from_secs(8), client.get(&url).send()).await {
        Ok(Ok(res)) => res,
        _ => return Err("Failed to fetch detail: request timeout or network error".to_string()),
    };

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch detail: {}", resp.status()));
    }

    let body = match timeout(Duration::from_secs(5), resp.text()).await {
        Ok(Ok(text)) => text,
        _ => return Err("Failed to read response: timeout".to_string()),
    };

    let search_res = serde_json::from_str::<ApiSearchResponse>(&body)
        .map_err(|e| format!("Parse error: {}, body: {}", e, body))?;

    let item = search_res
        .list
        .into_iter()
        .next()
        .ok_or_else(|| "Video not found".to_string())?;

    let (episodes, episodes_titles) = parse_episodes(item.vod_play_url.as_deref().unwrap_or(""));

    Ok(SearchResult {
        id: match item.vod_id {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            _ => "".to_string(),
        },
        title: item.vod_name.trim().to_string(),
        poster: item.vod_pic,
        episodes,
        episodes_titles,
        source: site.key,
        source_name: site.name,
        class: item.vod_class,
        year: item.vod_year,
        desc: item.vod_content.map(|c| clean_html_tags(&c)),
        type_name: item.type_name,
        douban_id: item
            .vod_douban_id
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
    })
}

#[tauri::command]
pub async fn get_video_detail_optimized(
    source: String,
    id: String,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
    also_search_similar: Option<bool>,
) -> Result<GetVideoDetailOptimizedResponse, String> {
    let data = storage.get_data()?;
    let config = &data.config;

    // 立即获取指定源的详情
    let api_site =
        if let Some(source_config) = config.get("SourceConfig").and_then(|v| v.as_array()) {
            source_config
                .iter()
                .find(|s| s.get("key").and_then(|v| v.as_str()) == Some(&source))
                .and_then(|s| {
                    Some(ApiSite {
                        key: s.get("key")?.as_str()?.to_string(),
                        api: s.get("api")?.as_str()?.to_string(),
                        name: s.get("name")?.as_str()?.to_string(),
                        detail: s
                            .get("detail")
                            .and_then(|v| v.as_str())
                            .map(|v| v.to_string()),
                        is_adult: s.get("is_adult").and_then(|v| v.as_bool()),
                    })
                })
        } else {
            None
        };

    let site = api_site.ok_or_else(|| "Source not found".to_string())?;
    let client = get_video_client();
    let url = format!("{}?ac=videolist&ids={}", site.api, id);

    let resp = match timeout(Duration::from_secs(8), client.get(&url).send()).await {
        Ok(Ok(res)) => res,
        _ => return Err("Failed to fetch detail: timeout".to_string()),
    };

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch detail: {}", resp.status()));
    }

    let body = match timeout(Duration::from_secs(5), resp.text()).await {
        Ok(Ok(text)) => text,
        _ => return Err("Failed to read response: timeout".to_string()),
    };

    let search_res = serde_json::from_str::<ApiSearchResponse>(&body).map_err(|e| e.to_string())?;

    let item = search_res
        .list
        .into_iter()
        .next()
        .ok_or_else(|| "Video not found".to_string())?;

    let (episodes, episodes_titles) = parse_episodes(item.vod_play_url.as_deref().unwrap_or(""));

    let detail = SearchResult {
        id: match item.vod_id {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            _ => "".to_string(),
        },
        title: item.vod_name.trim().to_string(),
        poster: item.vod_pic.clone(),
        episodes,
        episodes_titles,
        source: site.key,
        source_name: site.name,
        class: item.vod_class.clone(),
        year: item.vod_year.clone(),
        desc: item.vod_content.as_ref().map(|c| clean_html_tags(c)),
        type_name: item.type_name.clone(),
        douban_id: item
            .vod_douban_id
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
    };

    // 如果需要搜索相似源，尝试从缓存快速获取
    let other_sources = if also_search_similar.unwrap_or(false) {
        // 尝试从缓存获取搜索结果
        if let Some(cached_results) = cache.get(&detail.title).await {
            // 过滤掉当前源，返回其他源
            cached_results
                .into_iter()
                .filter(|r| !(r.source == source && r.id == id))
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Ok(GetVideoDetailOptimizedResponse {
        detail,
        other_sources,
    })
}

#[tauri::command]
pub async fn proxy_image(
    url: String,
    cache_manager: State<'_, crate::db::image_cache::ImageCacheManager>,
) -> Result<Vec<u8>, String> {
    // 1. 先尝试从 SQLite 缓存获取
    match cache_manager.get(&url) {
        Ok(Some(data)) => {
            return Ok(data);
        }
        Ok(None) => {
            // 缓存未命中，继续请求
        }
        Err(e) => {
            eprintln!("Failed to get cached image: {}", e);
            // 缓存读取失败，继续请求
        }
    }

    // 2. 使用全局 Client 获取图片
    let client = get_video_client();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    if url.contains("doubanio.com") {
        headers.insert(REFERER, HeaderValue::from_static("https://www.douban.com/"));
    }

    let resp = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch image: {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // 3. 压缩图片
    let compressed_bytes = tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).map_err(|e| format!("图片解码失败: {}", e))?;
        let (width, height) = img.dimensions();
        let processed_img = if width > 800 {
            img.resize(
                800,
                800 * height / width,
                image::imageops::FilterType::Triangle,
            )
        } else {
            img
        };

        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        processed_img
            .write_to(&mut cursor, ImageOutputFormat::Jpeg(70))
            .map_err(|e| format!("图片编码失败: {}", e))?;

        Ok::<Vec<u8>, String>(buf)
    })
    .await
    .map_err(|e| e.to_string())??;

    // 4. 保存到 SQLite 缓存
    if let Err(e) = cache_manager.set(&url, &compressed_bytes) {
        eprintln!("Failed to save image to cache: {}", e);
    }

    Ok(compressed_bytes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub status: u16,
    pub body: String,
}

#[tauri::command]
pub async fn fetch_url(
    url: String,
    method: Option<String>,
    headers_opt: Option<std::collections::HashMap<String, String>>,
) -> Result<FetchResponse, String> {
    // 使用全局 Client
    let client = get_video_client();
    let method_str = method.unwrap_or_else(|| "GET".to_string());
    let req_method = match method_str.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "HEAD" => reqwest::Method::HEAD,
        _ => reqwest::Method::GET,
    };
    let request_builder = client.request(req_method, &url);

    let mut final_headers = HeaderMap::new();

    if let Some(h) = headers_opt {
        for (k, v) in h {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&v) {
                    final_headers.insert(name, value);
                }
            }
        }
    }

    if !final_headers.contains_key(USER_AGENT) {
        final_headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    }

    if url.contains("doubanio.com") || url.contains("douban.com") {
        final_headers.insert(REFERER, HeaderValue::from_static("https://www.douban.com/"));
    }

    let resp = request_builder
        .headers(final_headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    Ok(FetchResponse { status, body })
}
// 带重试的请求 重试2次
async fn fetch_with_retry(
    url: &str,
    method: reqwest::Method,
    headers: HeaderMap,
) -> Result<reqwest::Response, String> {
    let client = get_video_client();
    let mut retries = 2;

    loop {
        let req = client.request(method.clone(), url).headers(headers.clone());
        match req.send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                retries -= 1;
                if retries == 0 {
                    return Err(e.to_string());
                }
                // 指数退避或固定等待
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchBinaryResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[tauri::command]
pub async fn fetch_binary(
    url: String,
    method: Option<String>,
    headers_opt: Option<std::collections::HashMap<String, String>>,
    cache_manager: State<'_, VideoCacheManager>,
) -> Result<FetchBinaryResponse, String> {
    let method_str = method.unwrap_or_else(|| "GET".to_string());
    let is_get = method_str.to_uppercase() == "GET";

    // 1. 尝试从缓存获取
    if is_get {
        if let Some(cached_data) = cache_manager.get(&url).await {
            return Ok(FetchBinaryResponse {
                status: 200,
                body: cached_data,
            });
        }
    }

    // 2. 准备 Headers
    let mut final_headers = HeaderMap::new();
    if let Some(h) = headers_opt.clone() {
        for (k, v) in h {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&v) {
                    final_headers.insert(name, value);
                }
            }
        }
    }
    if !final_headers.contains_key(USER_AGENT) {
        final_headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    }
    if url.contains("doubanio.com") || url.contains("douban.com") {
        final_headers.insert(REFERER, HeaderValue::from_static("https://www.douban.com/"));
    }
    if url.contains(".ts") {
        // 添加 Range 头
        final_headers.insert(RANGE, HeaderValue::from_static("bytes=0-"));
    }
    let req_method = match method_str.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "HEAD" => reqwest::Method::HEAD,
        _ => reqwest::Method::GET,
    };
    // 3. 执行带重试的网络请求
    let resp = fetch_with_retry(&url, req_method, final_headers).await?;
    let status = resp.status().as_u16();
    let body_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let body = body_bytes.to_vec();

    // 4. 只有成功的 GET 请求才存入缓存并触发预取
    if is_get && status == 200 {
        cache_manager.set(url.clone(), body.clone()).await;

        if url.contains(".ts") {
            let cache_clone = cache_manager.cache.clone();
            let headers_clone = headers_opt.clone();

            tokio::spawn(async move {
                prefetch_next_segments(url, headers_clone, cache_clone).await;
            });
        }
    }

    Ok(FetchBinaryResponse { status, body })
}

/// 获取 M3U8 内容并可选地进行去广告处理
///
/// # 参数
/// - `url`: M3U8 文件的 URL
/// - `enable_ad_block`: 是否启用去广告功能，默认为 false
/// - `headers_opt`: 可选的自定义 HTTP 请求头
///
/// # 返回
/// 处理后的 M3U8 文本内容
#[tauri::command]
pub async fn fetch_m3u8(
    url: String,
    enable_ad_block: Option<bool>,
    headers_opt: Option<std::collections::HashMap<String, String>>,
) -> Result<String, String> {
    // 准备 HTTP 请求头
    let mut final_headers = HeaderMap::new();
    if let Some(h) = headers_opt {
        for (k, v) in h {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&v) {
                    final_headers.insert(name, value);
                }
            }
        }
    }

    // 添加默认 User-Agent
    if !final_headers.contains_key(USER_AGENT) {
        final_headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        );
    }

    // 为特定域名添加 Referer
    if url.contains("doubanio.com") || url.contains("douban.com") {
        final_headers.insert(REFERER, HeaderValue::from_static("https://www.douban.com/"));
    }

    // 执行带重试的 HTTP 请求
    let resp = fetch_with_retry(&url, reqwest::Method::GET, final_headers).await?;
    let body_bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    // 解码为 UTF-8 文本
    let content = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| format!("无法将 M3U8 内容解码为 UTF-8: {}", e))?;

    // 如果启用了去广告，则调用 core 中的过滤函数
    let result = if enable_ad_block.unwrap_or(false) {
        quantumtv_core::filter_ads_from_m3_u8(&content)
    } else {
        content
    };

    Ok(result)
}

// 预测并预取后续分片
async fn prefetch_next_segments(
    current_url: String,
    headers: Option<HashMap<String, String>>,
    cache: Cache<String, Vec<u8>>,
) {
    // 简单的数字预测 logic: 查找末尾连续的数字
    // 如 segment_01.ts -> segment_02.ts
    let re = Regex::new(r"(\d+)(\.ts.*)$").unwrap();
    let Some(caps) = re.captures(&current_url) else {
        return;
    };

    let num_str = caps.get(1).unwrap().as_str();
    let suffix = caps.get(2).unwrap().as_str();
    let prefix = &current_url[..caps.get(1).unwrap().start()];

    let Ok(current_num) = num_str.parse::<u64>() else {
        return;
    };
    let padding = num_str.len();

    // 使用全局 Client
    let client = get_video_client();

    // 优化的 HTTP 客户端配置
    let mut final_headers = HeaderMap::new();
    if let Some(h) = headers {
        for (k, v) in h {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&v) {
                    final_headers.insert(name, value);
                }
            }
        }
    }
    if !final_headers.contains_key(USER_AGENT) {
        final_headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    }
    // 直接加上 Range
    final_headers.insert(RANGE, HeaderValue::from_static("bytes=0-"));

    // 预取接下来的 15 个分片
    let mut handles = Vec::new();
    for i in 1..=15 {
        let next_num = current_num + i;
        let next_num_str = format!("{:0width$}", next_num, width = padding);
        let next_url = format!("{}{}{}", prefix, next_num_str, suffix);

        // 已有缓存，跳过
        if cache.contains_key(&next_url) {
            continue;
        }

        // 并发下载（使用并发限制避免过载）
        let client_clone = client.clone();
        let request_headers = final_headers.clone();
        let cache_clone = cache.clone();
        let next_url_clone = next_url.clone();

        let handle = tokio::spawn(async move {
            // 增强的重试逻辑：3次重试
            let mut retries = 3;
            while retries > 0 {
                let resp = client_clone
                    .get(&next_url_clone)
                    .headers(request_headers.clone())
                    .timeout(std::time::Duration::from_secs(10)) // 10秒超时
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.status().is_success() => {
                        if let Ok(data) = r.bytes().await {
                            // moka 会自动处理 LRU 淘汰和 TTL 过期
                            cache_clone.insert(next_url_clone, data.to_vec()).await;
                        }
                        break; // 成功则退出循环
                    }
                    _ => {
                        retries -= 1;
                        if retries > 0 {
                            // 指数退避：200ms, 400ms, 800ms
                            let delay = 200 * (4 - retries);
                            tokio::time::sleep(std::time::Duration::from_millis(delay as u64)).await;
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    // 等待所有预取任务完成
    for handle in handles {
        let _ = handle.await;
    }
}

#[tauri::command]
pub async fn get_douban_data(
    subject_id: String,
    data_type: String, // "full" or "comments"
    start: Option<i32>,
    count: Option<i32>,
) -> Result<Value, String> {
    // 使用全局 Client 请求
    let client = get_video_client();
    let url = if data_type == "comments" {
        format!(
            "https://movie.douban.com/subject/{}/comments?status=P&sort=new_score&start={}&count={}",
            subject_id,
            start.unwrap_or(0),
            count.unwrap_or(20)
        )
    } else {
        format!("https://movie.douban.com/subject/{}/", subject_id)
    };

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://movie.douban.com/"),
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    let resp = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!(
            "Douban request failed with status: {}",
            resp.status()
        ));
    }

    let html_content = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html_content);

    if data_type == "comments" {
        let mut comments = Vec::new();
        let comment_selector = Selector::parse(".comment-item").unwrap();
        let avatar_selector = Selector::parse(".avatar a img").unwrap();
        let user_link_selector = Selector::parse(".comment-info a").unwrap();
        let rating_selector = Selector::parse(".comment-info .rating").unwrap();
        let short_selector = Selector::parse(".short").unwrap();
        let time_selector = Selector::parse(".comment-time").unwrap();
        let vote_selector = Selector::parse(".vote-count").unwrap();

        for element in document.select(&comment_selector) {
            let avatar_url = element
                .select(&avatar_selector)
                .next()
                .and_then(|img| img.value().attr("src"))
                .unwrap_or("")
                .replace("/u/pido/", "/u/")
                .replace("s_ratio", "m_ratio");

            let user_element = element.select(&user_link_selector).next();
            let user_name = user_element
                .map(|a| a.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
            let user_link = user_element
                .and_then(|a| a.value().attr("href"))
                .unwrap_or("");
            let user_id = user_link
                .split('/')
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("")
                .to_string();

            let rating_value = element
                .select(&rating_selector)
                .next()
                .and_then(|span| span.value().attr("class"))
                .and_then(|c| {
                    let re = Regex::new(r"allstar(\d+)").unwrap();
                    re.captures(c)
                        .and_then(|cap| cap.get(1))
                        .map(|m| m.as_str().parse::<f32>().unwrap_or(0.0) / 10.0)
                })
                .unwrap_or(0.0);

            let content = element
                .select(&short_selector)
                .next()
                .map(|s| s.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let time_element = element.select(&time_selector).next();
            let mut time = String::new();
            if let Some(t) = time_element {
                if let Some(title_attr) = t.value().attr("title") {
                    time = title_attr.to_string();
                } else {
                    time = t.text().collect::<String>().trim().to_string();
                }
            }

            let useful_count = element
                .select(&vote_selector)
                .next()
                .map(|v| {
                    v.text()
                        .collect::<String>()
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            let comment_id = element
                .value()
                .attr("data-cid")
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("sc_{}", Uuid::new_v4()));

            if !content.is_empty() {
                comments.push(DoubanComment {
                    id: comment_id,
                    created_at: time,
                    content,
                    useful_count,
                    rating: if rating_value > 0.0 {
                        Some(DoubanRatingShort {
                            max: 5,
                            value: rating_value,
                            min: 0,
                        })
                    } else {
                        None
                    },
                    author: DoubanAuthor {
                        id: user_id,
                        uid: user_name.clone(),
                        name: user_name,
                        avatar: avatar_url,
                        alt: Some(user_link.to_string()),
                    },
                });
            }
        }

        let total_selector = Selector::parse(".mod-hd h2 span").unwrap();
        let total_text = document
            .select(&total_selector)
            .next()
            .map(|s| s.text().collect::<String>())
            .unwrap_or_default();
        let re = Regex::new(r"全部\s*(\d+)\s*条").unwrap();
        let total = re
            .captures(&total_text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().parse::<i32>().unwrap_or(0))
            .unwrap_or(comments.len() as i32);

        let res = DoubanCommentsResponse {
            start: start.unwrap_or(0),
            count: count.unwrap_or(comments.len() as i32),
            total,
            comments,
        };
        Ok(serde_json::to_value(res).unwrap())
    } else {
        // Full subject data
        let title_selector = Selector::parse("span[property='v:itemreviewed']").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| {
                let t_selector = Selector::parse("title").unwrap();
                document
                    .select(&t_selector)
                    .next()
                    .map(|s| {
                        s.text()
                            .collect::<String>()
                            .split(' ')
                            .next()
                            .unwrap_or("")
                            .to_string()
                    })
                    .unwrap_or_default()
            });

        let year_selector = Selector::parse("span.year").unwrap();
        let year = document.select(&year_selector).next().map(|s| {
            s.text()
                .collect::<String>()
                .replace(['(', ')'], "")
                .trim()
                .to_string()
        });

        let rating_selector = Selector::parse("strong.rating_num").unwrap();
        let rating_avg = document
            .select(&rating_selector)
            .next()
            .and_then(|s| s.text().collect::<String>().trim().parse::<f32>().ok())
            .unwrap_or(0.0);

        let votes_selector = Selector::parse("span[property='v:votes']").unwrap();
        let rating_count = document
            .select(&votes_selector)
            .next()
            .and_then(|s| s.text().collect::<String>().trim().parse::<i32>().ok())
            .unwrap_or(0);

        let genre_selector = Selector::parse("span[property='v:genre']").unwrap();
        let genres: Vec<String> = document
            .select(&genre_selector)
            .map(|s| s.text().collect::<String>().trim().to_string())
            .collect();

        let duration_selector = Selector::parse("span[property='v:runtime']").unwrap();
        let durations: Vec<String> = document
            .select(&duration_selector)
            .map(|s| s.text().collect::<String>().trim().to_string())
            .collect();

        let summary_selector = Selector::parse("span[property='v:summary']").unwrap();
        let summary_hidden_selector = Selector::parse("span.all.hidden").unwrap();
        let summary = document
            .select(&summary_hidden_selector)
            .next()
            .or_else(|| document.select(&summary_selector).next())
            .map(|s| {
                s.text()
                    .collect::<String>()
                    .trim()
                    .replace('\n', " ")
                    .to_string()
            });

        let poster_selector = Selector::parse("#mainpic img").unwrap();
        let poster = document
            .select(&poster_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .unwrap_or("")
            .to_string();

        let mut directors = Vec::new();
        let director_selector = Selector::parse("a[rel='v:directedBy']").unwrap();
        for el in document.select(&director_selector) {
            let name = el.text().collect::<String>().trim().to_string();
            let href = el.value().attr("href").unwrap_or("");
            let id = href
                .split('/')
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                directors.push(DoubanCelebrity {
                    id,
                    name,
                    alt: Some(href.to_string()),
                    avatars: None,
                    roles: Some(vec!["导演".to_string()]),
                });
            }
        }

        let mut casts = Vec::new();
        let actor_selector = Selector::parse("a[rel='v:starring']").unwrap();
        for el in document.select(&actor_selector) {
            let name = el.text().collect::<String>().trim().to_string();
            let href = el.value().attr("href").unwrap_or("");
            let id = href
                .split('/')
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                casts.push(DoubanCelebrity {
                    id,
                    name,
                    alt: Some(href.to_string()),
                    avatars: None,
                    roles: None,
                });
            }
        }

        let detail = DoubanMovieDetail {
            id: subject_id,
            title,
            original_title: None, // Simplified
            alt: Some(url),
            rating: if rating_avg > 0.0 {
                Some(DoubanRating {
                    max: 10.0,
                    average: rating_avg,
                    stars: "".to_string(),
                    min: 0.0,
                })
            } else {
                None
            },
            ratings_count: Some(rating_count),
            images: Some(DoubanAvatars {
                small: poster.clone(),
                medium: poster.clone(),
                large: poster,
            }),
            subtype: Some("movie".to_string()),
            directors: Some(directors),
            casts: Some(casts),
            writers: None,
            pubdates: None,
            year,
            genres: Some(genres),
            countries: None, // Parsing from text is complex, skip for now
            mainland_pubdate: None,
            aka: None,
            summary,
            durations: Some(durations),
            seasons_count: None,
            episodes_count: None,
        };
        Ok(serde_json::to_value(detail).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferBestSourceResponse {
    pub best_source: SearchResult,
    pub test_results: Vec<(String, SourceTestResult)>,
}

/// 从多个播放源中选择最佳源
#[tauri::command]
pub async fn prefer_best_source_command(
    sources: Vec<SearchResult>,
) -> Result<PreferBestSourceResponse, String> {
    let client = get_video_client();

    let (best_source, test_results) = prefer_best_source(client, sources).await?;

    Ok(PreferBestSourceResponse {
        best_source,
        test_results,
    })
}

/// 测试单个视频源质量
#[tauri::command]
pub async fn test_video_source_command(
    m3u8_url: String,
) -> Result<SourceTestResult, String> {
    let client = get_video_client();
    test_video_source(client, &m3u8_url).await
}

/// 初始化播放器视图 - 聚合所有初始化数据
///
/// 一次性返回播放器启动所需的所有数据，减少 IPC 通信次数
///
/// # 参数
/// - `source`: 视频源标识
/// - `id`: 视频 ID
/// - `title`: 视频标题（用于搜索相似源）
///
/// # 返回
/// PlayerInitialState 包含：
/// - 视频详情
/// - 其他可用源
/// - 播放记录
/// - 收藏状态
/// - 跳过配置
/// - 播放器配置（去广告、优选开关）
#[tauri::command]
pub async fn initialize_player_view(
    source: String,
    id: String,
    _title: Option<String>,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
    db: State<'_, crate::db::db_client::Db>,
) -> Result<PlayerInitialState, String> {
    // 生成 storage key
    let key = format!("{}+{}", source, id);

    // 并行执行所有数据获取操作
    let (detail_result, play_record, is_favorited, skip_config, player_config) = tokio::join!(
        // 1. 获取视频详情和其他源
        get_video_detail_optimized(
            source.clone(),
            id.clone(),
            storage.clone(),
            cache.clone(),
            Some(true)
        ),
        // 2. 读取播放记录
        async {
            db.with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT episode_index, play_time FROM play_records WHERE key = ?1")?;

                let result = stmt.query_row(params![&key], |row| {
                    Ok(PlayRecordInfo {
                        episode_index: row.get(0)?,
                        play_time: row.get(1)?,
                    })
                });

                match result {
                    Ok(record) => Ok(Some(record)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })
        },
        // 3. 检查收藏状态
        async {
            db.with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT COUNT(*) FROM favorites WHERE key = ?1")?;

                let count: i32 = stmt
                    .query_row(params![&key], |row| row.get(0))?;

                Ok(count > 0)
            })
        },
        // 4. 读取跳过配置
        async {
            db.with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT enable, intro_time, outro_time FROM skip_configs WHERE key = ?1")?;

                let result = stmt.query_row(params![&key], |row| {
                    Ok(SkipConfigInfo {
                        enable: row.get::<_, i32>(0)? != 0,
                        intro_time: row.get::<_, f64>(1)? as i32,
                        outro_time: row.get::<_, f64>(2)? as i32,
                    })
                });

                match result {
                    Ok(config) => Ok(Some(config)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })
        },
        // 5. 读取播放器配置
        async {
            let data = storage.get_data().map_err(|e| e.to_string())?;

            // 尝试从配置中获取播放器配置
            if let Some(player_config) = data.config.get("PlayerConfig") {
                if let Ok(config) = serde_json::from_value::<crate::commands::config::PlayerConfig>(player_config.clone()) {
                    return Ok::<(bool, bool), String>((config.block_ad_enabled, config.optimization_enabled));
                }
            }

            // 返回默认配置
            Ok::<(bool, bool), String>((true, true))
        }
    );

    // 处理视频详情结果
    let detail_response = detail_result?;
    let play_record = play_record?;
    let is_favorited = is_favorited?;
    let skip_config = skip_config?;
    let (block_ad_enabled, optimization_enabled) = player_config?;

    Ok(PlayerInitialState {
        detail: detail_response.detail,
        other_sources: detail_response.other_sources,
        play_record,
        is_favorited,
        skip_config,
        block_ad_enabled,
        optimization_enabled,
    })
}
