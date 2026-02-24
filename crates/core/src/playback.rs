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
