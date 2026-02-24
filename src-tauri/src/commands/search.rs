use crate::db::db_client::Db;
use quantumtv_core::search_aggregation::{
    aggregate_search_results, apply_filter, compute_group_stats, sort_by_year, AggregatedGroup,
    SearchFilter,
};
use quantumtv_core::types::SearchResult;
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;

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

/// 聚合搜索结果并返回分组统计
///
/// # Arguments
/// * `results` - 搜索结果列表
/// * `query` - 搜索查询关键词
/// * `normalized_query` - 可选的归一化查询(如繁简转换后)
///
/// # Returns
/// 返回聚合后的分组Map: key -> AggregatedGroup
#[tauri::command]
pub async fn aggregate_search_results_command(
    results: Vec<SearchResult>,
    query: String,
    normalized_query: Option<String>,
) -> Result<HashMap<String, AggregatedGroup>, String> {
    let start = std::time::Instant::now();
    let result_count = results.len();

    let aggregated_list =
        aggregate_search_results(results, &query, normalized_query.as_deref());

    let mut aggregated_map: HashMap<String, AggregatedGroup> = HashMap::new();

    for (key, group) in aggregated_list {
        let stats = compute_group_stats(&group);
        aggregated_map.insert(key, stats);
    }

    let duration = start.elapsed();
    log::info!(
        "聚合搜索结果完成: {} 条结果 -> {} 个分组, 耗时 {:?}",
        result_count,
        aggregated_map.len(),
        duration
    );

    Ok(aggregated_map)
}

/// 应用过滤器并排序
///
/// # Arguments
/// * `results` - 搜索结果列表
/// * `filter` - 过滤器配置
///
/// # Returns
/// 过滤和排序后的结果列表
#[tauri::command]
pub async fn filter_and_sort_results(
    results: Vec<SearchResult>,
    filter: SearchFilter,
) -> Result<Vec<SearchResult>, String> {
    let start = std::time::Instant::now();
    let input_count = results.len();

    let filtered = apply_filter(results, &filter);
    let sorted = sort_by_year(filtered, filter.year_order);

    let duration = start.elapsed();
    log::debug!(
        "过滤排序完成: {} 条 -> {} 条, 耗时 {:?}",
        input_count,
        sorted.len(),
        duration
    );

    Ok(sorted)
}

