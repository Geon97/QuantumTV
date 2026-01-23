use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Weekday {
    en: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rating {
    score: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Images {
    large: Option<String>,
    common: Option<String>,
    medium: Option<String>,
    small: Option<String>,
    grid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Items {
    id: i32,
    name: String,
    name_cn: String,
    rating: Option<Rating>,
    air_date: Option<String>,
    images: Option<Images>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BangumiCalendarData {
    weekday: Option<Weekday>,
    items: Option<Vec<Items>>,
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
    let response = reqwest::get("https://api.bgm.tv/calendar")
        .await
        .map_err(|e| format!("Failed to fetch Bangumi data: {}", e))?;
    if !response.status().is_success() {
        return Ok(vec![]);
    }
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to fetch Bangumi data: {}", e))?;
    // 如果为数组
    let calendar_data = if let Some(array) = data.as_array() {
        serde_json::from_value(serde_json::Value::Array(array.clone())).unwrap_or_default()
    } else {
        vec![]
    };
    Ok(calendar_data)
}

#[tauri::command]
pub async fn get_bangumi_calendar_data() -> Result<Vec<BangumiCalendarData>, String> {
    let mut data = bangumi_calendar_data().await?;
    normalize_bangumi_data(&mut data);
    Ok(data)
}
