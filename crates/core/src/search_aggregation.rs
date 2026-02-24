use std::collections::HashMap;
use crate::types::SearchResult;
use serde::{Deserialize, Serialize};

/// 聚合后的分组统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedGroup {
    /// 代表性结果(组内第一个)
    pub representative: SearchResult,
    /// 最常见的剧集数
    pub episodes: usize,
    /// 来源名称列表(去重)
    pub source_names: Vec<String>,
    /// 最常见的豆瓣ID
    pub douban_id: Option<i32>,
}

/// 搜索过滤器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub source: String,
    pub title: String,
    pub year: String,
    pub year_order: YearOrder,
}

/// 年份排序顺序
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum YearOrder {
    None,
    Asc,
    Desc,
}

/// 聚合搜索结果
///
/// 按照 title + year + type 进行分组
/// type: 单集视频为 'movie', 多集为 'tv'
pub fn aggregate_search_results(
    results: Vec<SearchResult>,
    query: &str,
    normalized_query: Option<&str>,
) -> Vec<(String, Vec<SearchResult>)> {
    let query_lower = query.trim().to_lowercase();
    let query_no_space = query_lower.replace(" ", "");

    let norm_query = normalized_query.unwrap_or(&query_lower);
    let norm_query_lower = norm_query.trim().to_lowercase();
    let norm_query_no_space = norm_query_lower.replace(" ", "");

    // 过滤相关结果
    let relevant_results: Vec<SearchResult> = results.into_iter()
        .filter(|item| {
            let title_lower = item.title.to_lowercase();
            let title_no_space = title_lower.replace(" ", "");

            // 包含完整关键词
            if title_lower.contains(&query_lower)
                || title_no_space.contains(&query_no_space)
                || title_lower.contains(&norm_query_lower)
                || title_no_space.contains(&norm_query_no_space) {
                return true;
            }

            // 顺序包含关键词的所有字符 (原词)
            if subsequence_match(&title_no_space, &query_no_space) {
                return true;
            }

            // 顺序包含关键词的所有字符 (转换后的词)
            if norm_query != &query_lower && subsequence_match(&title_no_space, &norm_query_no_space) {
                return true;
            }

            false
        })
        .collect();

    // 聚合分组
    let mut map: HashMap<String, Vec<SearchResult>> = HashMap::new();
    let mut key_order: Vec<String> = Vec::new();

    for item in relevant_results {
        let item_type = if item.episodes.len() == 1 { "movie" } else { "tv" };
        let year_str = item.year.as_deref().unwrap_or("unknown");
        let key = format!(
            "{}-{}-{}",
            item.title.replace(" ", ""),
            year_str,
            item_type
        );

        let is_new_key = !map.contains_key(&key);
        map.entry(key.clone()).or_insert_with(Vec::new).push(item);

        if is_new_key {
            key_order.push(key);
        }
    }

    // 按出现顺序返回
    key_order.into_iter()
        .filter_map(|key| map.remove(&key).map(|group| (key, group)))
        .collect()
}

/// 子序列匹配: 检查 pattern 的所有字符是否按顺序出现在 text 中
fn subsequence_match(text: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars();
    let mut current_pattern = pattern_chars.next();

    for ch in text.chars() {
        if let Some(p) = current_pattern {
            if ch == p {
                current_pattern = pattern_chars.next();
            }
        } else {
            return true;
        }
    }

    current_pattern.is_none()
}

/// 计算分组统计信息
pub fn compute_group_stats(group: &[SearchResult]) -> AggregatedGroup {
    let episodes = calculate_most_common_episodes(group);
    let source_names = extract_unique_source_names(group);
    let douban_id = calculate_most_common_douban_id(group);

    AggregatedGroup {
        representative: group[0].clone(),
        episodes,
        source_names,
        douban_id,
    }
}

/// 计算最常见的剧集数
fn calculate_most_common_episodes(group: &[SearchResult]) -> usize {
    let mut count_map: HashMap<usize, u32> = HashMap::new();

    for result in group {
        let len = result.episodes.len();
        if len > 0 {
            *count_map.entry(len).or_insert(0) += 1;
        }
    }

    count_map.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(episodes, _)| episodes)
        .unwrap_or(0)
}

