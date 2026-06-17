use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Weekday {
    pub en: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rating {
    pub score: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Images {
    pub large: Option<String>,
    pub common: Option<String>,
    pub medium: Option<String>,
    pub small: Option<String>,
    pub grid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Items {
    pub id: i32,
    pub name: String,
    pub name_cn: String,
    pub rating: Option<Rating>,
    pub air_date: Option<String>,
    pub images: Option<Images>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BangumiCalendarData {
    pub weekday: Option<Weekday>,
    pub items: Option<Vec<Items>>,
}

fn normalize_image_url(url: Option<String>) -> Option<String> {
    url.map(|u| {
        u.strip_prefix("http://")
            .map(|rest| format!("https://{}", rest))
            .unwrap_or(u)
    })
}

impl Images {
    fn normalize_urls(&mut self) {
        self.large = normalize_image_url(self.large.take());
        self.common = normalize_image_url(self.common.take());
        self.medium = normalize_image_url(self.medium.take());
        self.small = normalize_image_url(self.small.take());
        self.grid = normalize_image_url(self.grid.take());
    }
}

impl Items {
    fn normalize_image(&mut self) {
        if let Some(images) = &mut self.images {
            images.normalize_urls();
        }
    }
}

impl BangumiCalendarData {
    fn normalize(&mut self) {
        if let Some(items) = &mut self.items {
            for item in items {
                item.normalize_image();
            }
        }
    }
}

fn normalize_bangumi_data(data: &mut Vec<BangumiCalendarData>) {
    for day in data {
        day.normalize();
    }
}

/// 测速单个节点，返回响应时间（毫秒）
async fn ping_endpoint(url: String) -> Option<(String, u128)> {
    let start = std::time::Instant::now();

    match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?
        .head(&url)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let latency = start.elapsed().as_millis();
            Some((url, latency))
        }
        _ => None,
    }
}

/// 并发测速所有节点，返回按响应时间排序的 URL 列表
async fn speed_test_endpoints(urls: &[&str]) -> Vec<String> {
    let mut handles = Vec::new();

    // 为每个 URL 创建并发任务
    for &url in urls {
        let handle = tokio::spawn(ping_endpoint(url.to_string()));
        handles.push(handle);
    }

    // 等待所有任务完成并收集成功的结果
    let mut successful = Vec::new();
    for handle in handles {
        if let Ok(Some((url, latency))) = handle.await {
            successful.push((url, latency));
        }
    }

    // 按延迟排序
    successful.sort_by_key(|(_, latency)| *latency);

    // 返回排序后的 URL
    let sorted_urls: Vec<String> = successful.into_iter().map(|(url, _)| url).collect();

    // 如果没有任何节点响应，回退到原始顺序
    if sorted_urls.is_empty() {
        urls.iter().map(|&s| s.to_string()).collect()
    } else {
        sorted_urls
    }
}

async fn bangumi_calendar_data() -> Result<Vec<BangumiCalendarData>, String> {
    let urls = [
        "https://api.bgm.tv/calendar",
        "https://api.bangumi.one/calendar",
        // "https://api.bgm.rdd.moe/calendar",
        "https://bgmapi.anibt.net/calendar",
    ];

    // 先测速，获取按响应时间排序的节点列表
    let sorted_urls = speed_test_endpoints(&urls).await;

    let mut errors = Vec::new();

    // 按测速结果依次尝试每个节点
    for url in sorted_urls {
        match reqwest::get(&url).await {
            Ok(response) => {
                // 如果状态码不成功，记录错误并尝试下一个节点
                if !response.status().is_success() {
                    errors.push(format!("{}: HTTP status {}", url, response.status()));
                    continue;
                }

                // 尝试解析 JSON 数据
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let calendar_data = if let Some(array) = data.as_array() {
                            serde_json::from_value(serde_json::Value::Array(array.clone()))
                                .unwrap_or_default()
                        } else {
                            vec![]
                        };
                        // 成功获取并解析数据，直接返回
                        return Ok(calendar_data);
                    }
                    Err(e) => {
                        errors.push(format!("{}: JSON parse error: {}", url, e));
                    }
                }
            }
            Err(e) => {
                // 网络连接失败，记录错误并尝试下一个节点
                errors.push(format!("{}: Network error: {}", url, e));
            }
        }
    }

    // 如果所有节点都尝试失败，返回汇总的错误信息
    Err(format!(
        "Failed to fetch Bangumi data from all endpoints. Details:\n{}",
        errors.join("\n")
    ))
}

#[tauri::command]
pub async fn get_bangumi_calendar_data() -> Result<Vec<BangumiCalendarData>, String> {
    let mut data = bangumi_calendar_data().await?;
    normalize_bangumi_data(&mut data);
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_speed_test_endpoints() {
        let urls = [
            "https://api.bgm.tv/calendar",
            "https://api.bgm.rdd.moe/calendar",
            "https://bgmapi.anibt.net/calendar",
        ];

        let sorted = speed_test_endpoints(&urls).await;

        // 应该返回非空列表
        assert!(!sorted.is_empty());
        // 返回的数量应该 <= 原始数量（某些节点可能测速失败）
        assert!(sorted.len() <= urls.len());
    }

    #[tokio::test]
    async fn test_speed_test_all_unreachable() {
        // 使用不可达的 URL
        let urls = [
            "https://unreachable1.invalid/calendar",
            "https://unreachable2.invalid/calendar",
        ];

        let sorted = speed_test_endpoints(&urls).await;

        // 应该回退到原始顺序
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0], urls[0]);
        assert_eq!(sorted[1], urls[1]);
    }

    #[tokio::test]
    async fn test_ping_endpoint_timeout() {
        // 测试超时机制（使用一个慢速响应的服务）
        let result = ping_endpoint("https://httpstat.us/200?sleep=5000".to_string()).await;

        // 3秒超时，所以应该返回 None
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_bangumi_data() {
        let mut data = vec![BangumiCalendarData {
            weekday: Some(Weekday {
                en: "Monday".to_string(),
            }),
            items: Some(vec![Items {
                id: 123,
                name: "Test".to_string(),
                name_cn: "测试".to_string(),
                air_date: Some("2024-01-01".to_string()),
                images: Some(Images {
                    large: Some("http://example.com/large.jpg".to_string()),
                    common: None,
                    medium: None,
                    small: None,
                    grid: None,
                }),
                rating: Some(Rating { score: Some(8.5) }),
            }]),
        }];

        normalize_bangumi_data(&mut data);

        // 验证图片 URL 已被规范化
        let items = data[0].items.as_ref().unwrap();
        let image_url = &items[0].images.as_ref().unwrap().large;
        assert!(image_url.as_ref().unwrap().starts_with("https://"));
    }

    #[test]
    fn test_normalize_image_url() {
        // HTTP 转 HTTPS
        assert_eq!(
            normalize_image_url(Some("http://example.com/image.jpg".to_string())),
            Some("https://example.com/image.jpg".to_string())
        );

        // 已经是 HTTPS 的不变
        assert_eq!(
            normalize_image_url(Some("https://example.com/image.jpg".to_string())),
            Some("https://example.com/image.jpg".to_string())
        );

        // None 返回 None
        assert_eq!(normalize_image_url(None), None);
    }
}
