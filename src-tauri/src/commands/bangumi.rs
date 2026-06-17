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

async fn bangumi_calendar_data() -> Result<Vec<BangumiCalendarData>, String> {
    let urls = [
        "https://api.bgm.tv/calendar",
        "https://api.bgm.rdd.moe/calendar",
        "https://bgmapi.anibt.net/calendar",
    ];

    let mut errors = Vec::new();

    // 依次尝试每个节点
    for &url in &urls {
        match reqwest::get(url).await {
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
