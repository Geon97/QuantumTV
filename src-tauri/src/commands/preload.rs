use crate::commands::video::{get_video_detail_optimized, SearchCacheManager};
use crate::storage::StorageManager;
use quantumtv_core::types::SearchResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use tauri::State;

#[derive(Debug, Default)]
struct PreloadGate {
    seen: HashSet<String>,
}

impl PreloadGate {
    fn should_preload(&mut self, key: &str) -> bool {
        if self.seen.contains(key) {
            false
        } else {
            self.seen.insert(key.to_string());
            true
        }
    }
}

fn preload_gate() -> &'static Mutex<PreloadGate> {
    static GATE: OnceLock<Mutex<PreloadGate>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(PreloadGate::default()))
}

fn should_trigger_preload(current_time: f64, total_duration: f64) -> bool {
    total_duration > 0.0 && current_time / total_duration >= 0.85
}

fn build_preload_key(source: &str, id: &str, next_episode: u32) -> String {
    format!("{source}_{id}_{next_episode}")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreloadDecision {
    pub did_preload: bool,
}

/// 预载下一集视频详情
///
/// # Arguments
/// * `source` - 当前视频源
/// * `id` - 当前视频ID
/// * `current_episode` - 当前集数索引
/// * `total_episodes` - 总集数
///
/// # Returns
/// 返回下一集的详情，如果是最后一集则返回 None
async fn preload_next_episode(
    source: String,
    id: String,
    current_episode: u32,
    total_episodes: u32,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
) -> Result<Option<SearchResult>, String> {
    let start = std::time::Instant::now();

    // 检查是否还有下一集
    if current_episode + 1 >= total_episodes {
        log::debug!(
            "预载跳过: 已是最后一集 ({}/{})",
            current_episode + 1,
            total_episodes
        );
        return Ok(None);
    }

    log::info!(
        "开始预载下一集: source={}, id={}, 当前集={}, 总集数={}",
        source,
        id,
        current_episode + 1,
        total_episodes
    );

    // 预加载当前视频详情（包含所有集数）
    // 使用 get_video_detail_optimized 会自动缓存结果
    match get_video_detail_optimized(
        source.clone(),
        id.clone(),
        storage,
        cache,
        Some(true), // 包含其他源
    )
    .await
    {
        Ok(detail_response) => {
            let duration = start.elapsed();
            log::info!(
                "预载成功: source={}, id={}, 耗时 {:?}",
                source,
                id,
                duration
            );
            Ok(Some(detail_response.detail))
        }
        Err(e) => {
            let duration = start.elapsed();
            // 预加载失败不应该影响当前播放
            log::warn!(
                "预载失败: source={}, id={}, 错误={}, 耗时 {:?}",
                source,
                id,
                e,
                duration
            );
            Ok(None)
        }
    }
}

/// 按播放进度触发预载下一集，内部负责「只预载一次」的去重。
#[tauri::command]
pub async fn preload_next_episode_if_needed(
    source: String,
    id: String,
    current_episode: u32,
    total_episodes: u32,
    current_time: f64,
    total_duration: f64,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
) -> Result<PreloadDecision, String> {
    if !should_trigger_preload(current_time, total_duration) {
        return Ok(PreloadDecision { did_preload: false });
    }

    if current_episode + 1 >= total_episodes {
        return Ok(PreloadDecision { did_preload: false });
    }

    let next_episode = current_episode + 1;
    let key = build_preload_key(&source, &id, next_episode);
    {
        let mut gate = preload_gate()
            .lock()
            .map_err(|_| "Preload gate poisoned".to_string())?;
        if !gate.should_preload(&key) {
            return Ok(PreloadDecision { did_preload: false });
        }
    }

    let _ = preload_next_episode(
        source,
        id,
        current_episode,
        total_episodes,
        storage,
        cache,
    )
    .await?;

    Ok(PreloadDecision { did_preload: true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_trigger_preload_respects_threshold() {
        assert!(!should_trigger_preload(0.0, 100.0));
        assert!(!should_trigger_preload(84.0, 100.0));
        assert!(should_trigger_preload(85.0, 100.0));
        assert!(should_trigger_preload(100.0, 100.0));
    }

    #[test]
    fn build_preload_key_is_stable() {
        let key = build_preload_key("s1", "id1", 2);
        assert_eq!(key, "s1_id1_2");
    }

    #[test]
    fn preload_gate_only_allows_once() {
        let mut gate = PreloadGate::default();
        assert!(gate.should_preload("k1"));
        assert!(!gate.should_preload("k1"));
        assert!(gate.should_preload("k2"));
    }
}
