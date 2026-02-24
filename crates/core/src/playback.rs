use serde::{Deserialize, Serialize};

/// 跳过片头片尾检测器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipDetection {
    pub intro_time: f64,
    pub outro_time: f64,
}

impl SkipDetection {
    pub fn new(intro_time: f64, outro_time: f64) -> Self {
        Self {
            intro_time,
            outro_time,
        }
    }

    /// 检查是否应该跳过片头
    ///
    /// 返回 Some(target_time) 表示应该跳转到的时间点
    pub fn should_skip_intro(&self, current_time: f64) -> Option<f64> {
        if self.intro_time > 0.0 && current_time < self.intro_time {
            Some(self.intro_time)
        } else {
            None
        }
    }

    /// 检查是否应该跳过片尾
    ///
    /// 当播放进度超过 (总时长 - outro_time) 时返回 true
    pub fn should_skip_outro(&self, current_time: f64, total_duration: f64) -> bool {
        if self.outro_time > 0.0 && total_duration > 0.0 {
            total_duration - current_time <= self.outro_time
        } else {
            false
        }
    }

    /// 检查当前时间是否应该触发跳过动作
    ///
    /// 返回 SkipAction 表示应该执行的动作
    pub fn check_skip_action(
        &self,
        current_time: f64,
        total_duration: f64,
    ) -> SkipAction {
        if let Some(target_time) = self.should_skip_intro(current_time) {
            return SkipAction::SkipIntro(target_time);
        }

        if self.should_skip_outro(current_time, total_duration) {
            return SkipAction::SkipOutro;
        }

        SkipAction::None
    }
}

/// 跳过动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkipAction {
    /// 无需跳过
    None,
    /// 跳过片头到指定时间
    SkipIntro(f64),
    /// 跳过片尾 (可以直接播放下一集)
    SkipOutro,
}