/// 提取唯一的来源名称列表
fn extract_unique_source_names(group: &[SearchResult]) -> Vec<String> {
    let mut source_names: Vec<String> = group.iter()
        .filter_map(|r| {
            if !r.source_name.is_empty() {
                Some(r.source_name.clone())
            } else {
                None
            }
        })
        .collect();

    source_names.sort();
    source_names.dedup();
    source_names
}

/// 计算最常见的豆瓣ID
fn calculate_most_common_douban_id(group: &[SearchResult]) -> Option<i32> {
    let mut count_map: HashMap<i32, u32> = HashMap::new();

    for result in group {
        if let Some(douban_id) = result.douban_id {
            if douban_id > 0 {
                *count_map.entry(douban_id).or_insert(0) += 1;
            }
        }
    }

    count_map.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(douban_id, _)| douban_id)
}

/// 应用过滤器
pub fn apply_filter(results: Vec<SearchResult>, filter: &SearchFilter) -> Vec<SearchResult> {
    results.into_iter()
        .filter(|item| {
            // 过滤来源
            if filter.source != "all" && item.source != filter.source {
                return false;
            }

            // 过滤标题
            if filter.title != "all" && item.title != filter.title {
                return false;
            }

            // 过滤年份
            if filter.year != "all" {
                let item_year = item.year.as_deref().unwrap_or("unknown");
                if item_year != filter.year {
                    return false;
                }
            }

            true
        })
        .collect()
}

/// 按年份排序
pub fn sort_by_year(mut results: Vec<SearchResult>, order: YearOrder) -> Vec<SearchResult> {
    if order == YearOrder::None {
        return results;
    }

    results.sort_by(|a, b| {
        let a_year = a.year.as_deref().unwrap_or("unknown");
        let b_year = b.year.as_deref().unwrap_or("unknown");
        compare_year(a_year, b_year, &order)
    });

    results
}

