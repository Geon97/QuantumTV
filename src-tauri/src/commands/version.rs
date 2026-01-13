use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

const VERSION_SOURCE_URLS: [&str; 4] = [
    "https://raw.githubusercontent.com/Geon97/QuantumTV/main/VERSION.txt",
    "https://cdn.jsdelivr.net/gh/Geon97/QuantumTV@main/VERSION.txt",
    "https://fastly.jsdelivr.net/gh/Geon97/QuantumTV@main/VERSION.txt",
    "https://ghproxy.net/https://raw.githubusercontent.com/Geon97/QuantumTV/main/VERSION.txt",
];

const RELEASE_URL: &str = "https://github.com/Geon97/QuantumTV/releases";

// 编译时嵌入的版本信息
const BUILD_TIMESTAMP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../VERSION.txt"));
const APP_VERSION: &str = env!("APP_VERSION");

#[derive(Serialize)]
pub struct RemoteVersionInfo {
    pub version: String,
    pub timestamp: String,
    pub build_time: String,
    pub release_notes: Vec<String>,
    pub download_url: String,
}

fn get_build_timestamp() -> String {
    BUILD_TIMESTAMP.trim().to_string()
}

fn get_app_version() -> String {
    APP_VERSION.to_string()
}
fn is_valid_timestamp(ts: &str) -> bool {
    ts.len() == 14 && ts.chars().all(|c| c.is_ascii_digit())
}

fn format_timestamp(ts: &str) -> String {
    if !is_valid_timestamp(ts) {
        return ts.to_string();
    }

    format!(
        "{}-{}-{} {}:{}:{}",
        &ts[0..4],
        &ts[4..6],
        &ts[6..8],
        &ts[8..10],
        &ts[10..12],
        &ts[12..14],
    )
}

async fn fetch_text(client: &Client, url: &str) -> Option<String> {
    let resp = client
        .get(url)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await
        .ok()?;

    if resp.status().is_success() {
        resp.text().await.ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

async fn fetch_remote_timestamp(client: &Client) -> Option<String> {
    for url in VERSION_SOURCE_URLS {
        if let Some(ts) = fetch_text(client, url).await {
            if is_valid_timestamp(&ts) {
                return Some(ts);
            }
        }
    }
    None
}
#[tauri::command]
pub fn get_current_version() -> String {
    get_app_version()
}

#[tauri::command]
pub async fn version_for_updates() -> Result<Option<RemoteVersionInfo>, ()> {
    let local_ts = get_build_timestamp();

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| ())?;

    let remote_ts = match fetch_remote_timestamp(&client).await {
        Some(ts) => ts,
        None => return Ok(None),
    };

    println!("Remote timestamp: {}", remote_ts);
    println!("Local timestamp: {}", local_ts);
    let local_num: u64 = local_ts.parse().map_err(|_| ())?;
    let remote_num: u64 = remote_ts.parse().map_err(|_| ())?;
    if remote_num <= local_num {
        return Ok(None);
    }

    let display_version = get_current_version();

    Ok(Some(RemoteVersionInfo {
        version: display_version.clone(),
        timestamp: remote_ts.clone(),
        build_time: format_timestamp(&remote_ts),
        release_notes: vec![
            "发现新版本可用".into(),
            format!("最新版本: {}", display_version),
            format!("构建时间: {}", format_timestamp(&remote_ts)),
        ],
        download_url: RELEASE_URL.into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_version_for_updates_returns_none_when_no_remote_update() {
        let result = version_for_updates().await;
        match result {
            Ok(None) => {
                println!("测试通过：无更新返回 None");
            }
            Ok(Some(_)) => {
                panic!("不应该有更新");
            }
            Err(_) => {
                panic!("不应该报错");
            }
        }
    }
    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        println!("version: {}", version);
        assert!(!version.is_empty());
    }

    #[test]
    fn test_get_build_timestamp() {
        let ts = get_build_timestamp();
        println!("timestamp: {}", ts);
        assert_eq!(ts.len(), 14);
        assert!(ts.chars().all(|c| c.is_ascii_digit()));
    }
}
