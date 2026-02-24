// 是否为成人源
const ADULT_KEYWORDS: [&str; 6] = ["adult", "18+", "nsfw", "成人", "情色", "🔞"];

pub fn is_adult_source(source: &str) -> bool {
    // 解析 源 是否为成人
    let source = source.to_lowercase();
    ADULT_KEYWORDS
        .iter()
        .any(|&keyword| source.contains(keyword))
}

/// 过滤18+内容源
///
/// # Arguments
///
/// * `sources` - 源名称列表
/// * `is_adult` - 每个源对应的是否为18+源的标记列表
/// * `filter_enabled` - 是否启用过滤
///
/// # Returns
///
/// 返回过滤后的源索引列表
pub fn filter_adult_sources(
    sources: &[String],
    is_adult: &[bool],
    filter_enabled: bool,
) -> Vec<usize> {
    if !filter_enabled {
        // 如果不启用过滤，返回所有索引
        return (0..sources.len()).collect();
    }

    sources
        .iter()
        .enumerate()
        .filter_map(|(idx, _)| {
            if idx < is_adult.len() && !is_adult[idx] {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
