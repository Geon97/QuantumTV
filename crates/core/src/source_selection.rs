use crate::types::SearchResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 视频源测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTestResult {
    pub quality: String,
    pub load_speed: String,
    pub ping_time: u64,
    pub has_error: bool,
}

/// 从 m3u8 URL 测试视频源质量
pub async fn test_video_source(
    client: &Client,
    m3u8_url: &str,
) -> Result<SourceTestResult, String> {
    // 1. 测量 ping 时间（HEAD 请求）
    let ping_start = Instant::now();
    let ping_result = client
        .head(m3u8_url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;
    let ping_time = ping_start.elapsed().as_millis() as u64;

    if ping_result.is_err() {
        return Err("Failed to ping m3u8 URL".to_string());
    }

    // 2. 获取 m3u8 内容以解析质量
    let m3u8_content = client
        .get(m3u8_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch m3u8: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read m3u8 content: {}", e))?;

    // 3. 解析质量信息
    let quality = parse_quality_from_m3u8(&m3u8_content);

    // 4. 测量下载速度（下载第一个 ts 分片）
    let load_speed = if let Some(first_segment_url) = extract_first_segment(&m3u8_content, m3u8_url)
    {
        measure_download_speed(client, &first_segment_url).await
    } else {
        "未知".to_string()
    };

    Ok(SourceTestResult {
        quality,
        load_speed,
        ping_time,
        has_error: false,
    })
}

/// 从 m3u8 内容解析视频质量
fn parse_quality_from_m3u8(content: &str) -> String {
    // 查找 RESOLUTION 标签
    for line in content.lines() {
        if line.contains("RESOLUTION=") {
            if let Some(resolution_part) = line.split("RESOLUTION=").nth(1) {
                if let Some(resolution) = resolution_part.split(',').next() {
                    // 解析宽度
                    if let Some(width_str) = resolution.split('x').next() {
                        if let Ok(width) = width_str.parse::<u32>() {
                            return match width {
                                w if w >= 3840 => "4K".to_string(),
                                w if w >= 2560 => "2K".to_string(),
                                w if w >= 1920 => "1080p".to_string(),
                                w if w >= 1280 => "720p".to_string(),
                                w if w >= 854 => "480p".to_string(),
                                _ => "SD".to_string(),
                            };
                        }
                    }
                }
            }
        }
    }

    // 如果没有找到 RESOLUTION，尝试从 BANDWIDTH 推测
    let mut max_bandwidth = 0u32;
    for line in content.lines() {
        if line.contains("BANDWIDTH=") {
            if let Some(bandwidth_part) = line.split("BANDWIDTH=").nth(1) {
                if let Some(bandwidth_str) = bandwidth_part.split(',').next() {
                    if let Ok(bandwidth) = bandwidth_str.parse::<u32>() {
                        max_bandwidth = max_bandwidth.max(bandwidth);
                    }
                }
            }
        }
    }

    // 根据带宽推测质量
    if max_bandwidth > 0 {
        return match max_bandwidth {
            b if b >= 8_000_000 => "1080p".to_string(),
            b if b >= 5_000_000 => "720p".to_string(),
            b if b >= 2_000_000 => "480p".to_string(),
            _ => "SD".to_string(),
        };
    }

    "未知".to_string()
}

/// 从 m3u8 内容提取第一个 ts 分片 URL
fn extract_first_segment(content: &str, base_url: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            // 如果是相对路径，需要拼接 base URL
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return Some(trimmed.to_string());
            } else {
                // 相对路径处理
                if let Some(base) = base_url.rsplit_once('/') {
                    return Some(format!("{}/{}", base.0, trimmed));
                }
            }
        }
    }
    None
}

/// 测量下载速度
async fn measure_download_speed(client: &Client, url: &str) -> String {
    let start = Instant::now();

    // 只下载前 512KB 来测速
    let result = client
        .get(url)
        .header("Range", "bytes=0-524287") // 512KB
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    match result {
        Ok(response) => {
            if let Ok(bytes) = response.bytes().await {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let size_kb = bytes.len() as f64 / 1024.0;
                    let speed_kbps = size_kb / elapsed;

                    if speed_kbps >= 1024.0 {
                        return format!("{:.1} MB/s", speed_kbps / 1024.0);
                    } else {
                        return format!("{:.1} KB/s", speed_kbps);
                    }
                }
            }
        }
        Err(_) => return "未知".to_string(),
    }

    "未知".to_string()
}

