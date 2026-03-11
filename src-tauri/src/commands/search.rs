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
use tauri::State;

/// 搜索结果缓存管理器
/// 按查询关键词缓存原始搜索结果，避免重复传输
pub struct SearchResultCache {
    cache: Arc<Mutex<HashMap<String, Vec<SearchResult>>>>,
}

impl SearchResultCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 保存搜索结果
    pub fn save(&self, query: &str, results: Vec<SearchResult>) {
        let normalized_query = query.trim().to_lowercase();
        let mut cache = self.cache.lock().unwrap();
        cache.insert(normalized_query, results);
    }

    /// 获取搜索结果
    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let normalized_query = query.trim().to_lowercase();
        let cache = self.cache.lock().unwrap();
        cache.get(&normalized_query).cloned()
    }

    /// 清除缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 获取缓存大小
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySearchFilterResponse {
    pub aggregated_entries: Vec<(String, AggregatedGroup)>,
    pub filtered_results: Vec<SearchResult>,
}

/// 应用搜索过滤器（从缓存读取结果）
///
/// 这是优化后的命令，避免每次过滤都传输完整结果集
#[tauri::command]
pub async fn apply_search_filter(
    query: String,
    filter_agg: SearchFilter,
    filter_all: SearchFilter,
    result_cache: State<'_, SearchResultCache>,
) -> Result<ApplySearchFilterResponse, String> {
    // 从缓存获取结果
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

    Ok(ApplySearchFilterResponse {
        aggregated_entries,
        filtered_results,
    })
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
}
