use quantumtv_core::playback::{SkipAction, SkipDetection};

/// 检查是否应该跳过片头或片尾
///
/// # Arguments
/// * `intro_time` - 片头结束时间(秒)
/// * `outro_time` - 片尾开始前的时间(秒)
/// * `current_time` - 当前播放时间(秒)
/// * `total_duration` - 视频总时长(秒)
///
/// # Returns
/// 返回 SkipAction: None | SkipIntro(target_time) | SkipOutro
#[tauri::command]
pub fn check_skip_action(
    intro_time: f64,
    outro_time: f64,
    current_time: f64,
    total_duration: f64,
) -> SkipAction {
    let detector = SkipDetection::new(intro_time, outro_time);
    let action = detector.check_skip_action(current_time, total_duration);

    // 只在触发跳过时记录日志，避免过多日志
    match &action {
        SkipAction::SkipIntro(target) => {
            log::info!(
                "触发片头跳过: {:.2}s -> {:.2}s (片头时长: {:.2}s)",
                current_time,
                target,
                intro_time
            );
        }
        SkipAction::SkipOutro => {
            log::info!(
                "触发片尾跳过: {:.2}s / {:.2}s (剩余: {:.2}s)",
                current_time,
                total_duration,
                total_duration - current_time
            );
        }
        SkipAction::None => {}
    }

    action
}
