use reqwest::Client;
use serde::{Deserialize, Serialize};
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
// 远程版本信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageJson {
    version: String,
}

async fn fetch_latest_github_release(client: &Client) -> Option<String> {
    let urls = [
        "https://raw.githubusercontent.com/Geon97/QuantumTV/main/package.json",
        "https://cdn.jsdelivr.net/gh/Geon97/QuantumTV@main/package.json",
        "https://fastly.jsdelivr.net/gh/Geon97/QuantumTV@main/package.json",
        "https://ghproxy.net/https://raw.githubusercontent.com/Geon97/QuantumTV/main/package.json",
    ];

    for url in urls {
        let resp = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(3))
            .send()
            .await;

        let Ok(resp) = resp else { continue };

        if !resp.status().is_success() {
            continue;
        }

        // ⚠️ 有些镜像会返回 HTML，这一步是关键兜底
        let Ok(pkg) = resp.json::<PackageJson>().await else {
            continue;
        };

        if !pkg.version.is_empty() {
            return Some(pkg.version);
        }
    }

    None
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
    let remote_version = fetch_latest_github_release(&client)
        .await
        .unwrap_or_else(|| "unknown".into());

    let download_url = RELEASE_URL.to_string();

    Ok(Some(RemoteVersionInfo {
        version: remote_version.clone(),
        timestamp: remote_ts.clone(),
        build_time: format_timestamp(&remote_ts),
        release_notes: vec![
            "发现新版本可用".into(),
            format!("最新版本: {}", remote_version),
            format!("构建时间: {}", format_timestamp(&remote_ts)),
        ],
        download_url: download_url.into(),
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
