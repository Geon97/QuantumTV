use crate::commands::bangumi::get_bangumi_calendar_data;
use crate::commands::recommendation::infer_category;
use crate::commands::video::{parse_source_categories, SourceCategoryItem};
use crate::db::db_client::Db;
use crate::db::page_cache::PageCacheManager;
use crate::storage::StorageManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DoubanSourceCategory {
    pub type_id: String,
    pub type_name: String,
    pub type_pid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubanPageStateRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub source_key: Option<String>,
    pub primary_selection: Option<String>,
    pub secondary_selection: Option<String>,
    pub multi_level_selection: Option<HashMap<String, String>>,
    pub selected_weekday: Option<String>,
    pub custom_categories: Option<Vec<DoubanCustomCategory>>,
    pub source_category_type_id: Option<String>,
    pub page_limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubanPageStateResponse {
    #[serde(rename = "type")]
    pub request_type: String,
    pub source_key: String,
    pub primary_selection: String,
    pub secondary_selection: String,
    pub multi_level_selection: HashMap<String, String>,
    pub selected_weekday: String,
    pub source_categories: Vec<DoubanSourceCategory>,
    pub selected_source_category_id: Option<String>,
    pub list: Vec<DoubanItem>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMoreDoubanPageRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub source_key: Option<String>,
    pub primary_selection: String,
    pub secondary_selection: String,
    pub multi_level_selection: Option<HashMap<String, String>>,
    pub selected_weekday: Option<String>,
    pub source_category_type_id: Option<String>,
    pub page: i32,
    pub page_limit: Option<i32>,
}

#[derive(Debug, Clone)]
struct ResolvedDoubanPageStateRequest {
    request_type: String,
    source_key: String,
    primary_selection: String,
    secondary_selection: String,
    multi_level_selection: HashMap<String, String>,
    selected_weekday: String,
    source_category_type_id: Option<String>,
    page_limit: i32,
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

const ANIME_DAILY_SELECTION: &str = "每日放送";
const ANIME_SERIES_SELECTION: &str = "番剧";
const ANIME_THEATRICAL_SELECTION: &str = "剧场版";
const ANIME_LEGACY_SERIES_SELECTION: &str = "热门";
const ALL_SELECTION: &str = "全部";

fn is_anime_daily_selection(value: &str) -> bool {
    value.trim() == ANIME_DAILY_SELECTION
}

fn anime_selection_prefers_tv(value: &str) -> bool {
    let normalized = value.trim();
    normalized == ANIME_SERIES_SELECTION
        || normalized == ANIME_LEGACY_SERIES_SELECTION
        || normalized == ALL_SELECTION
        || normalized.eq_ignore_ascii_case("tv")
}

fn is_anime_theatrical_selection(value: &str) -> bool {
    let normalized = value.trim();
    normalized == ANIME_THEATRICAL_SELECTION || normalized.eq_ignore_ascii_case("movie")
}

fn should_prefer_list_api_for_recommends(request: &DoubanPageRequest) -> bool {
    request.request_type == "anime" && is_anime_theatrical_selection(&request.primary_selection)
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
        .timeout(std::time::Duration::from_secs(30)) // 30秒超时        .connect_timeout(std::time::Duration::from_secs(10)) // 10绉掕繛鎺ヨ秴鏃?        .pool_max_idle_per_host(10) // 杩炴帴姹犱紭鍖?        .pool_idle_timeout(std::time::Duration::from_secs(90))
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
    let douban_data: DoubanRecommendApiResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error (ce={content_encoding}, ct={content_type}): {e}"))?;
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
    let douban_data: DoubanListApiResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error (ce={content_encoding}, ct={content_type}): {e}"))?;

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
        if is_anime_daily_selection(&request.primary_selection) {
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
            primary_selection: "热门".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "tv" => DoubanDefaultsResponse {
            primary_selection: "最近热门".to_string(),
            secondary_selection: "tv".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "show" => DoubanDefaultsResponse {
            primary_selection: "最近热门".to_string(),
            secondary_selection: "show".to_string(),
            multi_level_selection,
            cache_enabled: true,
            require_secondary: true,
        },
        "anime" => DoubanDefaultsResponse {
            primary_selection: ANIME_DAILY_SELECTION.to_string(),
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

fn extract_multi_value(selection: Option<&HashMap<String, String>>, key: &str) -> String {
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

fn is_retryable_douban_error(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    msg.contains("decoding response body")
        || msg.contains("json parse error")
        || msg.contains("request failed")
        || msg.contains("network request failed")
}

fn build_recommends_fallback_list_params(
    request: &DoubanPageRequest,
    page_limit: i32,
    page_start: i32,
) -> DoubanListParams {
    let request_type = request.request_type.as_str();
    let content_type = if request_type == "movie" {
        "movie"
    } else if request_type == "anime" {
        if anime_selection_prefers_tv(&request.primary_selection) {
            "tv"
        } else {
            "movie"
        }
    } else {
        "tv"
    };

    let secondary = request.secondary_selection.trim();
    let is_generic_secondary = secondary.is_empty()
        || secondary == "全部"
        || secondary.eq_ignore_ascii_case("tv")
        || secondary.eq_ignore_ascii_case("show");

    let tag = if is_generic_secondary {
        if request_type == "anime" {
            "动画".to_string()
        } else {
            "热门".to_string()
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

fn is_all_primary_selection(value: &str) -> bool {
    value.trim() == "全部"
}

fn is_all_secondary_selection(request_type: &str, value: &str) -> bool {
    let normalized = value.trim();
    if normalized.is_empty() {
        return false;
    }

    if normalized == "全部" || normalized.eq_ignore_ascii_case("all") {
        return true;
    }

    if request_type == "tv" {
        return normalized.eq_ignore_ascii_case("tv");
    }

    if request_type == "show" {
        return normalized.eq_ignore_ascii_case("show");
    }

    false
}

fn should_cache_douban_request(request: &DoubanPageRequest) -> bool {
    if request.request_type == "custom" {
        return false;
    }

    if request.request_type == "anime" {
        // 每日放送来自 Bangumi 日历，实时性更强，不缓存；
        // 番剧/剧场版走豆瓣接口，开启缓存提升重复进入速度。
        return !is_anime_daily_selection(&request.primary_selection);
    }

    let defaults = resolve_douban_defaults(&request.request_type, None, None);
    if !defaults.cache_enabled {
        return false;
    }

    if request.primary_selection != defaults.primary_selection {
        if is_all_primary_selection(&request.primary_selection) {
            return true;
        }
    }

    if defaults.require_secondary
        && request.secondary_selection != defaults.secondary_selection
        && !is_all_secondary_selection(&request.request_type, &request.secondary_selection)
    {
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
        None => return Err("没有找到对应的日期".to_string()),
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
        if anime_selection_prefers_tv(&request.primary_selection) {
            (Kind::Tv, "电视剧".to_string())
        } else if is_anime_theatrical_selection(&request.primary_selection) {
            (Kind::Movie, String::new())
        } else {
            (Kind::Movie, String::new())
        }
    } else if request.request_type == "show" {
        (Kind::Tv, "综艺".to_string())
    } else if request.request_type == "tv" {
        (Kind::Tv, "电视剧".to_string())
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
    let mut attempts = 0;
    let max_attempts = if should_prefer_list_api_for_recommends(&request) {
        1
    } else if request.request_type == "anime" {
        2
    } else {
        3
    };
    let list = loop {
        attempts += 1;
        let result: Result<Vec<DoubanItem>, String> = match mode {
            DoubanPageMode::Custom => {
                let params = DoubanListParams {
                    tag: request.secondary_selection.clone(),
                    type_: request.primary_selection.clone(),
                    page_limit: Some(page_limit),
                    page_start: Some(page_start),
                };
                ensure_success(get_douban_list(params).await?)
            }
            DoubanPageMode::AnimeDaily => {
                if page > 0 {
                    Ok(Vec::new())
                } else {
                    let weekday = resolve_selected_weekday(request.selected_weekday.as_deref());
                    let data = get_bangumi_calendar_data().await?;
                    build_bangumi_daily_list(data, &weekday)
                }
            }
            DoubanPageMode::Recommends => {
                if should_prefer_list_api_for_recommends(&request) {
                    let primary_params =
                        build_recommends_fallback_list_params(&request, page_limit, page_start);
                    match get_douban_list(primary_params).await {
                        Ok(result) => ensure_success(result),
                        Err(err) => {
                            if !is_body_decode_error(&err) {
                                Err(err)
                            } else {
                                let fallback_params =
                                    build_recommends_params(&request, page_limit, page_start);
                                ensure_success(get_douban_recommends(fallback_params).await?)
                                    .map_err(|fallback_err| {
                                        format!("{err}; fallback failed: {fallback_err}")
                                    })
                            }
                        }
                    }
                } else {
                    let params = build_recommends_params(&request, page_limit, page_start);
                    match get_douban_recommends(params).await {
                        Ok(result) => ensure_success(result),
                        Err(err) => {
                            if !is_body_decode_error(&err) {
                                Err(err)
                            } else {
                                let fallback_params = build_recommends_fallback_list_params(
                                    &request, page_limit, page_start,
                                );
                                ensure_success(get_douban_list(fallback_params).await?).map_err(
                                    |fallback_err| {
                                        format!("{err}; fallback failed: {fallback_err}")
                                    },
                                )
                            }
                        }
                    }
                }
            }
            DoubanPageMode::Categories => {
                let params = build_categories_params(&request, page_limit, page_start);
                ensure_success(get_douban_categories(params).await?)
            }
        };

        match result {
            Ok(items) => break items,
            Err(err) => {
                if attempts >= max_attempts || !is_retryable_douban_error(&err) {
                    return Err(err);
                }
                tokio::time::sleep(std::time::Duration::from_millis(120 * attempts as u64)).await;
            }
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

/// 将豆瓣页面数据同步到 content_pool 和 image_cache
/// request_type: movie / tv / anime / show
fn sync_douban_items_to_pool(db: &Db, items: &[DoubanItem], request_type: &str) {
    if items.is_empty() {
        return;
    }

    let category = match request_type {
        "movie" => "Movie",
        "tv" => "TvSeries",
        "anime" => "Anime",
        "show" => "Variety",
        _ => "",
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let _ = db.with_conn(|conn| {
        let mut pool_stmt = conn.prepare(
            "INSERT OR IGNORE INTO content_pool
             (title, source_name, year, cover, category, rating, description, tags, popularity_score, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', '', ?7, ?8, ?9)"
        )?;

        let mut cache_stmt = conn.prepare(
            "UPDATE image_cache
             SET title = ?1, source_name = ?2, year = ?3, category = ?4, rating = ?5
             WHERE url = ?6 AND (title IS NULL OR title = '')"
        )?;

        for item in items {
            if item.title.is_empty() {
                continue;
            }

            let rating: f64 = item.rate.parse().unwrap_or(0.0);
            // 如果 category 为空，尝试智能推断
            let final_category = if category.is_empty() {
                infer_category(&item.title, "豆瓣").unwrap_or_default()
            } else {
                category.to_string()
            };

            // 写入 content_pool
            let _ = pool_stmt.execute(rusqlite::params![
                item.title,
                "豆瓣",
                item.year,
                item.poster,
                final_category,
                rating,
                rating * 10.0,
                now,
                now,
            ]);

            // 回填 image_cache
            if !item.poster.is_empty() {
                let _ = cache_stmt.execute(rusqlite::params![
                    item.title,
                    "豆瓣",
                    item.year,
                    final_category,
                    rating,
                    item.poster,
                ]);
            }
        }

        Ok::<(), rusqlite::Error>(())
    });
}

#[tauri::command]
pub async fn get_douban_page_data(
    request: DoubanPageRequest,
    cache: tauri::State<'_, PageCacheManager>,
    db: tauri::State<'_, Db>,
) -> Result<DoubanPageResponse, String> {
    let request_type = request.request_type.clone();
    let response = get_douban_page_data_cached(request, &cache).await?;

    // 后台同步到 content_pool 和 image_cache
    sync_douban_items_to_pool(&db, &response.list, &request_type);

    Ok(response)
}

const SOURCE_CATEGORY_KEYWORDS: [(&str, &[&str]); 4] = [
    ("movie", &["电影", "影片", "院线", "4k", "蓝光"]),
    ("tv", &["电视剧", "剧集", "美剧", "韩剧", "日剧", "港剧"]),
    ("anime", &["动漫", "动画", "番剧", "漫画"]),
    ("show", &["综艺", "真人秀", "脱口秀", "晚会", "纪录片"]),
];
const SOURCE_GENERIC_FALLBACK_KEYWORDS: [&str; 3] = ["影", "剧", "漫"];
const SOURCE_CATEGORY_FALLBACK_LIMIT: usize = 15;
#[allow(dead_code)]
const SOURCE_PAGE_SIZE_THRESHOLD: usize = 20;

fn trim_to_option(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn normalize_source_key(source_key: Option<&str>) -> String {
    trim_to_option(source_key).unwrap_or_else(|| "auto".to_string())
}

fn category_keywords_for_request_type(request_type: &str) -> &'static [&'static str] {
    SOURCE_CATEGORY_KEYWORDS
        .iter()
        .find_map(|(key, keywords)| (*key == request_type).then_some(*keywords))
        .unwrap_or(&[])
}

fn matches_category_keywords(category_name: &str, keywords: &[&str]) -> bool {
    let lowered = category_name.to_lowercase();
    keywords
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_lowercase()))
}

fn stringify_source_category_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        _ => None,
    }
}

fn normalize_source_category(category: &SourceCategoryItem) -> Option<DoubanSourceCategory> {
    Some(DoubanSourceCategory {
        type_id: stringify_source_category_value(&category.type_id)?,
        type_name: category.type_name.clone(),
        type_pid: category
            .type_pid
            .as_ref()
            .and_then(stringify_source_category_value),
    })
}

fn filter_source_categories_by_request_type(
    request_type: &str,
    categories: &[SourceCategoryItem],
) -> Vec<DoubanSourceCategory> {
    let normalized: Vec<DoubanSourceCategory> = categories
        .iter()
        .filter_map(normalize_source_category)
        .collect();

    if normalized.is_empty() {
        return Vec::new();
    }

    let keywords = category_keywords_for_request_type(request_type);

    // 精确匹配
    let mut matched: Vec<DoubanSourceCategory> = normalized
        .iter()
        .filter(|c| matches_category_keywords(&c.type_name, keywords))
        .cloned()
        .collect();

    // 如果精确匹配为空，使用通用关键词回退
    if matched.is_empty() {
        matched = normalized
            .iter()
            .filter(|c| {
                SOURCE_GENERIC_FALLBACK_KEYWORDS
                    .iter()
                    .any(|kw| c.type_name.contains(kw))
            })
            .cloned()
            .collect();
    }

    // 限制返回数量
    if matched.len() > SOURCE_CATEGORY_FALLBACK_LIMIT {
        matched.truncate(SOURCE_CATEGORY_FALLBACK_LIMIT);
    }

    matched
}

/// 解析并规范化 Douban 页面状态请求
fn resolve_douban_page_state_request(
    request: DoubanPageStateRequest,
) -> ResolvedDoubanPageStateRequest {
    let source_key = normalize_source_key(request.source_key.as_deref());
    let page_limit = request.page_limit.unwrap_or(25);

    // 如果是特定源模式，直接返回
    if source_key != "auto" {
        return ResolvedDoubanPageStateRequest {
            request_type: request.request_type,
            source_key,
            primary_selection: request.primary_selection.unwrap_or_default(),
            secondary_selection: request.secondary_selection.unwrap_or_default(),
            multi_level_selection: request.multi_level_selection.unwrap_or_default(),
            selected_weekday: request.selected_weekday.unwrap_or_default(),
            source_category_type_id: request.source_category_type_id,
            page_limit,
        };
    }

    // 聚合模式：应用默认值
    let defaults = resolve_douban_defaults(&request.request_type, None, None);
    let primary_selection = request
        .primary_selection
        .unwrap_or_else(|| defaults.primary_selection.clone());
    let secondary_selection = request
        .secondary_selection
        .unwrap_or_else(|| defaults.secondary_selection.clone());

    let selected_weekday =
        if request.request_type == "anime" && is_anime_daily_selection(&primary_selection) {
            resolve_selected_weekday(request.selected_weekday.as_deref())
        } else {
            request.selected_weekday.unwrap_or_default()
        };

    let multi_level_selection = request
        .multi_level_selection
        .unwrap_or_else(default_multi_level_selection);

    ResolvedDoubanPageStateRequest {
        request_type: request.request_type,
        source_key,
        primary_selection,
        secondary_selection,
        multi_level_selection,
        selected_weekday,
        source_category_type_id: None,
        page_limit,
    }
}

/// 获取 Douban 页面完整状态（包括数据和源分类）
#[tauri::command]
pub async fn get_douban_page_state(
    request: DoubanPageStateRequest,
    cache: tauri::State<'_, PageCacheManager>,
    storage: tauri::State<'_, StorageManager>,
    db: tauri::State<'_, Db>,
) -> Result<DoubanPageStateResponse, String> {
    let resolved = resolve_douban_page_state_request(request);

    // 如果是特定源模式
    if resolved.source_key != "auto" {
        // 获取源分类
        let data = storage.get_data()?;
        let config = crate::commands::config::get_config_with_db_sources(&storage, &db)?;
        let source = crate::commands::video::resolve_enabled_source(&config, &resolved.source_key)
            .ok_or_else(|| format!("Source not found or disabled: {}", resolved.source_key))?;

        let url = crate::commands::video::source_url(&source.api, "?ac=list");
        let body = crate::commands::video::get_video_client()
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let all_categories = parse_source_categories(&body)?;
        let source_categories =
            filter_source_categories_by_request_type(&resolved.request_type, &all_categories);

        // 确定选中的分类
        let selected_category = if let Some(type_id) = &resolved.source_category_type_id {
            source_categories
                .iter()
                .find(|c| c.type_id == *type_id)
                .cloned()
        } else {
            source_categories.first().cloned()
        };

        // 返回空列表，让前端调用 get_source_videos_by_type 获取实际数据
        return Ok(DoubanPageStateResponse {
            request_type: resolved.request_type,
            source_key: resolved.source_key,
            primary_selection: resolved.primary_selection,
            secondary_selection: resolved.secondary_selection,
            multi_level_selection: resolved.multi_level_selection,
            selected_weekday: resolved.selected_weekday,
            source_categories,
            selected_source_category_id: selected_category.map(|c| c.type_id),
            list: Vec::new(), // 前端需要调用 get_source_videos_by_type 获取数据
            has_more: true,
        });
    }

    // 聚合模式：获取豆瓣数据
    let douban_request = DoubanPageRequest {
        request_type: resolved.request_type.clone(),
        primary_selection: resolved.primary_selection.clone(),
        secondary_selection: resolved.secondary_selection.clone(),
        multi_level_selection: Some(resolved.multi_level_selection.clone()),
        selected_weekday: Some(resolved.selected_weekday.clone()),
        page: Some(0),
        page_limit: Some(resolved.page_limit),
    };

    let douban_response = get_douban_page_data_cached(douban_request, &cache).await?;

    // 后台同步到 content_pool 和 image_cache
    sync_douban_items_to_pool(&db, &douban_response.list, &resolved.request_type);

    Ok(DoubanPageStateResponse {
        request_type: resolved.request_type,
        source_key: resolved.source_key,
        primary_selection: resolved.primary_selection,
        secondary_selection: resolved.secondary_selection,
        multi_level_selection: resolved.multi_level_selection,
        selected_weekday: resolved.selected_weekday,
        source_categories: Vec::new(),
        selected_source_category_id: None,
        list: douban_response.list,
        has_more: douban_response.has_more,
    })
}

/// 加载更多 Douban 页面数据
#[tauri::command]
pub async fn load_more_douban_page(
    request: LoadMoreDoubanPageRequest,
    cache: tauri::State<'_, PageCacheManager>,
    db: tauri::State<'_, Db>,
) -> Result<DoubanPageResponse, String> {
    let source_key = normalize_source_key(request.source_key.as_deref());
    let page_limit = request.page_limit.unwrap_or(25);

    // 如果是特定源模式，返回空列表，让前端调用 get_source_videos_by_type
    if source_key != "auto" {
        return Ok(DoubanPageResponse {
            list: Vec::new(),
            has_more: true,
        });
    }

    // 聚合模式：获取豆瓣数据
    let multi_level_selection = request
        .multi_level_selection
        .unwrap_or_else(default_multi_level_selection);

    let selected_weekday = if request.request_type == "anime"
        && is_anime_daily_selection(&request.primary_selection)
    {
        resolve_selected_weekday(request.selected_weekday.as_deref())
    } else {
        request.selected_weekday.unwrap_or_default()
    };

    let request_type = request.request_type.clone();
    let douban_request = DoubanPageRequest {
        request_type: request.request_type,
        primary_selection: request.primary_selection,
        secondary_selection: request.secondary_selection,
        multi_level_selection: Some(multi_level_selection),
        selected_weekday: Some(selected_weekday),
        page: Some(request.page),
        page_limit: Some(page_limit),
    };

    let response = get_douban_page_data_cached(douban_request, &cache).await?;

    // 后台同步到 content_pool 和 image_cache
    sync_douban_items_to_pool(&db, &response.list, &request_type);

    Ok(response)
}

/// 获取过滤后的源分类（统一的分类过滤规则）
#[tauri::command]
pub async fn get_filtered_source_categories(
    source_key: String,
    content_type: String,
    storage: tauri::State<'_, StorageManager>,
    db: tauri::State<'_, Db>,
) -> Result<Vec<DoubanSourceCategory>, String> {
    if source_key == "auto" {
        return Ok(Vec::new());
    }

    // 获取源分类
    let data = storage.get_data()?;
    let config = crate::commands::config::get_config_with_db_sources(&storage, &db)?;
    let source = crate::commands::video::resolve_enabled_source(&config, &source_key)
        .ok_or_else(|| format!("Source not found or disabled: {}", source_key))?;

    let url = crate::commands::video::source_url(&source.api, "?ac=list");
    let body = crate::commands::video::get_video_client()
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let all_categories = parse_source_categories(&body)?;
    let filtered = filter_source_categories_by_request_type(&content_type, &all_categories);

    Ok(filtered)
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
            primary_selection: ANIME_DAILY_SELECTION.to_string(),
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

        assert_eq!(
            resolve_douban_page_mode(&request),
            DoubanPageMode::Categories
        );
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
    fn resolve_douban_page_mode_anime_non_daily_is_recommends() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_SERIES_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
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
    fn build_recommends_params_for_anime_series_uses_tv_format() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_SERIES_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };
        let params = build_recommends_params(&request, 25, 0);

        assert_eq!(params.kind, Kind::Tv);
        assert_eq!(params.format, "电视剧");
        assert_eq!(params.category, "动画");
    }

    #[test]
    fn build_recommends_params_for_anime_theatrical_uses_movie_format() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_THEATRICAL_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };
        let params = build_recommends_params(&request, 25, 0);

        assert_eq!(params.kind, Kind::Movie);
        assert_eq!(params.format, "");
        assert_eq!(params.category, "动画");
    }

    #[test]
    fn build_recommends_fallback_list_params_anime_theatrical_uses_movie_type() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_THEATRICAL_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let params = build_recommends_fallback_list_params(&request, 25, 0);
        assert_eq!(params.type_, "movie");
        assert_eq!(params.tag, "动画");
    }

    #[test]
    fn should_prefer_list_api_for_recommends_anime_theatrical() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_THEATRICAL_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };
        assert!(should_prefer_list_api_for_recommends(&request));
    }

    #[test]
    fn should_not_prefer_list_api_for_recommends_anime_series() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_SERIES_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };
        assert!(!should_prefer_list_api_for_recommends(&request));
    }

    #[test]
    fn resolve_douban_defaults_movie() {
        let defaults = resolve_douban_defaults("movie", None, None);
        assert_eq!(defaults.primary_selection, "热门");
        assert_eq!(defaults.secondary_selection, "全部");
        assert!(defaults.cache_enabled);
        assert!(defaults.require_secondary);
    }

    #[test]
    fn resolve_douban_defaults_anime() {
        let defaults = resolve_douban_defaults("anime", None, None);
        assert_eq!(defaults.primary_selection, "每日放送");
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
            primary_selection: "全部".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let params = build_recommends_fallback_list_params(&request, 25, 0);
        assert_eq!(params.type_, "movie");
        assert_eq!(params.tag, "热门");
    }

    #[test]
    fn build_recommends_fallback_list_params_anime_defaults_to_animation_tag() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: "全部".to_string(),
            secondary_selection: "tv".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        let params = build_recommends_fallback_list_params(&request, 25, 0);
        assert_eq!(params.type_, "tv");
        assert_eq!(params.tag, "动画");
    }

    #[test]
    fn is_body_decode_error_matches_reqwest_text() {
        assert!(is_body_decode_error(
            "JSON parse error: error decoding response body"
        ));
        assert!(!is_body_decode_error("network timeout"));
    }

    #[test]
    fn is_retryable_douban_error_covers_decode_and_json_parse() {
        assert!(is_retryable_douban_error(
            "JSON parse error (ce=none, ct=application/json; charset=utf-8): error decoding response body"
        ));
        assert!(is_retryable_douban_error("JSON parse error: invalid type"));
        assert!(is_retryable_douban_error("Network request failed: timeout"));
        assert!(!is_retryable_douban_error("HTTP error! Status: 403"));
    }

    #[test]
    fn should_cache_tv_when_primary_is_all() {
        let request = DoubanPageRequest {
            request_type: "tv".to_string(),
            primary_selection: "全部".to_string(),
            secondary_selection: "tv".to_string(),
            multi_level_selection: Some(default_multi_level_selection()),
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert!(should_cache_douban_request(&request));
    }

    #[test]
    fn should_cache_movie_when_secondary_is_all() {
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "最新".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert!(should_cache_douban_request(&request));
    }

    #[test]
    fn should_not_cache_movie_when_secondary_is_not_all_and_not_default() {
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "热门".to_string(),
            secondary_selection: "韩国".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert!(!should_cache_douban_request(&request));
    }

    #[test]
    fn should_cache_anime_non_daily_selection() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: "番剧".to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: Some(default_multi_level_selection()),
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert!(should_cache_douban_request(&request));
    }

    #[test]
    fn should_not_cache_anime_daily_selection() {
        let request = DoubanPageRequest {
            request_type: "anime".to_string(),
            primary_selection: ANIME_DAILY_SELECTION.to_string(),
            secondary_selection: "全部".to_string(),
            multi_level_selection: Some(default_multi_level_selection()),
            selected_weekday: Some("Mon".to_string()),
            page: Some(0),
            page_limit: Some(25),
        };

        assert!(!should_cache_douban_request(&request));
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

    #[test]
    fn test_resolve_douban_page_state_request_movie_defaults() {
        let request = DoubanPageStateRequest {
            request_type: "movie".to_string(),
            source_key: None,
            primary_selection: None,
            secondary_selection: None,
            multi_level_selection: None,
            selected_weekday: None,
            custom_categories: None,
            source_category_type_id: None,
            page_limit: None,
        };

        let resolved = resolve_douban_page_state_request(request);

        assert_eq!(resolved.request_type, "movie");
        assert_eq!(resolved.source_key, "auto");
        assert_eq!(resolved.primary_selection, "热门");
        assert_eq!(resolved.secondary_selection, "全部");
        assert_eq!(resolved.page_limit, 25);
    }

    #[test]
    fn test_resolve_douban_page_state_request_tv_defaults() {
        let request = DoubanPageStateRequest {
            request_type: "tv".to_string(),
            source_key: None,
            primary_selection: None,
            secondary_selection: None,
            multi_level_selection: None,
            selected_weekday: None,
            custom_categories: None,
            source_category_type_id: None,
            page_limit: None,
        };

        let resolved = resolve_douban_page_state_request(request);

        assert_eq!(resolved.request_type, "tv");
        assert_eq!(resolved.source_key, "auto");
        assert_eq!(resolved.primary_selection, "最近热门");
        assert_eq!(resolved.secondary_selection, "tv");
    }

    #[test]
    fn test_resolve_douban_page_state_request_anime_defaults() {
        let request = DoubanPageStateRequest {
            request_type: "anime".to_string(),
            source_key: None,
            primary_selection: None,
            secondary_selection: None,
            multi_level_selection: None,
            selected_weekday: None,
            custom_categories: None,
            source_category_type_id: None,
            page_limit: None,
        };

        let resolved = resolve_douban_page_state_request(request);

        assert_eq!(resolved.request_type, "anime");
        assert_eq!(resolved.primary_selection, "每日放送");
        assert_eq!(resolved.secondary_selection, "全部");
        assert_eq!(resolved.selected_weekday, current_weekday_en());
    }

    #[test]
    fn test_resolve_douban_page_state_request_show_defaults() {
        let request = DoubanPageStateRequest {
            request_type: "show".to_string(),
            source_key: None,
            primary_selection: None,
            secondary_selection: None,
            multi_level_selection: None,
            selected_weekday: None,
            custom_categories: None,
            source_category_type_id: None,
            page_limit: None,
        };

        let resolved = resolve_douban_page_state_request(request);

        assert_eq!(resolved.request_type, "show");
        assert_eq!(resolved.primary_selection, "最近热门");
        assert_eq!(resolved.secondary_selection, "show");
    }

    #[test]
    fn test_filter_source_categories_by_request_type_movie() {
        let categories = vec![
            SourceCategoryItem {
                type_id: serde_json::Value::String("1".to_string()),
                type_name: "电影".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("2".to_string()),
                type_name: "电视剧".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("3".to_string()),
                type_name: "动漫".to_string(),
                type_pid: None,
            },
        ];

        let filtered = filter_source_categories_by_request_type("movie", &categories);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].type_name, "电影");
    }

    #[test]
    fn test_filter_source_categories_by_request_type_tv() {
        let categories = vec![
            SourceCategoryItem {
                type_id: serde_json::Value::String("1".to_string()),
                type_name: "电影".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("2".to_string()),
                type_name: "电视剧".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("3".to_string()),
                type_name: "美剧".to_string(),
                type_pid: None,
            },
        ];

        let filtered = filter_source_categories_by_request_type("tv", &categories);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|c| c.type_name == "电视剧"));
        assert!(filtered.iter().any(|c| c.type_name == "美剧"));
    }

    #[test]
    fn test_filter_source_categories_by_request_type_anime() {
        let categories = vec![
            SourceCategoryItem {
                type_id: serde_json::Value::String("1".to_string()),
                type_name: "电影".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("2".to_string()),
                type_name: "动漫".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("3".to_string()),
                type_name: "动画".to_string(),
                type_pid: None,
            },
        ];

        let filtered = filter_source_categories_by_request_type("anime", &categories);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|c| c.type_name == "动漫"));
        assert!(filtered.iter().any(|c| c.type_name == "动画"));
    }

    #[test]
    fn test_filter_source_categories_fallback_generic() {
        let categories = vec![
            SourceCategoryItem {
                type_id: serde_json::Value::String("1".to_string()),
                type_name: "影视".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("2".to_string()),
                type_name: "剧集".to_string(),
                type_pid: None,
            },
            SourceCategoryItem {
                type_id: serde_json::Value::String("3".to_string()),
                type_name: "漫画".to_string(),
                type_pid: None,
            },
        ];

        let filtered = filter_source_categories_by_request_type("movie", &categories);

        // 应该匹配通用关键词 "影"
        assert!(filtered.len() > 0);
        assert!(filtered.iter().any(|c| c.type_name.contains("影")));
    }

    #[test]
    fn test_filter_source_categories_limit() {
        let mut categories = Vec::new();
        for i in 0..20 {
            categories.push(SourceCategoryItem {
                type_id: serde_json::Value::String(i.to_string()),
                type_name: format!("电影{}", i),
                type_pid: None,
            });
        }

        let filtered = filter_source_categories_by_request_type("movie", &categories);

        // 应该限制在 15 个以内
        assert!(filtered.len() <= 15);
    }

    #[test]
    fn test_matches_category_keywords() {
        assert!(matches_category_keywords("电影", &["电影", "影片"]));
        assert!(matches_category_keywords("4K电影", &["电影", "影片"]));
        assert!(matches_category_keywords("蓝光影片", &["电影", "影片"]));
        assert!(!matches_category_keywords("电视剧", &["电影", "影片"]));
    }

    #[test]
    fn test_normalize_source_key() {
        assert_eq!(normalize_source_key(None), "auto");
        assert_eq!(normalize_source_key(Some("")), "auto");
        assert_eq!(normalize_source_key(Some("  ")), "auto");
        assert_eq!(normalize_source_key(Some("source1")), "source1");
        assert_eq!(normalize_source_key(Some("  source1  ")), "source1");
    }

    #[test]
    fn test_category_keywords_for_request_type() {
        let movie_keywords = category_keywords_for_request_type("movie");
        assert!(movie_keywords.contains(&"电影"));
        assert!(movie_keywords.contains(&"影片"));

        let tv_keywords = category_keywords_for_request_type("tv");
        assert!(tv_keywords.contains(&"电视剧"));
        assert!(tv_keywords.contains(&"剧集"));

        let anime_keywords = category_keywords_for_request_type("anime");
        assert!(anime_keywords.contains(&"动漫"));
        assert!(anime_keywords.contains(&"动画"));

        let show_keywords = category_keywords_for_request_type("show");
        assert!(show_keywords.contains(&"综艺"));
        assert!(show_keywords.contains(&"真人秀"));
    }

    #[test]
    fn kind_serialization_tv() {
        let kind = Kind::Tv;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"tv\"");
        let deserialized: Kind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Kind::Tv);
    }

    #[test]
    fn kind_serialization_movie() {
        let kind = Kind::Movie;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"movie\"");
        let deserialized: Kind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Kind::Movie);
    }

    #[test]
    fn douban_item_creation() {
        let item = DoubanItem {
            id: "12345".to_string(),
            title: "Test Title".to_string(),
            poster: "http://example.com/poster.jpg".to_string(),
            rate: "8.5".to_string(),
            year: "2024".to_string(),
        };

        assert_eq!(item.id, "12345");
        assert_eq!(item.title, "Test Title");
        assert!((item.rate.parse::<f64>().unwrap() - 8.5).abs() < 0.01);
    }

    #[test]
    fn douban_item_serialization() {
        let item = DoubanItem {
            id: "123".to_string(),
            title: "Test".to_string(),
            poster: "url".to_string(),
            rate: "8.0".to_string(),
            year: "2024".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\":\"123\""));
        assert!(json.contains("\"title\":\"Test\""));
    }

    #[test]
    fn douban_result_creation() {
        let items = vec![
            DoubanItem {
                id: "1".to_string(),
                title: "Item 1".to_string(),
                poster: "url1".to_string(),
                rate: "8.0".to_string(),
                year: "2024".to_string(),
            },
            DoubanItem {
                id: "2".to_string(),
                title: "Item 2".to_string(),
                poster: "url2".to_string(),
                rate: "7.5".to_string(),
                year: "2023".to_string(),
            },
        ];

        let result = DoubanResult {
            code: 0,
            message: "success".to_string(),
            list: items,
        };

        assert_eq!(result.code, 0);
        assert_eq!(result.message, "success");
        assert_eq!(result.list.len(), 2);
    }

    #[test]
    fn douban_page_request_creation() {
        let request = DoubanPageRequest {
            request_type: "movie".to_string(),
            primary_selection: "hot".to_string(),
            secondary_selection: "all".to_string(),
            multi_level_selection: None,
            selected_weekday: None,
            page: Some(0),
            page_limit: Some(25),
        };

        assert_eq!(request.request_type, "movie");
        assert_eq!(request.page, Some(0));
        assert_eq!(request.page_limit, Some(25));
    }

    #[test]
    fn douban_page_response_creation() {
        let items = vec![DoubanItem {
            id: "1".to_string(),
            title: "Item".to_string(),
            poster: "url".to_string(),
            rate: "8.0".to_string(),
            year: "2024".to_string(),
        }];

        let response = DoubanPageResponse {
            list: items,
            has_more: true,
        };

        assert_eq!(response.list.len(), 1);
        assert!(response.has_more);
    }

    #[test]
    fn douban_source_category_equality() {
        let cat1 = DoubanSourceCategory {
            type_id: "1".to_string(),
            type_name: "电影".to_string(),
            type_pid: None,
        };

        let cat2 = DoubanSourceCategory {
            type_id: "1".to_string(),
            type_name: "电影".to_string(),
            type_pid: None,
        };

        assert_eq!(cat1, cat2);
    }

    #[test]
    fn douban_source_category_with_parent_id() {
        let cat = DoubanSourceCategory {
            type_id: "2".to_string(),
            type_name: "爱情".to_string(),
            type_pid: Some("1".to_string()),
        };

        assert_eq!(cat.type_id, "2");
        assert_eq!(cat.type_name, "爱情");
        assert_eq!(cat.type_pid, Some("1".to_string()));
    }

    #[test]
    fn category_keywords_variation_coverage() {
        // TV categories
        let tv_keywords = category_keywords_for_request_type("tv");
        assert!(tv_keywords.contains(&"电视剧"));
        assert!(tv_keywords.len() > 2);

        // Anime categories
        let anime_keywords = category_keywords_for_request_type("anime");
        assert!(anime_keywords.contains(&"动漫"));
        assert!(anime_keywords.len() > 2);

        // Movie categories
        let movie_keywords = category_keywords_for_request_type("movie");
        assert!(movie_keywords.contains(&"电影"));
        assert!(movie_keywords.len() > 2);

        // Show categories
        let show_keywords = category_keywords_for_request_type("show");
        assert!(show_keywords.contains(&"综艺"));
        assert!(show_keywords.len() > 2);

        // Unknown type returns empty
        let unknown_keywords = category_keywords_for_request_type("unknown");
        assert!(unknown_keywords.is_empty());
    }

    #[test]
    fn normalize_source_key_comprehensive() {
        // None values
        assert_eq!(normalize_source_key(None), "auto");

        // Empty and whitespace
        assert_eq!(normalize_source_key(Some("")), "auto");
        assert_eq!(normalize_source_key(Some("   ")), "auto");
        assert_eq!(normalize_source_key(Some("\t\n")), "auto");

        // Valid keys
        assert_eq!(normalize_source_key(Some("source1")), "source1");
        assert_eq!(normalize_source_key(Some("  source1")), "source1");
        assert_eq!(normalize_source_key(Some("source1  ")), "source1");
        assert_eq!(normalize_source_key(Some("  source1  ")), "source1");

        // Keys with special characters
        assert_eq!(normalize_source_key(Some("source-1")), "source-1");
        assert_eq!(normalize_source_key(Some("source_1")), "source_1");
    }

    #[test]
    fn matches_category_keywords_comprehensive() {
        // Exact matches
        assert!(matches_category_keywords("电影", &["电影"]));
        assert!(matches_category_keywords("电视剧", &["电视剧"]));

        // Substring matches
        assert!(matches_category_keywords("高清电影", &["电影"]));
        assert!(matches_category_keywords("喜剧电视剧", &["电视剧"]));

        // Multiple keywords
        assert!(matches_category_keywords("电影", &["电视剧", "电影"]));

        // Non-matching
        assert!(!matches_category_keywords("电视剧", &["电影"]));
        assert!(!matches_category_keywords("动漫", &["电视剧"]));
    }

    #[test]
    fn douban_categories_params_creation() {
        let params = DoubanCategoriesParams {
            kind: Kind::Movie,
            category: "冒险".to_string(),
            type_: "热门".to_string(),
            page_limit: Some(20),
            page_start: Some(0),
        };

        assert_eq!(params.kind, Kind::Movie);
        assert_eq!(params.category, "冒险");
        assert_eq!(params.page_limit, Some(20));
    }
}
