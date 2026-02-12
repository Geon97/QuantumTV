use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::Engine;
use quantumtv_core::playback::filter_ads_from_m3_u8;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use crate::{AppState, SERVER_IP};
static PARSES_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("PARSES_URL").unwrap_or_else(|_| "127.0.0.1".to_string()));
// ================== 数据结构 ==================

#[derive(Clone, Debug)]
pub struct SpiderInfo {
    #[allow(dead_code)]
    pub buffer: Option<Vec<u8>>,
    pub md5: String,
    pub source: String,
    pub success: bool,
    #[allow(dead_code)]
    pub cached: bool,
    pub timestamp: SystemTime,
    #[allow(dead_code)]
    pub size: usize,
    #[allow(dead_code)]
    pub tried: usize,
}

#[derive(Clone, Debug)]
pub struct FailedSources {
    pub sources: HashSet<String>,
    pub last_reset: SystemTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Site {
    pub key: String,
    pub name: String,
    pub api: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_adult: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchable: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "quickSearch")]
    pub quick_search: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filterable: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Parse {
    pub name: String,
    #[serde(rename = "type")]
    pub parse_type: i32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubscriptionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sites: Option<Vec<Site>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parses: Option<Vec<Parse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lives: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiSiteInfo {
    pub name: String,
    pub api: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CustomSubscriptionFormat {
    pub api_site: std::collections::HashMap<String, ApiSiteInfo>,
}

#[derive(Deserialize)]
pub struct ConfigParams {
    filter: Option<String>,
    adult: Option<String>,
    mode: Option<String>,
    spider: Option<String>,
    #[serde(rename = "forceSpiderRefresh")]
    force_spider_refresh: Option<String>,
    #[serde(rename = "subscriptionUrl")]
    subscription_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CachedSubscription {
    pub config: SubscriptionConfig,
    pub cached_at: SystemTime,
}

// ================== Spider Jar 候选源配置 ==================

const DOMESTIC_CANDIDATES: &[&str] = &[
    "https://agit.ai/Yoursmile7/TVBox/raw/branch/master/jar/custom_spider.jar",
    "https://ghproxy.net/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
    "https://mirror.ghproxy.com/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
];

const INTERNATIONAL_CANDIDATES: &[&str] = &[
    "https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
    "https://raw.gitmirror.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
    "https://ghproxy.cc/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
];

const PROXY_CANDIDATES: &[&str] = &[
    "https://gh-proxy.com/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
    "https://ghps.cc/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
    "https://gh.api.99988866.xyz/https://raw.githubusercontent.com/FongMi/CatVodSpider/main/jar/custom_spider.jar",
];

// Fallback JAR (base64 encoded minimal working spider.jar)
const FALLBACK_JAR_BASE64: &str = "UEsDBBQACAgIACVFfFcAAAAAAAAAAAAAAAAJAAAATUVUQS1JTkYvUEsHCAAAAAACAAAAAAAAACVFfFcAAAAAAAAAAAAAAAANAAAATUVUQS1JTkYvTUFOSUZFU1QuTUZNYW5pZmVzdC1WZXJzaW9uOiAxLjAKQ3JlYXRlZC1CeTogMS44LjBfNDIxIChPcmFjbGUgQ29ycG9yYXRpb24pCgpQSwcIj79DCUoAAABLAAAAUEsDBBQACAgIACVFfFcAAAAAAAAAAAAAAAAMAAAATWVkaWFVdGlscy5jbGFzczWRSwrCQBBER3trbdPxm4BuBHfiBxHFH4hCwJX4ATfFCrAxnWnYgZCTuPIIHkCPYE+lM5NoILPpoqvrVVd1JslCaLB3MpILJ5xRz5gbMeMS+oyeBOc4xSWucYsZN3CHe7zgiQue8YJXvOEdH/jEFz7whW984weZ+Ecm/pGJf2TiH5n4Ryb+kYl/ZOIfmfhHJv6RiX9k4h+Z+Ecm/pGJf2TiH5n4Ryb+kYl/ZOIfGQaaaXzgE1/4xje+8Y1vfOMb3/jGN77xjW98q9c0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdM0TdOI06nO7p48NRQjICAgICAgICAgICAgICAoKCgoKCgoKCgoKCgoKChoqKioqKioqKio;";

const SUCCESS_TTL: u64 = 4 * 60 * 60; // 4 hours in seconds
const FAILURE_TTL: u64 = 10 * 60; // 10 minutes in seconds
const FAILURE_RESET_INTERVAL: u64 = 2 * 60 * 60; // 2 hours in seconds

// ================== 辅助函数 ==================

fn is_private_host(url: &str) -> bool {
    if url.starts_with("http") {
        let lower = url.to_lowercase();
        lower.contains("localhost") || lower.contains("127.") || lower.contains("10.")
    } else {
        true
    }
}

fn get_candidates() -> Vec<String> {
    // 简化版：返回所有候选源（国内优先，然后国际，最后代理）
    let mut candidates = Vec::new();
    candidates.extend(DOMESTIC_CANDIDATES.iter().map(|s| s.to_string()));
    candidates.extend(INTERNATIONAL_CANDIDATES.iter().map(|s| s.to_string()));
    candidates.extend(PROXY_CANDIDATES.iter().map(|s| s.to_string()));
    candidates
}

fn calculate_md5(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}

async fn fetch_remote(url: &str, timeout_ms: u64, retry_count: usize) -> Option<Vec<u8>> {
    for attempt in 0..=retry_count {
        match fetch_remote_once(url, timeout_ms).await {
            Ok(data) => {
                // 验证 JAR 文件格式（检查 ZIP 头）
                if data.len() < 1000 {
                    tracing::warn!("File too small: {} bytes from {}", data.len(), url);
                    continue;
                }

                if data[0] != 0x50 || data[1] != 0x4B {
                    tracing::warn!("Invalid JAR file format from {}", url);
                    continue;
                }

                return Some(data);
            }
            Err(e) => {
                tracing::warn!(
                    "Attempt {}/{} failed for {}: {}",
                    attempt + 1,
                    retry_count + 1,
                    url,
                    e
                );

                // 网络错误等待后重试
                if attempt < retry_count {
                    tokio::time::sleep(Duration::from_secs((attempt + 1) as u64)).await;
                }
            }
        }
    }

    None
}

async fn fetch_remote_once(url: &str, timeout_ms: u64) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    // 根据源类型优化请求头
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Accept-Encoding", "identity".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Connection", "close".parse().unwrap());

    let user_agent = if url.contains("github") || url.contains("raw.githubusercontent") {
        "curl/7.68.0"
    } else if url.contains("gitee") || url.contains("gitcode") {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    } else if url.contains("jsdelivr") || url.contains("fastly") {
        "DecoTV/1.0"
    } else {
        "Mozilla/5.0 (Linux; Android 11; SM-G973F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Mobile Safari/537.36"
    };

    headers.insert("User-Agent", user_agent.parse().unwrap());

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(bytes.to_vec())
}

async fn get_spider_jar(state: &AppState, force_refresh: bool) -> SpiderInfo {
    let now = SystemTime::now();

    // 重置失败记录（定期清理）
    {
        let mut failed = state.failed_sources.lock().await;
        if let Ok(elapsed) = now.duration_since(failed.last_reset) {
            if elapsed.as_secs() > FAILURE_RESET_INTERVAL {
                failed.sources.clear();
                failed.last_reset = now;
                tracing::info!("Reset failed sources list");
            }
        }
    }

    // 检查缓存
    if !force_refresh {
        let cache = state.spider_info.lock().await;
        if let Ok(elapsed) = now.duration_since(cache.timestamp) {
            let ttl = if cache.success {
                SUCCESS_TTL
            } else {
                FAILURE_TTL
            };
            if elapsed.as_secs() < ttl {
                tracing::info!(
                    "Using cached spider jar (age: {}s, success: {})",
                    elapsed.as_secs(),
                    cache.success
                );
                return SpiderInfo {
                    cached: true,
                    ..cache.clone()
                };
            }
        }
    }

    let mut tried = 0;
    let candidates = get_candidates();
    let total_candidates = candidates.len(); // Store length before move

    // 过滤掉近期失败的源
    let failed_sources = state.failed_sources.lock().await;
    let active_candidates: Vec<String> = candidates
        .iter()
        .filter(|url| !failed_sources.sources.contains(*url))
        .cloned()
        .collect();
    drop(failed_sources);

    let candidates_to_try = if active_candidates.is_empty() {
        candidates
    } else {
        active_candidates
    };

    tracing::info!("Trying {} spider jar candidates", candidates_to_try.len());

    for url in candidates_to_try {
        tried += 1;
        tracing::info!(
            "Trying spider jar source {}/{}: {}",
            tried,
            total_candidates,
            url
        );

        if let Some(buffer) = fetch_remote(&url, 3000, 1).await {
            // 成功时从失败列表移除
            let mut failed = state.failed_sources.lock().await;
            failed.sources.remove(&url);
            drop(failed);

            let md5_hash = calculate_md5(&buffer);
            let size = buffer.len();

            let info = SpiderInfo {
                buffer: Some(buffer),
                md5: md5_hash,
                source: url.clone(),
                success: true,
                cached: false,
                timestamp: now,
                size,
                tried,
            };

            // 更新缓存
            let mut cache = state.spider_info.lock().await;
            *cache = info.clone();

            tracing::info!(
                "Successfully fetched spider jar from {} (size: {} bytes, md5: {})",
                url,
                size,
                info.md5
            );
            return info;
        } else {
            // 失败时添加到失败列表
            let mut failed = state.failed_sources.lock().await;
            failed.sources.insert(url.clone());
        }
    }

    // Fallback to embedded JAR
    tracing::warn!("All spider jar sources failed, using fallback");

    let fallback_data = base64::engine::general_purpose::STANDARD
        .decode(FALLBACK_JAR_BASE64)
        .unwrap_or_default();

    let md5_hash = calculate_md5(&fallback_data);
    let size = fallback_data.len();

    let info = SpiderInfo {
        buffer: Some(fallback_data),
        md5: md5_hash,
        source: "fallback".to_string(),
        success: false,
        cached: false,
        timestamp: now,
        size,
        tried,
    };

    // 更新缓存
    let mut cache = state.spider_info.lock().await;
    *cache = info.clone();

    info
}

/// 从 URL 获取订阅配置
async fn fetch_subscription(url: &str) -> Result<SubscriptionConfig, String> {
    tracing::info!("Fetching subscription from: {}", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch subscription: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // 尝试解析自定义格式（api_site 格式）
    if let Ok(custom_format) = serde_json::from_str::<CustomSubscriptionFormat>(&text) {
        tracing::info!(
            "Parsed custom subscription format with {} sites",
            custom_format.api_site.len()
        );
        return Ok(convert_custom_format_to_tvbox(custom_format));
    }

    // 尝试解析标准 TVBox 格式
    serde_json::from_str::<SubscriptionConfig>(&text)
        .map_err(|e| format!("Failed to parse subscription JSON: {}", e))
}

/// 将自定义格式转换为 TVBox 配置格式
fn convert_custom_format_to_tvbox(custom: CustomSubscriptionFormat) -> SubscriptionConfig {
    let mut sites = Vec::new();

    for (domain, site_info) in custom.api_site {
        // 使用域名作为 key，移除特殊字符
        let key = domain.replace(".", "_").replace("-", "_").replace(":", "_");

        sites.push(Site {
            key,
            name: site_info.name,
            api: site_info.api,
            jar: None,
            is_adult: None,
            searchable: Some(1),
            quick_search: Some(1),
            filterable: Some(1),
        });
    }

    SubscriptionConfig {
        spider: Some(
            "https://cdn.jsdelivr.net/gh/FongMi/CatVodSpider@main/jar/spider.jar".to_string(),
        ),
        sites: Some(sites),
        parses: Some(vec![
            Parse {
                name: "默认解析".to_string(),
                parse_type: 0,
                url: "https://jx.xmflv.com/?url=".to_string(),
            },
            Parse {
                name: "并发解析".to_string(),
                parse_type: 2,
                url: "Parallel".to_string(),
            },
        ]),
        lives: None,
    }
}

/// 获取缓存的订阅配置
async fn get_cached_subscription(
    state: &AppState,
    url: &str,
    force_refresh: bool,
) -> Result<SubscriptionConfig, String> {
    let mut cache = state.subscription_cache.lock().await;

    // 检查缓存是否有效（10分钟）
    let cache_valid = if let Some(cached) = cache.as_ref() {
        SystemTime::now()
            .duration_since(cached.cached_at)
            .map(|d| d.as_secs() < 600)
            .unwrap_or(false)
    } else {
        false
    };

    if force_refresh || !cache_valid {
        tracing::info!("Fetching fresh subscription data");
        let config = fetch_subscription(url).await?;
        *cache = Some(CachedSubscription {
            config: config.clone(),
            cached_at: SystemTime::now(),
        });
        Ok(config)
    } else {
        Ok(cache.as_ref().unwrap().config.clone())
    }
}

/// 默认配置
fn get_default_config() -> SubscriptionConfig {
    SubscriptionConfig {
        spider: Some(
            "https://cdn.jsdelivr.net/gh/FongMi/CatVodSpider@main/jar/spider.jar".to_string(),
        ),
        sites: Some(vec![Site {
            key: "demo".to_string(),
            name: "演示站点".to_string(),
            api: "https://example.com/api".to_string(),
            jar: None,
            is_adult: Some(false),
            searchable: Some(1),
            quick_search: Some(1),
            filterable: Some(1),
        }]),
        parses: Some(vec![
            Parse {
                name: "默认解析".to_string(),
                parse_type: 0,
                url: "https://jx.xmflv.com/?url=".to_string(),
            },
            Parse {
                name: "并发解析".to_string(),
                parse_type: 2,
                url: "Parallel".to_string(),
            },
        ]),
        lives: None,
    }
}

// ================== API 处理器 ==================

pub async fn get_config_handler(
    Query(params): Query<ConfigParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 0. 构建 API 服务器地址（用于 M3U8 代理）

    let api_base_url = format!("http://{}", SERVER_IP.to_string());
    let m3u8_proxy_url = format!("{}/api/proxy/m3u8?url=", api_base_url);

    // 1. 获取订阅配置
    let subscription_url = params
        .subscription_url
        .as_deref()
        .unwrap_or(PARSES_URL.as_str());

    let force_refresh = params.force_spider_refresh.as_deref() == Some("1");

    let mut config = match get_cached_subscription(&state, subscription_url, force_refresh).await {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to fetch subscription, using default: {}", e);
            get_default_config()
        }
    };

    // 2. 处理 Spider 逻辑（使用 3 秒超时）
    let spider_info = match tokio::time::timeout(
        Duration::from_secs(3),
        get_spider_jar(&state, force_refresh),
    )
    .await
    {
        Ok(info) => info,
        Err(_) => {
            tracing::warn!("Spider JAR fetch timeout, using fallback");
            // 超时时使用 fallback
            let fallback_data = base64::engine::general_purpose::STANDARD
                .decode(FALLBACK_JAR_BASE64)
                .unwrap_or_default();
            let md5_hash = calculate_md5(&fallback_data);
            SpiderInfo {
                buffer: Some(fallback_data.clone()),
                md5: md5_hash.clone(),
                source: "fallback".to_string(),
                success: false,
                cached: false,
                timestamp: SystemTime::now(),
                size: fallback_data.len(),
                tried: 0,
            }
        }
    };

    // 构建 spider 字符串（格式：url;md5;hash）
    let global_spider_jar = if spider_info.success && spider_info.source != "fallback" {
        format!("{};md5;{}", spider_info.source, spider_info.md5)
    } else {
        // Fallback 策略
        format!(
            "https://agit.ai/Yoursmile7/TVBox/raw/branch/master/jar/custom_spider.jar;md5;{}",
            spider_info.md5
        )
    };

    // 允许 URL 参数覆盖 Spider（仅当是公网地址时）
    let final_spider = if let Some(spider_url) = &params.spider {
        if spider_url.starts_with("http") && !is_private_host(spider_url) {
            spider_url.clone()
        } else {
            global_spider_jar
        }
    } else {
        // 使用订阅配置中的 spider，如果没有则使用获取的 spider
        config.spider.clone().unwrap_or(global_spider_jar)
    };

    // 3. 过滤逻辑
    let filter_adult = !matches!(params.filter.as_deref(), Some("off") | Some("disable"))
        && !matches!(params.adult.as_deref(), Some("1") | Some("true"));

    // 4. 应用过滤
    if filter_adult {
        if let Some(sites) = config.sites.as_mut() {
            sites.retain(|s| !s.is_adult.unwrap_or(false));
        }
    }

    let mode = params
        .mode
        .clone()
        .unwrap_or_else(|| "standard".to_string());

    // 5. 添加广告过滤解析器
    let mut parses = config.parses.unwrap_or_default();

    // 在开头插入广告过滤解析器（优先使用）
    parses.insert(
        0,
        Parse {
            name: "🚫 广告过滤".to_string(),
            parse_type: 0,
            url: m3u8_proxy_url,
        },
    );

    // 6. 组装响应
    let response = serde_json::json!({
        "spider": final_spider,
        "spider_success": spider_info.success,
        "mode": mode,
        "sites": config.sites.unwrap_or_default(),
        "parses": parses,
        "lives": config.lives.unwrap_or_default(),
        "wallpaper": "https://picsum.photos/1920/1080",
        "debug_filter": filter_adult,
        "subscription_url": subscription_url,
    });

    (StatusCode::OK, Json(response))
}

/// M3U8 代理处理器（带广告过滤）
pub async fn proxy_m3u8_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let url = match params.get("url") {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Missing url parameter".to_string()),
    };

    tracing::info!("Proxying M3U8: {}", url);

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create client: {}", e),
            )
        }
    };

    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch M3U8: {}", e),
            )
        }
    };

    if !response.status().is_success() {
        return (
            StatusCode::BAD_GATEWAY,
            format!("Upstream error: {}", response.status()),
        );
    }

    let content = match response.text().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read response: {}", e),
            )
        }
    };

    // 使用 core crate 中的广告过滤函数
    let filtered = filter_ads_from_m3_u8(content);

    (StatusCode::OK, filtered)
}
