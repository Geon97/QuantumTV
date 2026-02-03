use crate::storage::StorageManager;
use image::{GenericImageView, ImageOutputFormat};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct VideoCacheManager {
    pub cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    pub keys: Arc<RwLock<Vec<String>>>,
}

impl VideoCacheManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get(&self, url: &str) -> Option<Vec<u8>> {
        let cache = self.cache.read().await;
        cache.get(url).cloned()
    }

    pub async fn set(&self, url: String, data: Vec<u8>) {
        let mut cache = self.cache.write().await;
        let mut keys = self.keys.write().await;

        if !cache.contains_key(&url) {
            // 缓存大小到 50
            if keys.len() >= 50 {
                if let Some(oldest) = keys.get(0).cloned() {
                    cache.remove(&oldest);
                    keys.remove(0);
                }
            }
            keys.push(url.clone());
        }
        cache.insert(url, data);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub poster: String,
    pub episodes: Vec<String>,
    pub episodes_titles: Vec<String>,
    pub source: String,
    pub source_name: String,
    pub class: Option<String>,
    pub year: Option<String>,
    pub desc: Option<String>,
    pub type_name: Option<String>,
    pub douban_id: Option<i32>,
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
    storage: State<'_, StorageManager>,
) -> Result<Vec<SearchResult>, String> {
    let data = storage.get_data()?;
    let config = &data.config;

    let sites = if let Some(source_config) = config.get("SourceConfig").and_then(|v| v.as_array()) {
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

    let disable_yellow_filter = config
        .get("SiteConfig")
        .and_then(|v| v.get("DisableYellowFilter"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let mut handles = Vec::new();
    for site in &sites {
        let client = client.clone();
        let query = query.clone();
        let site_clone = site.clone();

        let handle = tokio::spawn(async move {
            let url = format!(
                "{}?ac=videolist&wd={}",
                site_clone.api,
                urlencoding::encode(&query)
            );
            let resp = client.get(&url).send().await;

            match resp {
                Ok(res) if res.status().is_success() => {
                    let body = res.text().await.unwrap_or_default();
                    if let Ok(search_res) = serde_json::from_str::<ApiSearchResponse>(&body) {
                        return search_res
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
                }
                _ => {}
            }
            vec![]
        });
        handles.push(handle);
    }

    let mut all_results = Vec::new();
    for handle in handles {
        if let Ok(results) = handle.await {
            all_results.extend(results);
        }
    }

    // Filter duplicates
    let mut unique_results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for res in all_results {
        let key = format!("{}|{}", res.source, res.id);
        if seen.insert(key) {
            // Filter adult if needed
            if !disable_yellow_filter {
                let type_name = res.type_name.as_deref().unwrap_or("");
                let is_adult_site = sites
                    .iter()
                    .find(|s| s.key == res.source)
                    .and_then(|s| s.is_adult)
                    .unwrap_or(false);
                if is_adult_site || YELLOW_WORDS.iter().any(|w| type_name.contains(w)) {
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
    let client = reqwest::Client::new();

    let url = format!("{}?ac=videolist&ids={}", site.api, id);
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch detail: {}", resp.status()));
    }

    let body = resp.text().await.map_err(|e| e.to_string())?;
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

    // 2. 缓存未命中，通过 HTTP 请求获取图片
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        ),
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
    let client = reqwest::Client::new();
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

    // Add Referer only for douban domains
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

    // 2. 执行网络请求
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let req_method = match method_str.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "HEAD" => reqwest::Method::HEAD,
        _ => reqwest::Method::GET,
    };
    let request_builder = client.request(req_method, &url);

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

    let resp = request_builder
        .headers(final_headers.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let body = body_bytes.to_vec();

    // 3. 只有成功的 GET 请求才存入缓存并触发预取
    if is_get && status == 200 {
        cache_manager.set(url.clone(), body.clone()).await;

        // 4. 预取逻辑: 如果是 TS 分片，预测后续 URL 并启动后台下载
        if url.contains(".ts") {
            let cache_arc = cache_manager.cache.clone();
            let keys_arc = cache_manager.keys.clone();
            let headers_clone = headers_opt.clone();

            tokio::spawn(async move {
                prefetch_next_segments(url, headers_clone, cache_arc, keys_arc).await;
            });
        }
    }

    Ok(FetchBinaryResponse { status, body })
}

// 预测并预取后续分片
async fn prefetch_next_segments(
    current_url: String,
    headers: Option<HashMap<String, String>>,
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    keys: Arc<RwLock<Vec<String>>>,
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

    // 优化的 HTTP 客户端配置
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    // 预取接下来的 6 个分片
    let mut handles = Vec::new();
    for i in 1..=6 {
        let next_num = current_num + i;
        let next_num_str = format!("{:0width$}", next_num, width = padding);
        let next_url = format!("{}{}{}", prefix, next_num_str, suffix);

        // 如果已经在缓存中，跳过
        {
            let c = cache.read().await;
            if c.contains_key(&next_url) {
                continue;
            }
        }

        // 并发下载
        let client_clone = client.clone();
        let headers_clone = headers.clone();
        let cache_clone = cache.clone();
        let keys_clone = keys.clone();
        let next_url_clone = next_url.clone();

        let handle = tokio::spawn(async move {
            let mut final_headers = HeaderMap::new();
            if let Some(h) = headers_clone {
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

            let resp = client_clone
                .get(&next_url_clone)
                .headers(final_headers)
                .send()
                .await;
            if let Ok(r) = resp {
                if r.status() == 200 {
                    if let Ok(data) = r.bytes().await {
                        let mut c = cache_clone.write().await;
                        let mut k = keys_clone.write().await;

                        if !c.contains_key(&next_url_clone) {
                            if k.len() >= 80 {
                                // 预取缓存可以更大
                                if let Some(oldest) = k.get(0).cloned() {
                                    c.remove(&oldest);
                                    k.remove(0);
                                }
                            }
                            k.push(next_url_clone.clone());
                        }
                        c.insert(next_url_clone, data.to_vec());
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
    let client = reqwest::Client::new();
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
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://movie.douban.com/"),
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
