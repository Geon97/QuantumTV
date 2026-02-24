use crate::commands::video::{get_video_detail_optimized, SearchCacheManager};
use crate::storage::StorageManager;
use quantumtv_core::types::SearchResult;
use tauri::State;

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
#[tauri::command]
pub async fn preload_next_episode(
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
