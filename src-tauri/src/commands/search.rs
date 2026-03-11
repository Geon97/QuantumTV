use crate::commands::config::{get_user_preferences, UserPreferences};
use crate::commands::video::{search_with_cache_hit, SearchCacheManager};
use crate::db::db_client::Db;
use crate::db::search_history::get_search_history;
use crate::storage::StorageManager;
use quantumtv_core::search_aggregation::{
    aggregate_search_results_with_filter, apply_filter, compute_group_stats, sort_by_year,
    AggregatedGroup, SearchFilter,
};
use quantumtv_core::types::SearchResult;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// 搜索结果缓存条目
struct CacheEntry {
    results: Vec<SearchResult>,
    created_at: Instant,
    last_accessed: Instant,
}

/// 搜索结果缓存管理器
/// 按查询关键词缓存原始搜索结果，避免重复传输
///
/// 特性：
/// - TTL: 30 分钟过期
/// - LRU: 最多保留 200 个会话
/// - 自动清理过期条目
pub struct SearchResultCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    max_entries: usize,
    ttl: Duration,
}

impl SearchResultCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_entries: 200,
            ttl: Duration::from_secs(30 * 60), // 30 分钟
        }
    }

    /// 保存搜索结果
    pub fn save(&self, query: &str, results: Vec<SearchResult>) {
        let normalized_query = query.trim().to_lowercase();
        let mut cache = self.cache.lock().unwrap();

        // 清理过期条目
        self.cleanup_expired(&mut cache);

        // LRU: 如果超过上限，删除最旧的条目
        if cache.len() >= self.max_entries {
            self.evict_oldest(&mut cache);
        }

        let now = Instant::now();
        cache.insert(
            normalized_query,
            CacheEntry {
                results,
                created_at: now,
                last_accessed: now,
            },
        );
    }

    /// 获取搜索结果
    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let normalized_query = query.trim().to_lowercase();
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get_mut(&normalized_query) {
            // 检查是否过期
            if entry.created_at.elapsed() > self.ttl {
                cache.remove(&normalized_query);
                return None;
            }

            // 更新访问时间（LRU）
            entry.last_accessed = Instant::now();
            return Some(entry.results.clone());
        }

        None
    }

    /// 清除所有缓存
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 获取缓存大小
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// 清理过期条目
    fn cleanup_expired(&self, cache: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        cache.retain(|_, entry| now.duration_since(entry.created_at) < self.ttl);
    }

    /// 驱逐最旧的条目（LRU）
    fn evict_oldest(&self, cache: &mut HashMap<String, CacheEntry>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        let now = Instant::now();

        let mut total_results = 0;
        let mut expired_count = 0;

        for entry in cache.values() {
            total_results += entry.results.len();
            if now.duration_since(entry.created_at) > self.ttl {
                expired_count += 1;
            }
        }

        CacheStats {
            total_entries: cache.len(),
            total_results,
            expired_count,
            max_entries: self.max_entries,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_results: usize,
    pub expired_count: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
}

/// 过滤结果缓存条目
struct FilterCacheEntry {
    response: ApplySearchFilterResponse,
    created_at: Instant,
}

/// 过滤结果缓存管理器
/// 缓存 (query, filter_agg, filter_all) 的计算结果
///
/// 特性：
/// - TTL: 5 分钟过期（过滤结果变化较快）
/// - LRU: 最多保留 100 个过滤结果
pub struct FilterResultCache {
    cache: Arc<Mutex<HashMap<String, FilterCacheEntry>>>,
    max_entries: usize,
    ttl: Duration,
}

impl FilterResultCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_entries: 100,
            ttl: Duration::from_secs(5 * 60), // 5 分钟
        }
    }

    /// 生成缓存键
    fn cache_key(query: &str, filter_agg: &SearchFilter, filter_all: &SearchFilter) -> String {
        format!(
            "{}|{}:{}:{}:{}|{}:{}:{}:{}",
            query.trim().to_lowercase(),
            filter_agg.source,
            filter_agg.title,
            filter_agg.year,
            format!("{:?}", filter_agg.year_order),
            filter_all.source,
            filter_all.title,
            filter_all.year,
            format!("{:?}", filter_all.year_order),
        )
    }

    /// 保存过滤结果
    pub fn save(
        &self,
        query: &str,
        filter_agg: &SearchFilter,
        filter_all: &SearchFilter,
        response: ApplySearchFilterResponse,
    ) {
        let key = Self::cache_key(query, filter_agg, filter_all);
        let mut cache = self.cache.lock().unwrap();

        // 清理过期条目
        self.cleanup_expired(&mut cache);

        // LRU: 如果超过上限，删除最旧的条目
        if cache.len() >= self.max_entries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, entry)| entry.created_at)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            key,
            FilterCacheEntry {
                response,
                created_at: Instant::now(),
            },
        );
    }

    /// 获取过滤结果
    pub fn get(
        &self,
        query: &str,
        filter_agg: &SearchFilter,
        filter_all: &SearchFilter,
    ) -> Option<ApplySearchFilterResponse> {
        let key = Self::cache_key(query, filter_agg, filter_all);
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(&key) {
            // 检查是否过期
            if entry.created_at.elapsed() > self.ttl {
                cache.remove(&key);
                return None;
            }

            return Some(entry.response.clone());
        }

        None
    }

    /// 清理过期条目
    fn cleanup_expired(&self, cache: &mut HashMap<String, FilterCacheEntry>) {
        let now = Instant::now();
        cache.retain(|_, entry| now.duration_since(entry.created_at) < self.ttl);
    }

    /// 清除所有缓存
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 获取缓存大小
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