// 去广告相关
pub fn filter_ads_from_m3_u8(content: &str) -> String {
    let mut result = String::with_capacity(content.len());

    // ===== 状态 =====
    let mut skip_block = false; // 是否在 CUE 广告块内
    let mut collecting_segment = false; // 是否正在收集一个完整 segment
    let mut segment_buf: Vec<&str> = Vec::with_capacity(8);

    // ===== 零分配广告 URL 检测 =====
    #[inline]
    fn is_ad_url(tag: &str) -> bool {
        let bytes = tag.as_bytes();

        bytes.windows(4).any(|w| w.eq_ignore_ascii_case(b"/ad/"))
            || bytes.windows(4).any(|w| w.eq_ignore_ascii_case(b"_ad_"))
            || bytes.windows(4).any(|w| w.eq_ignore_ascii_case(b"-ad-"))
            || bytes.windows(5).any(|w| w.eq_ignore_ascii_case(b"promo"))
            || bytes
                .windows(11)
                .any(|w| w.eq_ignore_ascii_case(b"doubleclick"))
    }

    // ===== DATERANGE 是否为广告 =====
    #[inline]
    fn is_ad_daterange(tag: &str) -> bool {
        let bytes = tag.as_bytes();

        bytes
            .windows(10)
            .any(|w| w.eq_ignore_ascii_case(b"CLASS=\"AD\""))
            || bytes
                .windows(12)
                .any(|w| w.eq_ignore_ascii_case(b"INTERSTITIAL"))
            || bytes.windows(5).any(|w| w.eq_ignore_ascii_case(b"X-AD-"))
            || bytes.windows(6).any(|w| w.eq_ignore_ascii_case(b"SCTE35"))
    }

    for line in content.lines() {
        let trimmed = line.trim();

        // =========================================================
        // 1️⃣ CUE 广告块控制 (SCTE-35)
        // =========================================================
        if trimmed.starts_with("#EXT-X-CUE-OUT") || trimmed.starts_with("#EXT-OATCLS-SCTE35") {
            skip_block = true;
            collecting_segment = false;
            segment_buf.clear();
            continue;
        }

        if trimmed.starts_with("#EXT-X-CUE-IN") {
            skip_block = false;
            continue;
        }

        if skip_block {
            collecting_segment = false;
            segment_buf.clear();
            continue;
        }

        // =========================================================
        // 2️⃣ DATERANGE —— 只过滤标签本身
        // =========================================================
        if trimmed.starts_with("#EXT-X-DATERANGE") {
            if is_ad_daterange(trimmed) {
                continue;
            }
        }

        // =========================================================
        // 3️⃣ LL-HLS PART
        // =========================================================
        if trimmed.starts_with("#EXT-X-PART") {
            if !is_ad_url(trimmed) {
                result.push_str(line);
                result.push('\n');
            }
            continue;
        }

        // =========================================================
        // 4️⃣ Segment 起点（EXTINF）
        // =========================================================
        if trimmed.starts_with("#EXTINF") {
            segment_buf.clear();
            segment_buf.push(line);
            collecting_segment = true;
            continue;
        }

        // =========================================================
        // 5️⃣ Segment 中间标签 & URL 处理
        // =========================================================
        if collecting_segment {
            // 仍然是 segment 附属标签（BYTERANGE / PDT / MAP 等）
            if trimmed.starts_with('#') {
                segment_buf.push(line);
                continue;
            }

            // 走到这里一定是 URL
            if !is_ad_url(trimmed) {
                for tag in &segment_buf {
                    result.push_str(tag);
                    result.push('\n');
                }
                result.push_str(line);
                result.push('\n');
            }

            collecting_segment = false;
            segment_buf.clear();
            continue;
        }

        // =========================================================
        // 6️⃣ 普通标签（头部信息 / MAP / VERSION 等）
        // =========================================================
        result.push_str(line);
        result.push('\n');
    }

    // 去除末尾多余换行
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_detection_intro() {
        let detector = SkipDetection::new(90.0, 120.0);

        // 应该跳过片头
        assert_eq!(detector.should_skip_intro(30.0), Some(90.0));
        assert_eq!(detector.should_skip_intro(0.0), Some(90.0));

        // 不应该跳过（已过片头时间）
        assert_eq!(detector.should_skip_intro(100.0), None);
        assert_eq!(detector.should_skip_intro(90.0), None);
    }

    #[test]
    fn test_skip_detection_outro() {
        let detector = SkipDetection::new(90.0, 120.0);
        let total_duration = 1800.0; // 30分钟

        // 应该跳过片尾（剩余时间 <= outro_time）
        assert!(detector.should_skip_outro(1680.0, total_duration)); // 剩余120秒
        assert!(detector.should_skip_outro(1700.0, total_duration)); // 剩余100秒

        // 不应该跳过（剩余时间 > outro_time）
        assert!(!detector.should_skip_outro(1600.0, total_duration)); // 剩余200秒
        assert!(!detector.should_skip_outro(0.0, total_duration));
    }

    #[test]
    fn test_check_skip_action() {
        let detector = SkipDetection::new(90.0, 120.0);
        let total_duration = 1800.0;

        // 应该跳过片头
        assert_eq!(
            detector.check_skip_action(30.0, total_duration),
            SkipAction::SkipIntro(90.0)
        );

        // 正常播放
        assert_eq!(
            detector.check_skip_action(500.0, total_duration),
            SkipAction::None
        );

        // 应该跳过片尾
        assert_eq!(
            detector.check_skip_action(1680.0, total_duration),
            SkipAction::SkipOutro
        );
    }

    #[test]
    fn test_skip_detection_edge_cases() {
        // 片头片尾时间为0
        let detector = SkipDetection::new(0.0, 0.0);
        assert_eq!(detector.should_skip_intro(50.0), None);
        assert!(!detector.should_skip_outro(1600.0, 1800.0));

        // 负数时间（容错）
        let detector = SkipDetection::new(-10.0, -10.0);
        assert_eq!(detector.should_skip_intro(50.0), None);
        assert!(!detector.should_skip_outro(1600.0, 1800.0));
    }

    #[test]
    fn test_filter_ads_basic() {
        let content = "#EXTM3U\n\
            #EXT-X-VERSION:3\n\
            #EXTINF:10.0\n\
            segment1.ts\n\
            #EXTINF:10.0\n\
            http://example.com/ad/segment.ts\n\
            #EXTINF:10.0\n\
            segment2.ts";

        let filtered = filter_ads_from_m3_u8(content);

        // 应该包含正常片段
        assert!(filtered.contains("segment1.ts"));
        assert!(filtered.contains("segment2.ts"));

        // 不应该包含广告片段
        assert!(!filtered.contains("/ad/"));
    }

    #[test]
    fn test_filter_ads_cue_out() {
        let content = "#EXTM3U\n\
            #EXTINF:10.0\n\
            segment1.ts\n\
            #EXT-X-CUE-OUT:30.0\n\
            #EXTINF:10.0\n\
            ad_segment.ts\n\
            #EXT-X-CUE-IN\n\
            #EXTINF:10.0\n\
            segment2.ts";

        let filtered = filter_ads_from_m3_u8(content);

        // CUE-OUT 和 CUE-IN 之间的片段应该被过滤
        assert!(filtered.contains("segment1.ts"));
        assert!(filtered.contains("segment2.ts"));
        assert!(!filtered.contains("ad_segment.ts"));
    }

    #[test]
    fn test_filter_ads_daterange() {
        let content = "#EXTM3U\n\
            #EXT-X-DATERANGE:ID=1,CLASS=\"AD\"\n\
            #EXTINF:10.0\n\
            segment1.ts";

        let filtered = filter_ads_from_m3_u8(content);

        // DATERANGE 广告标签应该被移除
        assert!(!filtered.contains("CLASS=\"AD\""));
        assert!(filtered.contains("segment1.ts"));
    }

    #[test]
    fn test_filter_ads_url_patterns() {
        let patterns = vec![
            "http://example.com/ad/video.ts",
            "http://example.com/video_ad_123.ts",
            "http://example.com/video-ad-123.ts",
            "http://example.com/promo/video.ts",
            "http://doubleclick.net/video.ts",
        ];

        for pattern in patterns {
            let content = format!("#EXTM3U\n#EXTINF:10.0\n{}", pattern);
            let filtered = filter_ads_from_m3_u8(&content);
            assert!(
                !filtered.contains(pattern),
                "Pattern should be filtered: {}",
                pattern
            );
        }
    }

    #[test]
    fn test_filter_ads_preserves_normal_content() {
        let content = "#EXTM3U\n\
            #EXT-X-VERSION:3\n\
            #EXT-X-TARGETDURATION:10\n\
            #EXTINF:10.0\n\
            segment1.ts\n\
            #EXTINF:10.0\n\
            segment2.ts";

        let filtered = filter_ads_from_m3_u8(content);

        // 所有正常内容应该保留
        assert!(filtered.contains("#EXTM3U"));
        assert!(filtered.contains("#EXT-X-VERSION:3"));
        assert!(filtered.contains("#EXT-X-TARGETDURATION:10"));
        assert!(filtered.contains("segment1.ts"));
        assert!(filtered.contains("segment2.ts"));
    }
}
