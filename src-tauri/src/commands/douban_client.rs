use crate::commands::bangumi::get_bangumi_calendar_data;
use crate::db::page_cache::PageCacheManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap}, error::Error};
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")] // 不区分大小写
pub enum Kind {
    Tv,
    Movie,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanCategoriesParams {
    kind: Kind,
    category: String,
    #[serde(rename = "type")]
    type_: String,
    page_limit: Option<i32>,
    page_start: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanCategoryItemsPic {
    large: String,
    normal: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanCategoryItemsRating {
    value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanCategoryItems {
    id: String,
    title: String,
    #[serde(default)] // 如果 json 里是 null，这会将其转为空字符串 ""
    card_subtitle: String,
    pic: Option<DoubanCategoryItemsPic>,
    rating: Option<DoubanCategoryItemsRating>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanListApiResponseSubjects {
    id: String,
    title: String,
    card_subtitle: String,
    cover: String,
    rate: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanRecommendApiResponseItems {
    id: String,
    title: String,
    year: String,
    #[serde(rename = "type")]
    type_: String,
    pic: Option<DoubanCategoryItemsPic>,
    rating: Option<DoubanCategoryItemsRating>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanCategoryApiResponse {
    total: i32,
    items: Option<Vec<DoubanCategoryItems>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanListApiResponse {
    total: i32,
    subjects: Option<Vec<DoubanListApiResponseSubjects>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanRecommendApiResponse {
    total: i32,
    items: Option<Vec<DoubanRecommendApiResponseItems>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum DoubanProxyType {
    Direct,
    CorsProxyZwei,
    CmliussssCdnTencent,
    CmliussssCdnAli,
    CorsAnywhere,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoubanProxyConfig {
    proxy_type: DoubanProxyType,
    proxy_url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanItem {
    pub id: String,
    pub title: String,
    pub poster: String,
    pub rate: String,
    pub year: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanResult {
    pub code: i32,
    pub message: String,
    pub list: Vec<DoubanItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubanPageRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub primary_selection: String,
    pub secondary_selection: String,
    pub multi_level_selection: Option<HashMap<String, String>>,
    pub selected_weekday: Option<String>,
    pub page: Option<i32>,
    pub page_limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanPageResponse {
    pub list: Vec<DoubanItem>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanCustomCategory {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub category_type: String,
    pub query: String,
    pub disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubanDefaultsRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub custom_categories: Option<Vec<DoubanCustomCategory>>,
    pub fallback_secondary: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubanDefaultsResponse {
    pub primary_selection: String,
    pub secondary_selection: String,
    pub multi_level_selection: HashMap<String, String>,
    pub cache_enabled: bool,
    pub require_secondary: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum DoubanPageMode {
    Categories,
    Recommends,
    Custom,
    AnimeDaily,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanRecommendsParams {
    kind: Kind,
    page_limit: Option<i32>,
    page_start: Option<i32>,
    category: String,
    format: String,
    label: String,
    region: String,
    year: String,
    platform: String,
    sort: String,
}
impl DoubanRecommendsParams {
    fn page_limit(&self) -> i32 {
        self.page_limit.unwrap_or(20)
    }
    fn page_start(&self) -> i32 {
        self.page_start.unwrap_or(0)
    }
    fn kind(&self) -> &str {
        match self.kind {
            Kind::Tv => "tv",
            Kind::Movie => "movie",
        }
    }
    fn normalized_params(&self) -> (String, String, String, String, String, String, String) {
        let category = if self.category == "all" {
            "".to_string()
        } else {
            self.category.clone()
        };
        let format = if self.format == "all" {
            "".to_string()
        } else {
            self.format.clone()
        };
        let label = if self.label == "all" {
            "".to_string()
        } else {
            self.label.clone()
        };
        let region = if self.region == "all" {
            "".to_string()
        } else {
            self.region.clone()
        };
        let year = if self.year == "all" {
            "".to_string()
        } else {
            self.year.clone()
        };
        let platform = if self.platform == "all" {
            "".to_string()
        } else {
            self.platform.clone()
        };
        let sort = if self.sort == "T" {
            "".to_string()
        } else {
            self.sort.clone()
        };
        (category, format, label, region, year, platform, sort)
    }
    fn base_url(&self, use_tencent_cdn: bool, use_ali_cdn: bool) -> String {
        if use_tencent_cdn {
            format!(
                "https://m.douban.cmliussss.net/rexxar/api/v2/{}/recommend",
                self.kind()
            )
        } else if use_ali_cdn {
            format!(
                "https://m.douban.cmliussss.com/rexxar/api/v2/{}/recommend",
                self.kind()
            )
        } else {
            format!(
                "https://m.douban.com/rexxar/api/v2/{}/recommend",
                self.kind()
            )
        }
    }
    fn build_query_string(&self) -> Result<String, serde_json::Error> {
        let (category, format, label, region, year, platform, sort) = self.normalized_params();
        let mut selected_categories = HashMap::new();
        selected_categories.insert("类型", category.clone());
        if !format.is_empty() {
            selected_categories.insert("形式", format.clone());
        }
        if !label.is_empty() {
            selected_categories.insert("标签", label.clone());
        }
        if !region.is_empty() {
            selected_categories.insert("地区", region.clone());
        }
        if !year.is_empty() {
            selected_categories.insert("年份", year.clone());
        }
        let mut tags = Vec::new();
        if !category.is_empty() {
            tags.push(category.clone());
        }
        if category.is_empty() && !format.is_empty() {
            tags.push(format.clone());
        }
        if !label.is_empty() {
            tags.push(label.clone());
        }
        if !region.is_empty() {
            tags.push(region.clone());
        }
        if !year.is_empty() {
            tags.push(year.clone());
        }
        if !platform.is_empty() {
            tags.push(platform.clone());
        }
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("refresh", "0");
        serializer.append_pair("start", &self.page_start().to_string());
        serializer.append_pair("count", &self.page_limit().to_string());
        serializer.append_pair(
            "selected_categories",
            &serde_json::to_string(&selected_categories)?,
        );
        serializer.append_pair("uncollect", "false");
        serializer.append_pair("score_range", "0,10");
        serializer.append_pair("tags", &tags.join(","));
        if !sort.is_empty() {
            serializer.append_pair("sort", &sort);
        }

        Ok(serializer.finish())
    }
}

impl From<&str> for DoubanProxyType {
    fn from(value: &str) -> Self {
        match value {
            "direct" => DoubanProxyType::Direct,
            "cors-proxy-zwei" => DoubanProxyType::CorsProxyZwei,
            "cmliussss-cdn-tencent" => DoubanProxyType::CmliussssCdnTencent,
            "cmliussss-cdn-ali" => DoubanProxyType::CmliussssCdnAli,
            "cors-anywhere" => DoubanProxyType::CorsAnywhere,
            "custom" => DoubanProxyType::Custom,
            _ => DoubanProxyType::CmliussssCdnTencent,
        }
    }
}
fn get_douban_proxy_config(
    douban_data_source: Option<String>,
    runtime_proxy_type: Option<String>,
    douban_proxy_url: Option<String>,
    runtime_proxy_url: Option<String>,
) -> DoubanProxyConfig {
    let proxy_type_str = douban_data_source
        .or(runtime_proxy_type)
        .unwrap_or_else(|| "cmliussss-cdn-tencent".to_string());
    let proxy_type = DoubanProxyType::from(proxy_type_str.as_str());
    let proxy_url = douban_proxy_url.or(runtime_proxy_url).unwrap_or_default();
    DoubanProxyConfig {
        proxy_type,
        proxy_url,
    }
}
impl DoubanCategoriesParams {
    pub fn new(
        kind: Kind,
        category: impl Into<String>,
        type_: impl Into<String>,
        page_limit: Option<i32>,
        page_start: Option<i32>,
    ) -> Self {
        Self {
            kind,
            category: category.into(),
            type_: type_.into(),
            page_limit,
            page_start,
        }
    }

    fn page_limit(&self) -> i32 {
        self.page_limit.unwrap_or(20)
    }
    fn page_start(&self) -> i32 {
        self.page_start.unwrap_or(0)
    }
    fn kind(&self) -> &str {
        match self.kind {
            Kind::Tv => "tv",
            Kind::Movie => "movie",
        }
    }
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.category.is_empty() || self.type_.is_empty() {
            return Err("category 和 type 参数不能为空".into());
        }

        let limit = self.page_limit();
        if !(1..=100).contains(&limit) {
            return Err("page_limit 必须在 1-100 之间".into());
        }

        let start = self.page_start();
        if start < 0 {
            return Err("page_start 不能小于 0".into());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanListParams {
    tag: String,
    type_: String,
    page_limit: Option<i32>,
    page_start: Option<i32>,
}
impl DoubanListParams {
    fn page_limit(&self) -> i32 {
        self.page_limit.unwrap_or(20)
    }
    fn page_start(&self) -> i32 {
        self.page_start.unwrap_or(0)
    }
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.tag.is_empty() || self.type_.is_empty() {
            return Err("tag 和 type 参数不能为空".into());
        }

        if self.type_ != "tv" && self.type_ != "movie" {
            return Err("type 参数必须是 tv 或 movie".into());
        }

        let limit = self.page_limit();
        if !(1..=100).contains(&limit) {
            return Err("page_limit 必须在 1-100 之间".into());
        }

        let start = self.page_start();
        if start < 0 {
            return Err("page_start 不能小于 0".into());
        }

        Ok(())
    }
}

async fn fetch_douban_categories(
    params: DoubanCategoriesParams,
    proxy_url: String,
    use_tencent_cdn: bool,
    use_ali_cdn: bool,
) -> Result<DoubanResult, Box<dyn Error>> {
    params.validate()?;
    let page_limit = params.page_limit();
    let page_start = params.page_start();
    let base_url = if use_tencent_cdn {
        "https://m.douban.cmliussss.net"
    } else if use_ali_cdn {
        "https://m.douban.cmliussss.com"
    } else {
        "https://m.douban.com"
    };
    let target = format!(
        "{}/rexxar/api/v2/subject/recent_hot/{}\
        ?start={}&limit={}&category={}&type={}",
        base_url,
        params.kind(),
        page_start,
        page_limit,
        params.category,
        params.type_
    );
    let client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30)) // 30秒超时
        .connect_timeout(std::time::Duration::from_secs(10)) // 10秒连接超时
        .pool_max_idle_per_host(10) // 连接池优化
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .http1_only()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd(); // TCP keepalive

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url)?)
    } else {
        client_builder
    };

    let client = client_builder.build()?;

    // 添加重试机制
    let mut retries = 0;
    let max_retries = 3;

    loop {
        match client
            .get(&target)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Referer", "https://movie.douban.com/")
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "identity")
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(format!("HTTP error! Status: {}", response.status()).into());
                }
                let content_encoding = response
                    .headers()
                    .get(reqwest::header::CONTENT_ENCODING)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("none")
                    .to_string();
                let content_type = response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();
                let douban_data: DoubanCategoryApiResponse = response.json().await.map_err(
                    |e| format!("JSON parse error (ce={content_encoding}, ct={content_type}): {e}"),
                )?;
                let list = douban_data
                    .items
                    .unwrap_or_default()
                    .into_iter()
                    .map(|items| {
                        let poster = items.pic.map(|pic| pic.normal).unwrap_or_default();
                        let rate = items
                            .rating
                            .map(|rating| rating.value.to_string())
                            .unwrap_or_default();
                        let year = items
                            .card_subtitle
                            .chars()
                            .filter(|c| c.is_digit(10))
                            .collect::<String>();
                        DoubanItem {
                            id: items.id,
                            title: items.title,
                            poster,
                            rate,
                            year,
                        }
                    })
                    .collect();
                return Ok(DoubanResult {
                    code: 200,
                    message: "获取成功".to_string(),
                    list,
                });
            }
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(format!("请求失败，已重试 {} 次: {}", max_retries, e).into());
                }
                // 等待一段时间后重试（指数退避）
                tokio::time::sleep(std::time::Duration::from_millis(500 * retries as u64)).await;
            }
        }
    }
}

async fn fetch_douban_recommends(
    params: DoubanRecommendsParams,
    proxy_url: String,
    use_tencent_cdn: bool,
    use_ali_cdn: bool,
) -> Result<DoubanResult, Box<dyn Error>> {
    let base_url = params.base_url(use_tencent_cdn, use_ali_cdn);
    let query = params.build_query_string()?;
    let target_url = format!("{}?{}", base_url, query);

    let client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .http1_only()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd();

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url)?)
    } else {
        client_builder
    };

    let client = client_builder.build()?;

    let response = client
        .get(&target_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Referer", "https://movie.douban.com/")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "identity")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error! Status: {}", response.status()).into());
    }
    let content_encoding = response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none")
        .to_string();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let douban_data: DoubanRecommendApiResponse = response.json().await.map_err(|e| {
        format!("JSON parse error (ce={content_encoding}, ct={content_type}): {e}")
    })?;
    let list = douban_data
        .items
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.type_ == "movie" || item.type_ == "tv")
        .map(|item| {
            let poster = item
                .pic
                .as_ref()
                .and_then(|pic| {
                    if !pic.normal.is_empty() {
                        Some(pic.normal.clone())
                    } else if !pic.large.is_empty() {
                        Some(pic.large.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let rate = item
                .rating
                .as_ref()
                .map(|r| format!("{:.1}", r.value))
                .unwrap_or_default();

            DoubanItem {
                id: item.id,
                title: item.title,
                poster: poster,
                rate: rate,
                year: item.year,
            }
        })
        .collect();

    Ok(DoubanResult {
        code: 200,
        message: "获取成功".to_string(),
        list,
    })
}
#[tauri::command]
pub async fn get_douban_list(params: DoubanListParams) -> Result<DoubanResult, String> {
    let proxy_config = get_douban_proxy_config(None, None, None, None);
    let proxy_type = proxy_config.proxy_type;
    let proxy_url = proxy_config.proxy_url;

    match proxy_type {
        DoubanProxyType::CorsProxyZwei => {
            fetch_douban_list(
                params,
                "https://ciao-cors.is-an.org/".to_string(),
                false,
                false,
            )
            .await
        }
        DoubanProxyType::CmliussssCdnTencent => {
            fetch_douban_list(params, "".to_string(), true, false).await
        }
        DoubanProxyType::CmliussssCdnAli => {
            fetch_douban_list(params, "".to_string(), false, true).await
        }
        DoubanProxyType::CorsAnywhere => {
            fetch_douban_list(
                params,
                "https://cors-anywhere.com/".to_string(),
                false,
                false,
            )
            .await
        }
        DoubanProxyType::Custom => fetch_douban_list(params, proxy_url, false, false).await,
        DoubanProxyType::Direct => {
            // 直接调用本地接口示例
            let url = format!(
                "/api/douban?tag={}&type={}&pageSize={}&pageStart={}",
                params.tag,
                params.type_,
                params.page_limit.unwrap_or(20),
                params.page_start.unwrap_or(0)
            );

            let client = reqwest::Client::new();
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!("HTTP error! Status: {}", resp.status()));
            }

            let result = resp
                .json::<DoubanResult>()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            Ok(result)
        }
    }
}

#[tauri::command]
pub async fn get_douban_categories(params: DoubanCategoriesParams) -> Result<DoubanResult, String> {
    let douban_proxy_config = get_douban_proxy_config(None, None, None, None);
    let proxy_type = douban_proxy_config.proxy_type;
    let proxy_url = douban_proxy_config.proxy_url;
    let result = match proxy_type {
        DoubanProxyType::Direct => fetch_douban_categories(params, proxy_url, false, false).await,
        DoubanProxyType::CorsProxyZwei => {
            fetch_douban_categories(
                params,
                "https://ciao-cors.is-an.org/".to_string(),
                false,
                false,
            )
            .await
        }
        DoubanProxyType::CmliussssCdnTencent => {
            fetch_douban_categories(params, "".to_string(), true, false).await
        }
        DoubanProxyType::CmliussssCdnAli => {
            fetch_douban_categories(params, "".to_string(), false, true).await
        }
        DoubanProxyType::CorsAnywhere => {
            fetch_douban_categories(
                params,
                "https://cors-anywhere.com/".to_string(),
                false,
                false,
            )
            .await
        }
        DoubanProxyType::Custom => fetch_douban_categories(params, proxy_url, false, false).await,
    };
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_douban_list(
    params: DoubanListParams,
    proxy_url: String,
    use_tencent_cdn: bool,
    use_ali_cdn: bool,
) -> Result<DoubanResult, String> {
    let _ = params.validate();
    let page_limit = params.page_limit();
    let page_start = params.page_start();
    // 构造请求 URL
    let base_url = if use_tencent_cdn {
        "https://movie.douban.cmliussss.net/j/search_subjects"
    } else if use_ali_cdn {
        "https://movie.douban.cmliussss.com/j/search_subjects"
    } else {
        "https://movie.douban.com/j/search_subjects"
    };

    let target = format!(
        "{}?type={}&tag={}&sort=recommend&page_limit={}&page_start={}",
        base_url, params.type_, params.tag, page_limit, page_start
    );
    let client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .http1_only()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd();

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
    } else {
        client_builder
    };

    let client = client_builder.build().map_err(|e| e.to_string())?;

    let response = client
        .get(&target)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Referer", "https://movie.douban.com/")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error! Status: {}", response.status()).into());
    }

    let content_encoding = response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none")
        .to_string();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let douban_data: DoubanListApiResponse = response.json().await.map_err(|e| {
        format!("JSON parse error (ce={content_encoding}, ct={content_type}): {e}")
    })?;

    // 正则提取年份
    let year_re = Regex::new(r"(\d{4})").unwrap();

    let raw_items = douban_data.subjects.unwrap_or_default();

    // 显式标注类型 Vec<DoubanItem>
    let list: Vec<DoubanItem> = raw_items
        .into_iter()
        .map(|item| {
            let year = year_re
                .captures(&item.card_subtitle)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            DoubanItem {
                id: item.id,
                title: item.title,
                poster: item.cover,
                rate: item.rate,
                year,
            }
        })
        .collect();

    Ok(DoubanResult {
        code: 200,
        message: "获取成功".to_string(),
        list,
    })
}

#[tauri::command]
pub async fn get_douban_recommends(params: DoubanRecommendsParams) -> Result<DoubanResult, String> {
    let proxy_config = get_douban_proxy_config(None, None, None, None);

    match proxy_config.proxy_type {
        DoubanProxyType::CorsProxyZwei => fetch_douban_recommends(
            params,
            "https://ciao-cors.is-an.org/".to_string(),
            false,
            false,
        )
        .await
        .map_err(|e| e.to_string()),
        DoubanProxyType::CmliussssCdnTencent => {
            fetch_douban_recommends(params, "".to_string(), true, false)
                .await
                .map_err(|e| e.to_string())
        }
        DoubanProxyType::CmliussssCdnAli => {
            fetch_douban_recommends(params, "".to_string(), false, true)
                .await
                .map_err(|e| e.to_string())
        }
        DoubanProxyType::CorsAnywhere => fetch_douban_recommends(
            params,
            "https://cors-anywhere.com/".to_string(),
            false,
            false,
        )
        .await
        .map_err(|e| e.to_string()),
        DoubanProxyType::Custom => {
            fetch_douban_recommends(params, proxy_config.proxy_url, false, false)
                .await
                .map_err(|e| e.to_string())
        }
        DoubanProxyType::Direct => {
            // 这里你可以实现直接请求本地API，或者调用fetch_douban_recommends不带代理的版本
            // 假设直接请求本地API可以用 reqwest 请求你的服务接口
            let url = format!(
                "/api/douban/recommends?kind={}&limit={}&start={}&category={}&format={}&region={}&year={}&platform={}&sort={}&label={}",
                params.kind(),
                params.page_limit(),
                params.page_start(),
                params.category,
                params.format,
                params.region,
                params.year,
                params.platform,
                params.sort,
                params.label
            );

            let client = reqwest::Client::new();
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e.to_string()))?;

            if !resp.status().is_success() {
                return Err(format!("HTTP error! Status: {}", resp.status()));
            }

            let result = resp
                .json::<DoubanResult>()
                .await
                .map_err(|e| format!("JSON parse error: {}", e.to_string()))?;
            Ok(result)
        }
    }
}

fn resolve_douban_page_mode(request: &DoubanPageRequest) -> DoubanPageMode {
    if request.request_type == "custom" {
        return DoubanPageMode::Custom;
    }

    if request.request_type == "anime" {
        if request.primary_selection == "\u{6bcf}\u{65e5}\u{653e}\u{9001}" {
            return DoubanPageMode::AnimeDaily;
        }
        return DoubanPageMode::Recommends;
    }

    if (request.request_type == "tv" || request.request_type == "show")
        && request.primary_selection == "全部"
    {
        return DoubanPageMode::Recommends;
    }

    DoubanPageMode::Categories
}

fn default_multi_level_selection() -> HashMap<String, String> {
    let mut selection = HashMap::new();
    selection.insert("type".to_string(), "all".to_string());
    selection.insert("region".to_string(), "all".to_string());
    selection.insert("year".to_string(), "all".to_string());
    selection.insert("platform".to_string(), "all".to_string());
    selection.insert("label".to_string(), "all".to_string());
    selection.insert("sort".to_string(), "T".to_string());
    selection
}

fn resolve_douban_defaults(
    request_type: &str,
    custom_categories: Option<&[DoubanCustomCategory]>,
    fallback_secondary: Option<&str>,
) -> DoubanDefaultsResponse {
    let multi_level_selection = default_multi_level_selection();

    if request_type == "custom" {
        let categories = custom_categories.unwrap_or(&[]);
        if categories.is_empty() {
            return DoubanDefaultsResponse {
                primary_selection: String::new(),
                secondary_selection: fallback_secondary.unwrap_or("").to_string(),
                multi_level_selection,
                cache_enabled: false,
                require_secondary: false,
            };
        }

        let mut types = Vec::new();
        for category in categories {
            if !types.contains(&category.category_type.as_str()) {
                types.push(category.category_type.as_str());
            }
        }

        let selected_type = if types.iter().any(|t| *t == "movie") {
            "movie"
        } else {
            types.first().copied().unwrap_or("")
        };

        let secondary_selection = categories
            .iter()
            .find(|cat| cat.category_type == selected_type)
            .map(|cat| cat.query.clone())
            .unwrap_or_else(|| fallback_secondary.unwrap_or("").to_string());

        return DoubanDefaultsResponse {
            primary_selection: selected_type.to_string(),
            secondary_selection,
            multi_level_selection,
            cache_enabled: false,
            require_secondary: false,
        };
    }

    match request_type {
        "movie" => DoubanDefaultsResponse {
            primary_selection: "\u{70ed}\u{95e8}".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "tv" => DoubanDefaultsResponse {
            primary_selection: "\u{6700}\u{8fd1}\u{70ed}\u{95e8}".to_string(),
            secondary_selection: "tv".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "show" => DoubanDefaultsResponse {
            primary_selection: "\u{6700}\u{8fd1}\u{70ed}\u{95e8}".to_string(),
            secondary_selection: "show".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "anime" => DoubanDefaultsResponse {
            primary_selection: "\u{6bcf}\u{65e5}\u{653e}\u{9001}".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: false,
        },
        _ => DoubanDefaultsResponse {
            primary_selection: String::new(),
            secondary_selection: "全部".to_string(),
            multi_level_selection,
            cache_enabled: false,
            require_secondary: true,
        },
    }
}

fn current_weekday_en() -> &'static str {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0);
    let index = ((days + 4) % 7) as usize;
    WEEKDAYS[index]
}

fn resolve_selected_weekday(selected: Option<&str>) -> String {
    selected
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| current_weekday_en().to_string())
}

fn extract_multi_value(
    selection: Option<&HashMap<String, String>>,
    key: &str,
) -> String {
    selection
        .and_then(|map| map.get(key))
        .cloned()
        .unwrap_or_default()
}

fn ensure_success(result: DoubanResult) -> Result<Vec<DoubanItem>, String> {
    if result.code == 200 {
        Ok(result.list)
    } else {
        Err(result.message)
    }
}

fn is_body_decode_error(message: &str) -> bool {
    message
        .to_ascii_lowercase()
        .contains("decoding response body")
}

fn build_recommends_fallback_list_params(
    request: &DoubanPageRequest,
    page_limit: i32,
    page_start: i32,
) -> DoubanListParams {
    let request_type = request.request_type.as_str();
    let content_type = if request_type == "movie" {
        "movie"
    } else {
        "tv"
    };

    let secondary = request.secondary_selection.trim();
    let is_generic_secondary = secondary.is_empty()
        || secondary == "\u{5168}\u{90e8}"
        || secondary.eq_ignore_ascii_case("tv")
        || secondary.eq_ignore_ascii_case("show");

    let tag = if is_generic_secondary {
        if request_type == "anime" {
            "\u{52a8}\u{753b}".to_string()
        } else {
            "\u{70ed}\u{95e8}".to_string()
        }
    } else {
        secondary.to_string()
    };

    DoubanListParams {
        tag,
        type_: content_type.to_string(),
        page_limit: Some(page_limit),
        page_start: Some(page_start),
    }
}

#[derive(Debug, Serialize)]
struct DoubanCacheKey {
    request_type: String,
    primary_selection: String,
    secondary_selection: String,
    multi_level_selection: Option<BTreeMap<String, String>>,
    selected_weekday: Option<String>,
    page: i32,
    page_limit: i32,
}

fn douban_cache_key(request: &DoubanPageRequest, page_limit: i32, page: i32) -> String {
    let selection = request.multi_level_selection.as_ref().map(|map| {
        map.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<String, String>>()
    });

    let key = DoubanCacheKey {
        request_type: request.request_type.clone(),
        primary_selection: request.primary_selection.clone(),
        secondary_selection: request.secondary_selection.clone(),
        multi_level_selection: selection,
        selected_weekday: request.selected_weekday.clone(),
        page,
        page_limit,
    };

    serde_json::to_string(&key)
        .map(|serialized| format!("douban:{}", serialized))
        .unwrap_or_else(|_| format!("douban:{}:{}:{}", request.request_type, page_limit, page))
}

fn should_cache_douban_request(request: &DoubanPageRequest) -> bool {
    if request.request_type == "custom" || request.request_type == "anime" {
        return false;
    }

    let defaults = resolve_douban_defaults(&request.request_type, None, None);
    if !defaults.cache_enabled {
        return false;
    }

    if request.primary_selection != defaults.primary_selection {
        return false;
    }

    if defaults.require_secondary && request.secondary_selection != defaults.secondary_selection {
        return false;
    }

    true
}

fn build_bangumi_daily_list(
    data: Vec<crate::commands::bangumi::BangumiCalendarData>,
    weekday: &str,
) -> Result<Vec<DoubanItem>, String> {
    let day = data.into_iter().find(|item| {
        item.weekday
            .as_ref()
            .map(|w| w.en.as_str() == weekday)
            .unwrap_or(false)
    });

    let items = match day {
        Some(day) => day.items.unwrap_or_default(),
        None => return Err("\u{6ca1}\u{6709}\u{627e}\u{5230}\u{5bf9}\u{5e94}\u{7684}\u{65e5}\u{671f}".to_string()),
    };

    let list = items
        .into_iter()
        .filter(|item| item.id != 0)
        .map(|item| {
            let title = if !item.name_cn.is_empty() {
                item.name_cn
            } else {
                item.name
            };
            let poster = item
                .images
                .as_ref()
                .and_then(|images| {
                    images
                        .large
                        .clone()
                        .or_else(|| images.common.clone())
                        .or_else(|| images.medium.clone())
                        .or_else(|| images.small.clone())
                        .or_else(|| images.grid.clone())
                })
                .unwrap_or_else(|| "/logo.png".to_string());
            let rate = item
                .rating
                .as_ref()
                .and_then(|rating| rating.score)
                .map(|score| format!("{:.1}", score))
                .unwrap_or_default();
            let year = item
                .air_date
                .as_deref()
                .and_then(|value| value.split('-').next())
                .unwrap_or("")
                .to_string();

            DoubanItem {
                id: item.id.to_string(),
                title,
                poster,
                rate,
                year,
            }
        })
        .collect();

    Ok(list)
}

fn build_recommends_params(
    request: &DoubanPageRequest,
    page_limit: i32,
    page_start: i32,
) -> DoubanRecommendsParams {
    let selection = request.multi_level_selection.as_ref();
    let region = extract_multi_value(selection, "region");
    let year = extract_multi_value(selection, "year");
    let platform = extract_multi_value(selection, "platform");
    let sort = extract_multi_value(selection, "sort");
    let label = extract_multi_value(selection, "label");
    let category = if request.request_type == "anime" {
        "动画".to_string()
    } else {
        extract_multi_value(selection, "type")
    };

    let (kind, format) = if request.request_type == "anime" {
        if request.primary_selection == "\u{70ed}\u{95e8}" {
            (Kind::Tv, "\u{7535}\u{89c6}\u{5267}".to_string())
        } else {
            (Kind::Movie, String::new())
        }
    } else if request.request_type == "show" {
        (Kind::Tv, "综艺".to_string())
    } else if request.request_type == "tv" {
        (Kind::Tv, "\u{7535}\u{89c6}\u{5267}".to_string())
    } else {
        (Kind::Movie, String::new())
    };

    DoubanRecommendsParams {
        kind,
        page_limit: Some(page_limit),
        page_start: Some(page_start),
        category,
        format,
        label,
        region,
        year,
        platform,
        sort,
    }
}

fn build_categories_params(
    request: &DoubanPageRequest,
    page_limit: i32,
    page_start: i32,
) -> DoubanCategoriesParams {
    if request.request_type == "tv" || request.request_type == "show" {
        DoubanCategoriesParams::new(
            Kind::Tv,
            request.request_type.clone(),
            request.secondary_selection.clone(),
            Some(page_limit),
            Some(page_start),
        )
    } else {
        let kind = if request.request_type == "movie" {
            Kind::Movie
        } else {
            Kind::Tv
        };
        DoubanCategoriesParams::new(
            kind,
            request.primary_selection.clone(),
            request.secondary_selection.clone(),
            Some(page_limit),
            Some(page_start),
        )
    }
}

#[tauri::command]
pub fn get_douban_defaults(request: DoubanDefaultsRequest) -> DoubanDefaultsResponse {
    resolve_douban_defaults(
        &request.request_type,
        request.custom_categories.as_deref(),
        request.fallback_secondary.as_deref(),
    )
}

async fn get_douban_page_data_cached(
    request: DoubanPageRequest,
    cache: &PageCacheManager,
) -> Result<DoubanPageResponse, String> {
    let page_limit = request.page_limit.unwrap_or(25);
    let page = request.page.unwrap_or(0).max(0);
    let page_start = page.saturating_mul(page_limit);

    let cache_key = if should_cache_douban_request(&request) {
        Some(douban_cache_key(&request, page_limit, page))
    } else {
        None
    };

    if let Some(key) = cache_key.as_ref() {
        if let Ok(Some(cached)) = cache.get(key) {
            if let Ok(parsed) = serde_json::from_str::<DoubanPageResponse>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let mode = resolve_douban_page_mode(&request);
    let list = match mode {
        DoubanPageMode::Custom => {
            let params = DoubanListParams {
                tag: request.secondary_selection.clone(),
                type_: request.primary_selection.clone(),
                page_limit: Some(page_limit),
                page_start: Some(page_start),
            };
            ensure_success(get_douban_list(params).await?)?
        }
        DoubanPageMode::AnimeDaily => {
            if page > 0 {
                Vec::new()
            } else {
                let weekday = resolve_selected_weekday(request.selected_weekday.as_deref());
                let data = get_bangumi_calendar_data().await?;
                build_bangumi_daily_list(data, &weekday)?
            }
        }
        DoubanPageMode::Recommends => {
            let params = build_recommends_params(&request, page_limit, page_start);
            match get_douban_recommends(params).await {
                Ok(result) => ensure_success(result)?,
                Err(err) => {
                    if !is_body_decode_error(&err) {
                        return Err(err);
                    }

                    let fallback_params =
                        build_recommends_fallback_list_params(&request, page_limit, page_start);
                    ensure_success(get_douban_list(fallback_params).await?)
                        .map_err(|fallback_err| format!("{err}; fallback failed: {fallback_err}"))?
                }
            }
        }
        DoubanPageMode::Categories => {
            let params = build_categories_params(&request, page_limit, page_start);
            ensure_success(get_douban_categories(params).await?)?
        }
    };

    let has_more = list.len() == page_limit as usize;
    let response = DoubanPageResponse { list, has_more };

    if let Some(key) = cache_key {
        if let Ok(serialized) = serde_json::to_string(&response) {
            let _ = cache.set(&key, &serialized);
        }
    }

    Ok(response)
}

#[tauri::command]
pub async fn get_douban_page_data(
    request: DoubanPageRequest,
    cache: tauri::State<'_, PageCacheManager>,
) -> Result<DoubanPageResponse, String> {
    get_douban_page_data_cached(request, &cache).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::page_cache::PageCacheManager;
    use rusqlite::Connection;

    fn setup_page_cache() -> PageCacheManager {
        let conn = Connection::open_in_memory().expect("open cache db");
        let cache = PageCacheManager::new(conn);
        cache.init_table().expect("init cache table");
        cache
    }

    #[test]
    fn resolve_douban_page_mode_custom() {
        let request = DoubanPageRequest {
            request_type: "custom".to_string(),
            primary_selection: "movie".to_string(),
            secondary_selection: "tag".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert_eq!(resolve_douban_page_mode(&request), DoubanPageMode::Custom);
    }

    #[test]
    fn resolve_douban_page_mode_anime_daily() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: "\u{6bcf}\u{65e5}\u{653e}\u{9001}".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert_eq!(
            resolve_douban_page_mode(&request),
            DoubanPageMode::AnimeDaily
        );
    }

    #[test]
    fn resolve_douban_page_mode_recommends() {
        let request = DoubanPageRequest {
            request_type: "show".to_string(),
            primary_selection: "全部".to_string(),
            secondary_selection: "show".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert_eq!(
            resolve_douban_page_mode(&request),
            DoubanPageMode::Recommends
        );
    }

    #[test]
    fn resolve_douban_page_mode_movie_all_is_categories() {
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "全部".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert_eq!(resolve_douban_page_mode(&request), DoubanPageMode::Categories);
    }

    #[test]
    fn build_recommends_params_for_show() {
        let request = DoubanPageRequest {
            request_type: "show".to_string(),
            primary_selection: "全部".to_string(),
            secondary_selection: "show".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };
        let params = build_recommends_params(&request, 25, 0);

        assert_eq!(params.kind, Kind::Tv);
        assert_eq!(params.format, "综艺");
    }

    #[test]
    fn resolve_douban_defaults_movie() {
        let defaults = resolve_douban_defaults("movie", None, None);
        assert_eq!(defaults.primary_selection, "\u{70ed}\u{95e8}");
        assert_eq!(defaults.secondary_selection, "全部");
        assert!(defaults.cache_enabled);
        assert!(defaults.require_secondary);
    }

    #[test]
    fn resolve_douban_defaults_anime() {
        let defaults = resolve_douban_defaults("anime", None, None);
        assert_eq!(defaults.primary_selection, "\u{6bcf}\u{65e5}\u{653e}\u{9001}");
        assert_eq!(defaults.secondary_selection, "全部");
        assert!(defaults.cache_enabled);
        assert!(!defaults.require_secondary);
    }

    #[test]
    fn resolve_douban_defaults_custom_prefers_movie() {
        let categories = vec![
            DoubanCustomCategory {
                name: None,
                category_type: "tv".to_string(),
                query: "q1".to_string(),
                disabled: None,
            },
            DoubanCustomCategory {
                name: None,
                category_type: "movie".to_string(),
                query: "q2".to_string(),
                disabled: None,
            },
        ];

        let defaults = resolve_douban_defaults("custom", Some(&categories), Some("fallback"));
        assert_eq!(defaults.primary_selection, "movie");
        assert_eq!(defaults.secondary_selection, "q2");
        assert!(!defaults.cache_enabled);
    }

    #[test]
    fn resolve_selected_weekday_prefers_trimmed_value() {
        let resolved = resolve_selected_weekday(Some("  Wed  "));
        assert_eq!(resolved, "Wed");
    }

    #[test]
    fn resolve_selected_weekday_falls_back_on_empty() {
        let fallback = current_weekday_en().to_string();
        let resolved = resolve_selected_weekday(Some("   "));
        assert_eq!(resolved, fallback);
    }

    #[test]
    fn build_recommends_fallback_list_params_movie_all() {
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "\u{5168}\u{90e8}".to_string(),
            secondary_selection: "\u{5168}\u{90e8}".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let params = build_recommends_fallback_list_params(&request, 25, 0);
        assert_eq!(params.type_, "movie");
        assert_eq!(params.tag, "\u{70ed}\u{95e8}");
    }

    #[test]
    fn build_recommends_fallback_list_params_anime_defaults_to_animation_tag() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: "\u{5168}\u{90e8}".to_string(),
            secondary_selection: "tv".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let params = build_recommends_fallback_list_params(&request, 25, 0);
        assert_eq!(params.type_, "tv");
        assert_eq!(params.tag, "\u{52a8}\u{753b}");
    }

    #[test]
    fn is_body_decode_error_matches_reqwest_text() {
        assert!(is_body_decode_error("JSON parse error: error decoding response body"));
        assert!(!is_body_decode_error("network timeout"));
    }

    #[tokio::test]
    async fn get_douban_page_data_uses_cache_when_available() {
        let cache = setup_page_cache();
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "热门".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let cached = DoubanPageResponse {
            list: vec![DoubanItem {
                id: "1".to_string(),
                title: "cached".to_string(),
                poster: "p".to_string(),
                rate: "9.0".to_string(),
                year: "2024".to_string(),
            }],
            has_more: false,
        };

        let key = douban_cache_key(&request, 25, 0);
        let payload = serde_json::to_string(&cached).expect("serialize cached data");
        cache.set(&key, &payload).expect("seed cache");

        let data = get_douban_page_data_cached(request, &cache)
            .await
            .expect("get douban data");

        assert_eq!(data.list.len(), 1);
        assert_eq!(data.list[0].title, "cached");
    }
}
