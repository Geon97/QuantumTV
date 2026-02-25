use quantumtv_core::is_adult_source;
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Site {
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub site_type: i32,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changeable: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Parse {
    pub name: String,
    #[serde(rename = "type")]
    pub parse_type: i32,
    pub url: String,
}

/// 根据 API URL 判断站点类型
/// MacCMS 资源站使用 type: 1，Spider 站点使用 type: 3
pub fn determine_site_type(api_url: &str) -> i32 {
    if api_url.contains("/api.php/provide/vod")
        || api_url.contains("/api.php/provide/")
        || api_url.contains("maccms")
    {
        1 // MacCMS 资源站
    } else {
        3 // Spider 站点
    }
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
/// 将自定义格式转换为 TVBox 配置格式
fn convert_custom_format_to_tvbox(
    custom: CustomSubscriptionFormat,
    adult: bool,
) -> SubscriptionConfig {
    let mut sites = Vec::new();

    for (domain, site_info) in custom.api_site {
        // 如果不允许成人 跳过
        if !adult {
            let source_str = format!("{} {}", site_info.name, site_info.api);
            if is_adult_source(&source_str) {
                continue; // 成人源直接跳过
            }
        }

        // 使用域名作为 key，移除特殊字符
        let key = domain.replace(".", "_").replace("-", "_").replace(":", "_");

        // 使用统一的 site_type 判断逻辑
        let site_type = determine_site_type(&site_info.api);

        sites.push(Site {
            key,
            name: site_info.name,
            site_type,
            api: site_info.api,
            jar: None,
            is_adult: None,
            searchable: Some(1),
            quick_search: Some(1),
            filterable: Some(1),
            changeable: None,
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

/// 从 URL 获取订阅配置
pub async fn fetch_subscription(url: &str, adult: bool) -> Result<SubscriptionConfig, String> {
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
        return Ok(convert_custom_format_to_tvbox(custom_format, adult));
    }

    // 尝试解析标准 TVBox 格式
    serde_json::from_str::<SubscriptionConfig>(&text)
        .map_err(|e| format!("Failed to parse subscription JSON: {}", e))
}