#[tauri::command]
pub fn get_search_suggestions(db: State<'_, Db>, query: String) -> Result<Vec<String>, String> {
    // 获取数据库数据
    db.with_conn(|conn| {
        let like_query = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT keyword FROM search_history WHERE keyword LIKE ?1 ORDER BY save_time DESC LIMIT 10",
            )?;
        let rows = stmt.query_map(params![like_query], |row| row.get(0))?;
        let suggestions = rows.collect::<Result<Vec<String>, _>>()?;
        Ok(suggestions)
    })
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilterOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilterCategory {
    pub key: String,
    pub label: String,
    pub options: Vec<SearchFilterOption>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPageStateResponse {
    pub aggregated_entries: Vec<(String, AggregatedGroup)>,
    pub filtered_results: Vec<SearchResult>,
    pub filter_categories_all: Vec<SearchFilterCategory>,
    pub filter_categories_agg: Vec<SearchFilterCategory>,
}

#[derive(Debug, Serialize)]
pub struct SearchPageBootstrap {
    pub search_history: Vec<String>,
    pub fluid_search: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPageQueryResponse {
    pub results: Vec<SearchResult>,
    pub cache_hit: bool,
    pub filter_categories_all: Vec<SearchFilterCategory>,
    pub filter_categories_agg: Vec<SearchFilterCategory>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPageOpenResponse {
    pub search_history: Vec<String>,
    pub fluid_search: bool,
    pub results: Vec<SearchResult>,
    pub cache_hit: bool,
    pub filter_categories_all: Vec<SearchFilterCategory>,
    pub filter_categories_agg: Vec<SearchFilterCategory>,
}

fn build_filter_categories(results: &[SearchResult]) -> Vec<SearchFilterCategory> {
    use std::collections::{BTreeSet, HashMap};

    let mut sources_map: HashMap<String, String> = HashMap::new();
    let mut titles_set: BTreeSet<String> = BTreeSet::new();
    let mut years_set: BTreeSet<String> = BTreeSet::new();

    for item in results {
        if !item.source.is_empty() && !item.source_name.is_empty() {
            sources_map.insert(item.source.clone(), item.source_name.clone());
        }

        if !item.title.is_empty() {
            titles_set.insert(item.title.clone());
        }

        if let Some(year) = item
            .year
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            years_set.insert(year.to_string());
        }
    }

    let mut source_entries: Vec<(String, String)> = sources_map.into_iter().collect();
    source_entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut source_options = vec![SearchFilterOption {
        label: "全部来源".to_string(),
        value: "all".to_string(),
    }];
    source_options.extend(
        source_entries
            .into_iter()
            .map(|(value, label)| SearchFilterOption { label, value }),
    );

    let mut title_options = vec![SearchFilterOption {
        label: "全部标题".to_string(),
        value: "all".to_string(),
    }];
    title_options.extend(titles_set.into_iter().map(|value| SearchFilterOption {
        label: value.clone(),
        value,
    }));

    let mut years: Vec<String> = years_set.into_iter().collect();
    let has_unknown = years.iter().any(|y| y == "unknown");
    years.retain(|y| y != "unknown");
    years.sort_by(|a, b| {
        b.parse::<i32>()
            .unwrap_or(0)
            .cmp(&a.parse::<i32>().unwrap_or(0))
    });

    let mut year_options = vec![SearchFilterOption {
        label: "全部年份".to_string(),
        value: "all".to_string(),
    }];
    year_options.extend(years.into_iter().map(|value| SearchFilterOption {
        label: value.clone(),
        value,
    }));
    if has_unknown {
        year_options.push(SearchFilterOption {
            label: "未知".to_string(),
            value: "unknown".to_string(),
        });
    }

    vec![
        SearchFilterCategory {
            key: "source".to_string(),
            label: "来源".to_string(),
            options: source_options,
        },
        SearchFilterCategory {
            key: "title".to_string(),
            label: "标题".to_string(),
            options: title_options,
        },
        SearchFilterCategory {
            key: "year".to_string(),
            label: "年份".to_string(),
            options: year_options,
        },
    ]
}

fn build_search_bootstrap(
    search_history: Vec<String>,
    preferences: UserPreferences,
) -> SearchPageBootstrap {
    SearchPageBootstrap {
        search_history,
        fluid_search: preferences.fluid_search,
    }
}

#[tauri::command]
pub async fn build_search_page_state(
    results: Vec<SearchResult>,
    query: String,
    normalized_query: Option<String>,
    filter_agg: SearchFilter,
    filter_all: SearchFilter,
) -> Result<SearchPageStateResponse, String> {
    let filter_categories = build_filter_categories(&results);

    let aggregated_list = aggregate_search_results_with_filter(
        results.clone(),
        &query,
        normalized_query.as_deref(),
        &filter_agg,
    );
    let aggregated_entries = aggregated_list
        .into_iter()
        .map(|(key, group)| (key, compute_group_stats(&group)))
        .collect::<Vec<_>>();

    let filtered = apply_filter(results, &filter_all);
    let filtered_results = sort_by_year(filtered, filter_all.year_order.clone());

    Ok(SearchPageStateResponse {
        aggregated_entries,
        filtered_results,
        filter_categories_all: filter_categories.clone(),
        filter_categories_agg: filter_categories,
    })
}

/// 应用搜索过滤器响应
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplySearchFilterResponse {
    pub aggregated_entries: Vec<(String, AggregatedGroup)>,
    pub filtered_results: Vec<SearchResult>,
}

/// 应用搜索过滤器（从缓存读取结果）
///
/// 这是优化后的命令，避免每次过滤都传输完整结果集
/// 并且缓存过滤结果，减少重复计算
#[tauri::command]
pub async fn apply_search_filter(
    query: String,
    filter_agg: SearchFilter,
    filter_all: SearchFilter,
    result_cache: State<'_, SearchResultCache>,
    filter_cache: State<'_, FilterResultCache>,
) -> Result<ApplySearchFilterResponse, String> {
    // 先检查过滤结果缓存
    if let Some(cached_response) = filter_cache.get(&query, &filter_agg, &filter_all) {
        return Ok(cached_response);
    }

    // 从结果缓存获取原始搜索结果
    let results = result_cache
        .get(&query)
        .ok_or_else(|| "搜索结果未找到，请先执行搜索".to_string())?;

    // 聚合模式：应用 filter_agg
    let aggregated_list = aggregate_search_results_with_filter(
        results.clone(),
        &query,
        None,
        &filter_agg,
    );
    let aggregated_entries = aggregated_list
        .into_iter()
        .map(|(key, group)| (key, compute_group_stats(&group)))
        .collect::<Vec<_>>();

    // 全部模式：应用 filter_all
    let filtered = apply_filter(results, &filter_all);
    let filtered_results = sort_by_year(filtered, filter_all.year_order.clone());

    let response = ApplySearchFilterResponse {
        aggregated_entries,
        filtered_results,
    };

    // 保存到过滤结果缓存
    filter_cache.save(&query, &filter_agg, &filter_all, response.clone());

    Ok(response)
}

/// 获取搜索结果缓存统计信息
#[tauri::command]
pub fn get_search_cache_stats(result_cache: State<'_, SearchResultCache>) -> CacheStats {
    result_cache.stats()
}

#[tauri::command]
pub async fn get_search_page_bootstrap(
    db: State<'_, Db>,
    storage: State<'_, StorageManager>,
) -> Result<SearchPageBootstrap, String> {
    let search_history = get_search_history(db)?;
    let preferences = get_user_preferences(storage.clone()).await?;
    Ok(build_search_bootstrap(search_history, preferences))
}

#[tauri::command]
pub async fn search_page_query(
    query: String,
    app_handle: tauri::AppHandle,
    storage: State<'_, StorageManager>,
    cache: State<'_, SearchCacheManager>,
    result_cache: State<'_, SearchResultCache>,
) -> Result<SearchPageQueryResponse, String> {
    let (results, cache_hit) = search_with_cache_hit(query.clone(), app_handle, storage, cache).await?;

    // 保存搜索结果到缓存
    result_cache.save(&query, results.clone());

    let filter_categories = build_filter_categories(&results);

    Ok(SearchPageQueryResponse {
        results,
        cache_hit,
        filter_categories_all: filter_categories.clone(),
        filter_categories_agg: filter_categories,
    })
}

#[tauri::command]
pub async fn search_page_open(
    query: Option<String>,
    db: State<'_, Db>,
    storage: State<'_, StorageManager>,
    app_handle: tauri::AppHandle,
    cache: State<'_, SearchCacheManager>,
    result_cache: State<'_, SearchResultCache>,
) -> Result<SearchPageOpenResponse, String> {
    let search_history = get_search_history(db)?;
    let preferences = get_user_preferences(storage.clone()).await?;

    let trimmed_query = query.unwrap_or_default().trim().to_string();
    let (results, cache_hit) = if trimmed_query.is_empty() {
        (Vec::new(), false)
    } else {
        search_with_cache_hit(trimmed_query.clone(), app_handle, storage, cache).await?
    };

    // 保存搜索结果到缓存
    if !trimmed_query.is_empty() {
        result_cache.save(&trimmed_query, results.clone());
    }

    let filter_categories = build_filter_categories(&results);

    Ok(SearchPageOpenResponse {
        search_history,
        fluid_search: preferences.fluid_search,
        results,
        cache_hit,
        filter_categories_all: filter_categories.clone(),
        filter_categories_agg: filter_categories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_result_cache_saves_and_retrieves() {
        let cache = SearchResultCache::new();
        let results = vec![SearchResult {
            id: "1".to_string(),
            title: "测试".to_string(),
            ..Default::default()
        }];

        cache.save("测试查询", results.clone());
        let retrieved = cache.get("测试查询").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].id, "1");
    }

    #[test]
    fn search_result_cache_normalizes_query() {
        let cache = SearchResultCache::new();
        let results = vec![SearchResult {
            id: "1".to_string(),
            ..Default::default()
        }];

        cache.save("  Test Query  ", results.clone());

        // 不同格式的查询应该命中同一缓存
        assert!(cache.get("test query").is_some());
        assert!(cache.get("  TEST QUERY  ").is_some());
    }

    #[test]
    fn search_result_cache_clears() {
        let cache = SearchResultCache::new();
        cache.save("query1", vec![SearchResult::default()]);
        cache.save("query2", vec![SearchResult::default()]);

        assert_eq!(cache.size(), 2);
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn search_result_cache_returns_none_for_missing() {
        let cache = SearchResultCache::new();
        assert!(cache.get("不存在的查询").is_none());
    }

    #[test]
    fn search_result_cache_lru_eviction() {
        let cache = SearchResultCache::new();

        // 填充到上限
        for i in 0..200 {
            cache.save(&format!("query{}", i), vec![SearchResult::default()]);
        }
        assert_eq!(cache.size(), 200);

        // 添加新条目应该触发 LRU 驱逐
        cache.save("new_query", vec![SearchResult::default()]);
        assert_eq!(cache.size(), 200);

        // 新条目应该存在
        assert!(cache.get("new_query").is_some());
    }

    #[test]
    fn search_result_cache_stats() {
        let cache = SearchResultCache::new();

        cache.save("query1", vec![SearchResult::default(), SearchResult::default()]);
        cache.save("query2", vec![SearchResult::default()]);

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_results, 3);
        assert_eq!(stats.max_entries, 200);
        assert_eq!(stats.ttl_seconds, 30 * 60);
    }

    #[test]
    fn search_result_cache_updates_last_accessed() {
        let cache = SearchResultCache::new();
        let results = vec![SearchResult {
            id: "1".to_string(),
            ..Default::default()
        }];

        cache.save("query", results);

        // 第一次访问
        assert!(cache.get("query").is_some());

        // 添加更多条目
        for i in 0..10 {
            cache.save(&format!("other{}", i), vec![SearchResult::default()]);
        }

        // 再次访问应该更新 last_accessed
        assert!(cache.get("query").is_some());
    }

    #[test]
    fn filter_result_cache_saves_and_retrieves() {
        let cache = FilterResultCache::new();
        let filter_agg = SearchFilter {
            source: "all".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: quantumtv_core::search_aggregation::YearOrder::None,
        };
        let filter_all = filter_agg.clone();

        let response = ApplySearchFilterResponse {
            aggregated_entries: vec![],
            filtered_results: vec![],
        };

        cache.save("test_query", &filter_agg, &filter_all, response.clone());

        let retrieved = cache.get("test_query", &filter_agg, &filter_all);
        assert!(retrieved.is_some());
    }

    #[test]
    fn filter_result_cache_different_filters_different_keys() {
        let cache = FilterResultCache::new();
        let filter1 = SearchFilter {
            source: "source1".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: quantumtv_core::search_aggregation::YearOrder::None,
        };
        let filter2 = SearchFilter {
            source: "source2".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: quantumtv_core::search_aggregation::YearOrder::None,
        };

        let response1 = ApplySearchFilterResponse {
            aggregated_entries: vec![],
            filtered_results: vec![SearchResult {
                id: "1".to_string(),
                ..Default::default()
            }],
        };
        let response2 = ApplySearchFilterResponse {
            aggregated_entries: vec![],
            filtered_results: vec![SearchResult {
                id: "2".to_string(),
                ..Default::default()
            }],
        };

        cache.save("query", &filter1, &filter1, response1);
        cache.save("query", &filter2, &filter2, response2);

        // 不同的过滤器应该有不同的缓存条目
        let result1 = cache.get("query", &filter1, &filter1).unwrap();
        let result2 = cache.get("query", &filter2, &filter2).unwrap();

        assert_eq!(result1.filtered_results[0].id, "1");
        assert_eq!(result2.filtered_results[0].id, "2");
    }

    #[test]
    fn filter_result_cache_lru_eviction() {
        let cache = FilterResultCache::new();
        let filter = SearchFilter {
            source: "all".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: quantumtv_core::search_aggregation::YearOrder::None,
        };

        // 填充到上限
        for i in 0..100 {
            let response = ApplySearchFilterResponse {
                aggregated_entries: vec![],
                filtered_results: vec![],
            };
            cache.save(&format!("query{}", i), &filter, &filter, response);
        }
        assert_eq!(cache.size(), 100);

        // 添加新条目应该触发 LRU 驱逐
        let response = ApplySearchFilterResponse {
            aggregated_entries: vec![],
            filtered_results: vec![],
        };
        cache.save("new_query", &filter, &filter, response);
        assert_eq!(cache.size(), 100);
    }

    #[test]
    fn build_filter_categories_sorts_and_includes_unknown() {
        let results = vec![
            SearchResult {
                source: "s1".to_string(),
                source_name: "Beta".to_string(),
                title: "B".to_string(),
                year: Some("2022".to_string()),
                ..Default::default()
            },
            SearchResult {
                source: "s2".to_string(),
                source_name: "Alpha".to_string(),
                title: "A".to_string(),
                year: Some("unknown".to_string()),
                ..Default::default()
            },
            SearchResult {
                source: "s3".to_string(),
                source_name: "".to_string(),
                title: "C".to_string(),
                year: None,
                ..Default::default()
            },
        ];

        let categories = build_filter_categories(&results);
        let source_options = &categories[0].options;
        assert_eq!(source_options[0].value, "all");
        assert_eq!(source_options[1].label, "Alpha");
        assert_eq!(source_options[2].label, "Beta");

        let title_options = &categories[1].options;
        assert_eq!(title_options[0].value, "all");
        assert_eq!(title_options[1].label, "A");
        assert_eq!(title_options[2].label, "B");
        assert_eq!(title_options[3].label, "C");

        let year_options = &categories[2].options;
        assert_eq!(year_options[0].value, "all");
        assert_eq!(year_options[1].label, "2022");
        assert_eq!(year_options[2].label, "未知");
    }

    #[test]
    fn build_search_bootstrap_uses_fluid_search_flag() {
        let mut prefs = UserPreferences::default();
        prefs.fluid_search = false;

        let bootstrap = build_search_bootstrap(vec!["a".to_string()], prefs);
        assert_eq!(bootstrap.search_history, vec!["a".to_string()]);
        assert!(!bootstrap.fluid_search);
    }

    // ========== 集成测试 ==========

    /// 测试完整的搜索和过滤流程
    #[test]
    fn test_search_filter_cache_integration() {
        let result_cache = SearchResultCache::new();
        let filter_cache = FilterResultCache::new();

        // 1. 模拟搜索结果
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                title: "复仇者联盟".to_string(),
                source: "source1".to_string(),
                source_name: "来源1".to_string(),
                year: Some("2012".to_string()),
                episodes: vec!["ep1".to_string()],
                ..Default::default()
            },
            SearchResult {
                id: "2".to_string(),
                title: "复仇者联盟".to_string(),
                source: "source2".to_string(),
                source_name: "来源2".to_string(),
                year: Some("2012".to_string()),
                episodes: vec!["ep1".to_string()],
                ..Default::default()
            },
        ];

        // 2. 保存搜索结果到缓存
        result_cache.save("复仇者", results.clone());

        // 3. 验证可以获取搜索结果
        let cached_results = result_cache.get("复仇者");
        assert!(cached_results.is_some());
        assert_eq!(cached_results.unwrap().len(), 2);

        // 4. 创建过滤器
        let filter_agg = quantumtv_core::search_aggregation::SearchFilter {
            source: "all".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: quantumtv_core::search_aggregation::YearOrder::None,
        };
        let filter_all = filter_agg.clone();

        // 5. 模拟过滤结果并缓存
        let response = ApplySearchFilterResponse {
            aggregated_entries: vec![],
            filtered_results: results.clone(),
        };
        filter_cache.save("复仇者", &filter_agg, &filter_all, response.clone());

        // 6. 验证过滤结果缓存命中
        let cached_filter = filter_cache.get("复仇者", &filter_agg, &filter_all);
        assert!(cached_filter.is_some());
        assert_eq!(cached_filter.unwrap().filtered_results.len(), 2);
    }

    /// 测试并发访问
    #[test]
    fn test_concurrent_cache_access() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(SearchResultCache::new());
        let mut handles = vec![];

        // 启动多个线程并发写入
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                cache_clone.save(
                    &format!("query{}", i),
                    vec![SearchResult {
                        id: format!("{}", i),
                        ..Default::default()
                    }],
                );
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有条目都已保存
        assert_eq!(cache.size(), 10);
    }

    /// 测试大数据量性能
    #[test]
    fn test_large_dataset_performance() {
        let cache = SearchResultCache::new();

        // 创建大量搜索结果
        let large_results: Vec<SearchResult> = (0..1000)
            .map(|i| SearchResult {
                id: format!("id{}", i),
                title: format!("Title {}", i),
                source: format!("source{}", i % 10),
                source_name: format!("Source {}", i % 10),
                year: Some(format!("{}", 2000 + (i % 25))),
                episodes: vec![format!("ep{}", i)],
                ..Default::default()
            })
            .collect();

        // 保存大数据集
        let start = std::time::Instant::now();
        cache.save("large_query", large_results.clone());
        let save_duration = start.elapsed();

        // 验证保存时间合理（应该 < 100ms）
        assert!(save_duration.as_millis() < 100, "Save took too long: {:?}", save_duration);

        // 读取大数据集
        let start = std::time::Instant::now();
        let retrieved = cache.get("large_query");
        let get_duration = start.elapsed();

        // 验证读取时间合理（应该 < 50ms）
        assert!(get_duration.as_millis() < 50, "Get took too long: {:?}", get_duration);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 1000);
    }

    /// 测试边界条件：空结果集
    #[test]
    fn test_empty_results_handling() {
        let cache = SearchResultCache::new();

        // 保存空结果集
        cache.save("no_results", vec![]);
        let result = cache.get("no_results");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 0);
    }

    /// 测试统计信息准确性
    #[test]
    fn test_cache_stats_accuracy() {
        let cache = SearchResultCache::new();

        // 添加不同大小的结果集
        cache.save("query1", vec![SearchResult::default(); 10]);
        cache.save("query2", vec![SearchResult::default(); 20]);
        cache.save("query3", vec![SearchResult::default(); 30]);

        let stats = cache.stats();

        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.total_results, 60);
        assert_eq!(stats.max_entries, 200);
        assert_eq!(stats.ttl_seconds, 30 * 60);
        assert_eq!(stats.expired_count, 0);
    }
}
