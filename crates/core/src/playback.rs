//  去广告相关
pub fn filter_ads_from_m3_u8(m3u8_content: String) -> String {
    if m3u8_content.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(m3u8_content.len());
    let mut skipping = false;
    for line in m3u8_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#EXT-X-DISCONTINUITY") {
            skipping = true;
            continue;
        }
        if skipping {
            if trimmed.starts_with("#EXT-X-DISCONTINUITY") {
                skipping = false;
            }
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}
