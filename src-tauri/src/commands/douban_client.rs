use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use url::form_urlencoded;
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // 不区分大小写
enum Kind {
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
struct DoubanItem {
    id: String,
    title: String,
    poster: String,
    rate: String,
    year: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoubanResult {
    code: i32,
    message: String,
    list: Vec<DoubanItem>,
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
    let client_builder = reqwest::Client::builder();

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url)?)
    } else {
        client_builder
    };

    let client = client_builder.build()?;

    let response = client
        .get(&target)
        .header("User-Agent", "Mozilla/5.0 ...")
        .header("Referer", "https://movie.douban.com/")
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error! Status: {}", response.status()).into());
    }
    let douban_data: DoubanCategoryApiResponse = response.json().await?;
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
    Ok(DoubanResult {
        code: 200,
        message: "获取成功".to_string(),
        list,
    })
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
    println!("target_url: {}", target_url);
    let client_builder = reqwest::Client::builder();

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url)?)
    } else {
        client_builder
    };

    let client = client_builder.build()?;

    let response = client
        .get(&target_url)
        .header("User-Agent", "Mozilla/5.0 ...")
        .header("Referer", "https://movie.douban.com/")
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error! Status: {}", response.status()).into());
    }
    let douban_data: DoubanRecommendApiResponse = response.json().await?;
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
    let client_builder = reqwest::Client::builder();

    let client_builder = if !proxy_url.is_empty() && !use_tencent_cdn && !use_ali_cdn {
        // 只有非 CDN 情况下用代理
        client_builder.proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
    } else {
        client_builder
    };

    let client = client_builder.build().map_err(|e| e.to_string())?;

    let response = client
        .get(&target)
        .header("User-Agent", "Mozilla/5.0 ...")
        .header("Referer", "https://movie.douban.com/")
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error! Status: {}", response.status()).into());
    }

    let douban_data: DoubanListApiResponse = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

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
