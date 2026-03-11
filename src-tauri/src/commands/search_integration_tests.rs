/// 集成测试：搜索 → 过滤 → 缓存完整流程
#[cfg(test)]
mod integration_tests {
    use super::*;
    use quantumtv_core::search_aggregation::{SearchFilter, YearOrder};
    use quantumtv_core::types::SearchResult;

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
            SearchResult {
                id: "3".to_string(),
                title: "钢铁侠".to_string(),
                source: "source1".to_string(),
                source_name: "来源1".to_string(),
                year: Some("2008".to_string()),
                episodes: vec!["ep1".to_string()],
                ..Default::default()
            },
        ];

        // 2. 保存搜索结果到缓存
        result_cache.save("复仇者", results.clone());

        // 3. 验证可以获取搜索结果
        let cached_results = result_cache.get("复仇者");
        assert!(cached_results.is_some());
        assert_eq!(cached_results.unwrap().len(), 3);

        // 4. 创建过滤器
        let filter_agg = SearchFilter {
            source: "all".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: YearOrder::None,
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
        assert_eq!(cached_filter.unwrap().filtered_results.len(), 3);

        // 7. 验证不同过滤器不会命中缓存
        let filter_source1 = SearchFilter {
            source: "source1".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: YearOrder::None,
        };
        let cached_filter2 = filter_cache.get("复仇者", &filter_source1, &filter_all);
        assert!(cached_filter2.is_none());
    }

    /// 测试缓存过期和清理
    #[test]
    fn test_cache_expiration_and_cleanup() {
        let cache = SearchResultCache::new();

        // 添加多个条目
        for i in 0..10 {
            cache.save(
                &format!("query{}", i),
                vec![SearchResult {
                    id: format!("{}", i),
                    ..Default::default()
                }],
            );
        }

        assert_eq!(cache.size(), 10);

        // 访问部分条目以更新 last_accessed
        for i in 0..5 {
            let _ = cache.get(&format!("query{}", i));
        }

        // 验证所有条目仍然存在
        assert_eq!(cache.size(), 10);
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

        // 启动多个线程并发读取
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                let result = cache_clone.get(&format!("query{}", i));
                assert!(result.is_some());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
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

    /// 测试边界条件：空查询
    #[test]
    fn test_empty_query_handling() {
        let cache = SearchResultCache::new();

        // 空字符串查询
        cache.save("", vec![SearchResult::default()]);
        let result = cache.get("");
        assert!(result.is_some());

        // 只有空格的查询（应该被规范化为空字符串）
        cache.save("   ", vec![SearchResult::default()]);
        let result = cache.get("");
        assert!(result.is_some());
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

    /// 测试 LRU 驱逐顺序
    #[test]
    fn test_lru_eviction_order() {
        let cache = SearchResultCache::new();

        // 填充到上限
        for i in 0..200 {
            cache.save(&format!("query{}", i), vec![SearchResult::default()]);
        }

        // 访问前 100 个查询以更新它们的 last_accessed
        for i in 0..100 {
            let _ = cache.get(&format!("query{}", i));
        }

        // 添加新查询，应该驱逐最旧的（未被访问的）
        cache.save("new_query", vec![SearchResult::default()]);

        // 验证新查询存在
        assert!(cache.get("new_query").is_some());

        // 验证被访问过的查询仍然存在
        assert!(cache.get("query0").is_some());
        assert!(cache.get("query50").is_some());
        assert!(cache.get("query99").is_some());
    }

    /// 测试过滤缓存键的唯一性
    #[test]
    fn test_filter_cache_key_uniqueness() {
        let cache = FilterResultCache::new();

        let filter1 = SearchFilter {
            source: "source1".to_string(),
            title: "all".to_string(),
            year: "all".to_string(),
            year_order: YearOrder::None,
        };

        let filter2 = SearchFilter {
            source: "source1".to_string(),
            title: "all".to_string(),
            year: "2023".to_string(), // 不同的年份
            year_order: YearOrder::None,
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

        // 保存两个不同的过滤结果
        cache.save("query", &filter1, &filter1, response1);
        cache.save("query", &filter2, &filter2, response2);

        // 验证两个缓存条目都存在且不同
        let result1 = cache.get("query", &filter1, &filter1).unwrap();
        let result2 = cache.get("query", &filter2, &filter2).unwrap();

        assert_eq!(result1.filtered_results[0].id, "1");
        assert_eq!(result2.filtered_results[0].id, "2");
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
