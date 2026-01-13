use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateStatus {
    Checking,
    HasUpdate,
    NoUpdate,
    FetchFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCheckResult {
    pub status: UpdateStatus,
    pub local_timestamp: Option<String>,
    pub remote_timestamp: Option<String>,
    pub formatted_local_time: Option<String>,
    pub formatted_remote_time: Option<String>,
    pub error: Option<String>,
}

const REMOTE_VERSION_URLS: &[&str] = &[
    "https://cdn.jsdelivr.net/gh/Geon97/QuantumTV@main/VERSION.txt",
    "https://fastly.jsdelivr.net/gh/Geon97/QuantumTV@main/VERSION.txt",
    "https://raw.githubusercontent.com/Geon97/QuantumTV/main/VERSION.txt",
    "https://ghproxy.net/https://raw.githubusercontent.com/Geon97/QuantumTV/main/VERSION.txt",
    "https://mirror.ghproxy.com/https://raw.githubusercontent.com/Geon97/QuantumTV/main/VERSION.txt",
];

// 编译时嵌入本地版本时间戳
const BUILD_TIMESTAMP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../VERSION.txt"));

// 工具：格式化时间戳 YYYYMMDDHHMMSS -> 可读字符串
fn format_timestamp(ts: &str) -> Option<String> {
    if ts.len() != 14 || !ts.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year = &ts[0..4];
    let month = &ts[4..6];
    let day = &ts[6..8];
    let hour = &ts[8..10];
    let min = &ts[10..12];
    let sec = &ts[12..14];
    Some(format!("{year}-{month}-{day} {hour}:{min}:{sec}"))
}

// 比较版本时间戳，使用字符串比较即可（14位数字的字典序）
fn compare_timestamps(local: &str, remote: &str) -> i8 {
    use std::cmp::Ordering;
    match local.cmp(remote) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

fn get_local_timestamp() -> String {
    BUILD_TIMESTAMP.trim().to_string()
}

async fn fetch_url_with_timeout(
    client: &Client,
    url: &str,
    timeout_ms: u64,
) -> reqwest::Result<Option<String>> {
    let fut = client.get(url).header("Cache-Control", "no-cache").send();
    match timeout(Duration::from_millis(timeout_ms), fut).await {
        Ok(Ok(resp)) if resp.status().is_success() => {
            let text = resp.text().await?;
            let trimmed = text.trim().to_string();
            if trimmed.len() == 14 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                Ok(Some(trimmed))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

async fn fetch_remote_timestamp(client: &Client) -> Option<String> {
    for &url in REMOTE_VERSION_URLS {
        if let Ok(Some(ts)) = fetch_url_with_timeout(client, url, 6000).await {
            return Some(ts);
        }
    }
    None
}

// 主版本检测函数
#[tauri::command]
pub async fn check_for_updates() -> VersionCheckResult {
    let client = Client::builder()
        .user_agent("QuantumTV-VersionCheck")
        .build()
        .unwrap();

    let local_ts = get_local_timestamp();

    let remote_ts = fetch_remote_timestamp(&client).await;

    let status = if let Some(remote) = &remote_ts {
        match compare_timestamps(&local_ts, remote) {
            -1 => UpdateStatus::HasUpdate,
            _ => UpdateStatus::NoUpdate,
        }
    } else {
        UpdateStatus::FetchFailed
    };

    VersionCheckResult {
        status,
        local_timestamp: Some(local_ts.clone()),
        remote_timestamp: remote_ts.clone(),
        formatted_local_time: format_timestamp(&local_ts),
        formatted_remote_time: remote_ts.as_deref().and_then(format_timestamp),
        error: if remote_ts.is_none() {
            Some("无法获取远程版本信息".into())
        } else {
            None
        },
    }
}

// 为方便 Default，实现（部分字段可空）
impl Default for VersionCheckResult {
    fn default() -> Self {
        VersionCheckResult {
            status: UpdateStatus::Checking,
            local_timestamp: None,
            remote_timestamp: None,
            formatted_local_time: None,
            formatted_remote_time: None,
            error: None,
        }
    }
}