/// 年份比较函数
fn compare_year(a_year: &str, b_year: &str, order: &YearOrder) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    if *order == YearOrder::None {
        return Ordering::Equal;
    }

    let a_is_empty = a_year.is_empty() || a_year == "unknown";
    let b_is_empty = b_year.is_empty() || b_year == "unknown";

    if a_is_empty && b_is_empty {
        return Ordering::Equal;
    }
    if a_is_empty {
        return Ordering::Greater; // a 在后
    }
    if b_is_empty {
        return Ordering::Less; // b 在后
    }

    // 都是有效年份，按数字比较
    let a_num = a_year.parse::<i32>().unwrap_or(0);
    let b_num = b_year.parse::<i32>().unwrap_or(0);

    match order {
        YearOrder::Asc => a_num.cmp(&b_num),
        YearOrder::Desc => b_num.cmp(&a_num),
        YearOrder::None => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsequence_match() {
        assert!(subsequence_match("abcdef", "ace"));
        assert!(subsequence_match("hello world", "hld"));
        assert!(!subsequence_match("abc", "adc"));

        // 边界情况
        assert!(subsequence_match("", ""));
        assert!(subsequence_match("abc", ""));
        assert!(!subsequence_match("", "a"));
    }

    #[test]
    fn test_calculate_most_common_episodes() {
        let results = vec![
            SearchResult {
                episodes: vec!["1".to_string(), "2".to_string()],
                ..Default::default()
            },
            SearchResult {
                episodes: vec!["1".to_string(), "2".to_string()],
                ..Default::default()
            },
            SearchResult {
                episodes: vec!["1".to_string()],
                ..Default::default()
            },
        ];

        assert_eq!(calculate_most_common_episodes(&results), 2);

        // 空结果
        let empty: Vec<SearchResult> = vec![];
        assert_eq!(calculate_most_common_episodes(&empty), 0);
    }

    #[test]
    fn test_compare_year() {
        use std::cmp::Ordering;

        assert_eq!(compare_year("2024", "2023", &YearOrder::Desc), Ordering::Less);
        assert_eq!(compare_year("2023", "2024", &YearOrder::Asc), Ordering::Less);
        assert_eq!(compare_year("unknown", "2024", &YearOrder::Asc), Ordering::Greater);

        // 边界情况
        assert_eq!(compare_year("", "", &YearOrder::Asc), Ordering::Equal);
        assert_eq!(compare_year("unknown", "unknown", &YearOrder::Desc), Ordering::Equal);
        assert_eq!(compare_year("2024", "2024", &YearOrder::Asc), Ordering::Equal);
    }

    #[test]
    fn test_aggregate_search_results() {
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                title: "复仇者联盟".to_string(),
                year: Some("2012".to_string()),
                episodes: vec!["1".to_string()],
                source: "source1".to_string(),
                source_name: "来源1".to_string(),
                ..Default::default()
            },
            SearchResult {
                id: "2".to_string(),
                title: "复仇者联盟".to_string(),
                year: Some("2012".to_string()),
                episodes: vec!["1".to_string()],
                source: "source2".to_string(),
                source_name: "来源2".to_string(),
                ..Default::default()
            },
            SearchResult {
                id: "3".to_string(),
                title: "不相关的电影".to_string(),
                year: Some("2020".to_string()),
                episodes: vec!["1".to_string()],
                source: "source1".to_string(),
                source_name: "来源1".to_string(),
                ..Default::default()
            },
        ];

        let aggregated = aggregate_search_results(results, "复仇者", None);

        // 应该只有一个分组（两个"复仇者联盟"聚合在一起）
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].1.len(), 2);
    }

    #[test]
    fn test_apply_filter() {
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                title: "电影A".to_string(),
                year: Some("2023".to_string()),
                source: "source1".to_string(),
                ..Default::default()
            },
            SearchResult {
                id: "2".to_string(),
                title: "电影B".to_string(),
                year: Some("2024".to_string()),
                source: "source2".to_string(),
                ..Default::default()
            },
        ];

        // 按来源过滤
        let filter = SearchFilter {
            source: "source1".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: YearOrder::None,
        };
        let filtered = apply_filter(results.clone(), &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");

        // 按年份过滤
        let filter = SearchFilter {
            source: "all".to_string(),
            title: "all".to_string(),
            year: "2024".to_string(),
            year_order: YearOrder::None,
        };
        let filtered = apply_filter(results.clone(), &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
    }

    #[test]
    fn test_sort_by_year() {
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                year: Some("2024".to_string()),
                ..Default::default()
            },
            SearchResult {
                id: "2".to_string(),
                year: Some("2022".to_string()),
                ..Default::default()
            },
            SearchResult {
                id: "3".to_string(),
                year: Some("2023".to_string()),
                ..Default::default()
            },
        ];

        // 降序排序
        let sorted = sort_by_year(results.clone(), YearOrder::Desc);
        assert_eq!(sorted[0].id, "1"); // 2024
        assert_eq!(sorted[1].id, "3"); // 2023
        assert_eq!(sorted[2].id, "2"); // 2022

        // 升序排序
        let sorted = sort_by_year(results.clone(), YearOrder::Asc);
        assert_eq!(sorted[0].id, "2"); // 2022
        assert_eq!(sorted[1].id, "3"); // 2023
        assert_eq!(sorted[2].id, "1"); // 2024
    }

    #[test]
    fn test_compute_group_stats() {
        let group = vec![
            SearchResult {
                id: "1".to_string(),
                title: "测试剧集".to_string(),
                episodes: vec!["1".to_string(), "2".to_string()],
                source_name: "来源A".to_string(),
                douban_id: Some(123),
                ..Default::default()
            },
            SearchResult {
                id: "2".to_string(),
                title: "测试剧集".to_string(),
                episodes: vec!["1".to_string(), "2".to_string()],
                source_name: "来源B".to_string(),
                douban_id: Some(123),
                ..Default::default()
            },
        ];

        let stats = compute_group_stats(&group);
        assert_eq!(stats.episodes, 2);
        assert_eq!(stats.source_names.len(), 2);
        assert_eq!(stats.douban_id, Some(123));
    }

    #[test]
    fn test_extract_unique_source_names() {
        let group = vec![
            SearchResult {
                source_name: "来源A".to_string(),
                ..Default::default()
            },
            SearchResult {
                source_name: "来源B".to_string(),
                ..Default::default()
            },
            SearchResult {
                source_name: "来源A".to_string(),
                ..Default::default()
            },
        ];

        let names = extract_unique_source_names(&group);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"来源A".to_string()));
        assert!(names.contains(&"来源B".to_string()));
    }
}