/// 计算播放源综合评分
pub fn calculate_source_score(
    test_result: &SourceTestResult,
    max_speed: f64,
    min_ping: u64,
    max_ping: u64,
) -> f64 {
    let mut score = 0.0;

    // 1. 分辨率评分 (40% 权重)
    let quality_score = match test_result.quality.as_str() {
        "4K" => 100.0,
        "2K" => 85.0,
        "1080p" => 75.0,
        "720p" => 60.0,
        "480p" => 40.0,
        "SD" => 20.0,
        _ => 0.0,
    };
    score += quality_score * 0.4;

    // 2. 下载速度评分 (40% 权重)
    let speed_score = parse_speed_score(&test_result.load_speed, max_speed);
    score += speed_score * 0.4;

    // 3. 网络延迟评分 (20% 权重)
    let ping_score = if max_ping == min_ping {
        100.0 // 所有延迟相同，给满分
    } else if test_result.ping_time > 0 {
        let ping_ratio = (max_ping - test_result.ping_time) as f64 / (max_ping - min_ping) as f64;
        (ping_ratio * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    score += ping_score * 0.2;

    (score * 100.0).round() / 100.0 // 保留两位小数
}

/// 解析速度字符串并计算评分
fn parse_speed_score(speed_str: &str, max_speed: f64) -> f64 {
    if speed_str == "未知" || speed_str == "测量中..." {
        return 30.0;
    }

    // 解析速度值，格式如 "1.5 MB/s" 或 "500.0 KB/s"
    let parts: Vec<&str> = speed_str.split_whitespace().collect();
    if parts.len() != 2 {
        return 30.0;
    }

    let value = parts[0].parse::<f64>().unwrap_or(0.0);
    let unit = parts[1];

    let speed_kbps = if unit == "MB/s" {
        value * 1024.0
    } else {
        value
    };

    if max_speed > 0.0 {
        let speed_ratio = speed_kbps / max_speed;
        (speed_ratio * 100.0).clamp(0.0, 100.0)
    } else {
        30.0
    }
}

/// 从多个播放源中选择最佳源
pub async fn prefer_best_source(
    client: &Client,
    sources: Vec<SearchResult>,
) -> Result<(SearchResult, Vec<(String, SourceTestResult)>), String> {
    if sources.is_empty() {
        return Err("No sources provided".to_string());
    }

    if sources.len() == 1 {
        return Ok((sources[0].clone(), vec![]));
    }

    // 分批测速，避免一次性过多请求
    let batch_size = (sources.len() + 1) / 2; // 分成两批
    let mut all_results = Vec::new();

    for start in (0..sources.len()).step_by(batch_size) {
        let end = (start + batch_size).min(sources.len());
        let batch = &sources[start..end];

        let mut batch_handles = Vec::new();

        for source in batch {
            // 检查是否有可用的播放地址
            if source.episodes.is_empty() {
                continue;
            }

            // 使用第二集或第一集
            let episode_url = if source.episodes.len() > 1 {
                source.episodes[1].clone()
            } else {
                source.episodes[0].clone()
            };

            let client_clone = client.clone();
            let source_clone = source.clone();

            let handle = tokio::spawn(async move {
                match test_video_source(&client_clone, &episode_url).await {
                    Ok(result) => Some((source_clone, result)),
                    Err(_) => None,
                }
            });

            batch_handles.push(handle);
        }

        // 等待当前批次完成
        for handle in batch_handles {
            if let Ok(Some(result)) = handle.await {
                all_results.push(result);
            }
        }
    }

    if all_results.is_empty() {
        return Ok((sources[0].clone(), vec![]));
    }

    // 计算最大速度和延迟范围
    let valid_speeds: Vec<f64> = all_results
        .iter()
        .filter_map(|(_, result)| {
            let parts: Vec<&str> = result.load_speed.split_whitespace().collect();
            if parts.len() == 2 {
                let value = parts[0].parse::<f64>().ok()?;
                let unit = parts[1];
                Some(if unit == "MB/s" {
                    value * 1024.0
                } else {
                    value
                })
            } else {
                None
            }
        })
        .collect();

    let max_speed = valid_speeds.iter().cloned().fold(1024.0, f64::max);

    let valid_pings: Vec<u64> = all_results
        .iter()
        .map(|(_, result)| result.ping_time)
        .filter(|&ping| ping > 0)
        .collect();

    let min_ping = valid_pings.iter().cloned().min().unwrap_or(50);
    let max_ping = valid_pings.iter().cloned().max().unwrap_or(1000);

    // 计算每个源的评分
    let mut scored_results: Vec<(SearchResult, SourceTestResult, f64)> = all_results
        .into_iter()
        .map(|(source, result)| {
            let score = calculate_source_score(&result, max_speed, min_ping, max_ping);
            (source, result, score)
        })
        .collect();

    // 按评分排序
    scored_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    // 返回最佳源和所有测试结果
    let best_source = scored_results[0].0.clone();
    let test_results: Vec<(String, SourceTestResult)> = scored_results
        .into_iter()
        .map(|(source, result, _score)| {
            let key = format!("{}-{}", source.source, source.id);
            (key, result)
        })
        .collect();

    Ok((best_source, test_results))
}
