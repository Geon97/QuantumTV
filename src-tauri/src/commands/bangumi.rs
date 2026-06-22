use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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

/// 全局番剧代理配置
static BANGUMI_PROXY_URL: Mutex<String> = Mutex::new(String::new());

/// 设置番剧代理配置
pub fn set_bangumi_proxy_url(url: &str) {
    if let Ok(mut guard) = BANGUMI_PROXY_URL.lock() {
        *guard = url.to_string();
    }
}

/// 获取番剧代理配置
fn get_bangumi_proxy_url() -> String {
    if let Ok(guard) = BANGUMI_PROXY_URL.lock() {
        guard.clone()
    } else {
        String::new()
    }
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

/// 根据代理配置获取要使用的 URL 列表
fn get_bangumi_urls() -> Vec<String> {
    let proxy_url = get_bangumi_proxy_url();

    if !proxy_url.is_empty() {
        // 用户设置了自定义代理，优先使用，失败时回退到备选端点
        vec![
            proxy_url,
            "https://api.bangumi.one/calendar".to_string(),
            "https://bgmapi.anibt.net/calendar".to_string(),
        ]
    } else {
        // 未设置代理，使用默认端点列表（按可靠性排序）
        vec![
            "https://api.bgm.tv/calendar".to_string(),
            "https://api.bangumi.one/calendar".to_string(),
            "https://bgmapi.anibt.net/calendar".to_string(),
        ]
    }
}

async fn bangumi_calendar_data() -> Result<Vec<BangumiCalendarData>, String> {
    let urls = get_bangumi_urls();
    let mut errors = Vec::new();

    for url in urls {
        match reqwest::get(&url).await {
            Ok(response) => {
                if !response.status().is_success() {
                    errors.push(format!("{}: HTTP status {}", url, response.status()));
                    continue;
                }

                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let calendar_data = if let Some(array) = data.as_array() {
                            serde_json::from_value(serde_json::Value::Array(array.clone()))
                                .unwrap_or_default()
                        } else {
                            vec![]
                        };
                        return Ok(calendar_data);
                    }
                    Err(e) => {
                        errors.push(format!("{}: JSON parse error: {}", url, e));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("{}: Network error: {}", url, e));
            }
        }
    }

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

        let items = data[0].items.as_ref().unwrap();
        let image_url = &items[0].images.as_ref().unwrap().large;
        assert!(image_url.as_ref().unwrap().starts_with("https://"));
    }

    #[test]
    fn test_normalize_image_url() {
        assert_eq!(
            normalize_image_url(Some("http://example.com/image.jpg".to_string())),
            Some("https://example.com/image.jpg".to_string())
        );

        assert_eq!(
            normalize_image_url(Some("https://example.com/image.jpg".to_string())),
            Some("https://example.com/image.jpg".to_string())
        );

        assert_eq!(normalize_image_url(None), None);
    }

    #[test]
    fn test_get_bangumi_urls_with_proxy() {
        set_bangumi_proxy_url("https://custom.bangumi.com/calendar");
        let urls = get_bangumi_urls();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://custom.bangumi.com/calendar");
        assert_eq!(urls[1], "https://api.bangumi.one/calendar");
        assert_eq!(urls[2], "https://bgmapi.anibt.net/calendar");
    }

    #[test]
    fn test_get_bangumi_urls_without_proxy() {
        set_bangumi_proxy_url("");
        let urls = get_bangumi_urls();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://api.bgm.tv/calendar");
        assert_eq!(urls[1], "https://api.bangumi.one/calendar");
        assert_eq!(urls[2], "https://bgmapi.anibt.net/calendar");
    }
}
