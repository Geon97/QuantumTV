/// 视频源智能选择和成功率追踪
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// 源测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTestResult {
    pub source_key: String,
    pub success: bool,
    pub response_time_ms: u64,
    pub error_reason: Option<String>,
    pub timestamp: u64,
}

/// 源统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStats {
    pub source_key: String,
    pub total_tests: u64,
    pub successful_tests: u64,
    pub failed_tests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub last_success_time: Option<u64>,
    pub last_failure_time: Option<u64>,
    pub consecutive_failures: u32,
}

/// 源性能记录
#[derive(Debug, Clone)]
struct SourcePerformance {
    total_tests: u64,
    successful_tests: u64,
    total_response_time_ms: u64,
    last_success_time: Option<Instant>,
    last_failure_time: Option<Instant>,
    consecutive_failures: u32,
    recent_results: Vec<bool>, // 最近 20 次测试结果
}

impl SourcePerformance {
    fn new() -> Self {
        Self {
            total_tests: 0,
            successful_tests: 0,
            total_response_time_ms: 0,
            last_success_time: None,
            last_failure_time: None,
            consecutive_failures: 0,
            recent_results: Vec::new(),
        }
    }

    fn record_test(&mut self, success: bool, response_time_ms: u64) {
        self.total_tests += 1;

        if success {
            self.successful_tests += 1;
            self.total_response_time_ms += response_time_ms;
            self.last_success_time = Some(Instant::now());
            self.consecutive_failures = 0;
        } else {
            self.last_failure_time = Some(Instant::now());
            self.consecutive_failures += 1;
        }

        // 保留最近 20 次结果
        self.recent_results.push(success);
        if self.recent_results.len() > 20 {
            self.recent_results.remove(0);
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        (self.successful_tests as f64 / self.total_tests as f64) * 100.0
    }

    fn recent_success_rate(&self) -> f64 {
        if self.recent_results.is_empty() {
            return 0.0;
        }
        let recent_successes = self.recent_results.iter().filter(|&&x| x).count();
        (recent_successes as f64 / self.recent_results.len() as f64) * 100.0
    }

    fn avg_response_time(&self) -> u64 {
        if self.successful_tests == 0 {
            return 0;
        }
        self.total_response_time_ms / self.successful_tests
    }

    fn to_stats(&self, source_key: String) -> SourceStats {
        SourceStats {
            source_key,
            total_tests: self.total_tests,
            successful_tests: self.successful_tests,
            failed_tests: self.total_tests - self.successful_tests,
            success_rate: self.success_rate(),
            avg_response_time_ms: self.avg_response_time(),
            last_success_time: self.last_success_time.map(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }),
            last_failure_time: self.last_failure_time.map(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }),
            consecutive_failures: self.consecutive_failures,
        }
    }
}

/// 源智能选择管理器
pub struct SourceIntelligenceManager {
    performances: Arc<Mutex<HashMap<String, SourcePerformance>>>,
    max_consecutive_failures: u32,
}

impl SourceIntelligenceManager {
    pub fn new() -> Self {
        Self {
            performances: Arc::new(Mutex::new(HashMap::new())),
            max_consecutive_failures: 3,
        }
    }

    /// 记录源测试结果
    pub fn record_test_result(&self, result: SourceTestResult) {
        let mut performances = self.performances.lock().unwrap();
        let perf = performances
            .entry(result.source_key.clone())
            .or_insert_with(SourcePerformance::new);

        perf.record_test(result.success, result.response_time_ms);
    }

    /// 获取所有源的统计信息
    pub fn get_all_stats(&self) -> Vec<SourceStats> {
        let performances = self.performances.lock().unwrap();
        performances
            .iter()
            .map(|(key, perf)| perf.to_stats(key.clone()))
            .collect()
    }

    /// 获取单个源的统计信息
    pub fn get_source_stats(&self, source_key: &str) -> Option<SourceStats> {
        let performances = self.performances.lock().unwrap();
        performances
            .get(source_key)
            .map(|perf| perf.to_stats(source_key.to_string()))
    }

    /// 根据成功率和响应时间排序源
    pub fn rank_sources(&self, source_keys: Vec<String>) -> Vec<String> {
        let performances = self.performances.lock().unwrap();

        let mut sources_with_scores: Vec<(String, f64)> = source_keys
            .into_iter()
            .map(|key| {
                let score = if let Some(perf) = performances.get(&key) {
                    // 如果连续失败次数过多，降低优先级
                    if perf.consecutive_failures >= self.max_consecutive_failures {
                        return (key, 0.0);
                    }

                    // 计算综合得分
                    let recent_rate = perf.recent_success_rate();
                    let overall_rate = perf.success_rate();
                    let avg_time = perf.avg_response_time();

                    // 权重：最近成功率 50%，总体成功率 30%，响应时间 20%
                    let rate_score = recent_rate * 0.5 + overall_rate * 0.3;

                    // 响应时间得分（越快越好，超过 5000ms 得分为 0）
                    let time_score = if avg_time == 0 {
                        20.0 // 没有数据时给予中等分数
                    } else if avg_time > 5000 {
                        0.0
                    } else {
                        ((5000 - avg_time) as f64 / 5000.0) * 20.0
                    };

                    rate_score + time_score
                } else {
                    // 没有历史数据的源给予中等分数
                    50.0
                };
                (key, score)
            })
            .collect();

        // 按得分降序排序
        sources_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        sources_with_scores.into_iter().map(|(key, _)| key).collect()
    }

    /// 判断源是否应该被跳过（连续失败过多）
    pub fn should_skip_source(&self, source_key: &str) -> bool {
        let performances = self.performances.lock().unwrap();
        if let Some(perf) = performances.get(source_key) {
            perf.consecutive_failures >= self.max_consecutive_failures
        } else {
            false
        }
    }

    /// 清除源的统计信息
    pub fn clear_source_stats(&self, source_key: &str) {
        let mut performances = self.performances.lock().unwrap();
        performances.remove(source_key);
    }

    /// 清除所有统计信息
    pub fn clear_all_stats(&self) {
        let mut performances = self.performances.lock().unwrap();
        performances.clear();
    }

    /// 重置源的连续失败计数
    pub fn reset_consecutive_failures(&self, source_key: &str) {
        let mut performances = self.performances.lock().unwrap();
        if let Some(perf) = performances.get_mut(source_key) {
            perf.consecutive_failures = 0;
        }
    }
}

// ========== Tauri 命令 ==========

/// 记录源测试结果
#[tauri::command]
pub fn record_source_test(
    result: SourceTestResult,
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<(), String> {
    manager.record_test_result(result);
    Ok(())
}

/// 获取所有源的统计信息
#[tauri::command]
pub fn get_all_source_stats(
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Vec<SourceStats>, String> {
    Ok(manager.get_all_stats())
}

/// 获取单个源的统计信息
#[tauri::command]
pub fn get_source_stats(
    source_key: String,
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Option<SourceStats>, String> {
    Ok(manager.get_source_stats(&source_key))
}

/// 根据成功率排序源
#[tauri::command]
pub fn rank_sources(
    source_keys: Vec<String>,
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Vec<String>, String> {
    Ok(manager.rank_sources(source_keys))
}

/// 清除所有源统计信息
#[tauri::command]
pub fn clear_all_source_stats(
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<(), String> {
    manager.clear_all_stats();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_performance_record() {
        let mut perf = SourcePerformance::new();

        perf.record_test(true, 100);
        assert_eq!(perf.total_tests, 1);
        assert_eq!(perf.successful_tests, 1);
        assert_eq!(perf.consecutive_failures, 0);

        perf.record_test(false, 0);
        assert_eq!(perf.total_tests, 2);
        assert_eq!(perf.successful_tests, 1);
        assert_eq!(perf.consecutive_failures, 1);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut perf = SourcePerformance::new();

        perf.record_test(true, 100);
        perf.record_test(true, 200);
        perf.record_test(false, 0);

        assert_eq!(perf.success_rate(), 66.66666666666666);
        assert_eq!(perf.avg_response_time(), 150);
    }

    #[test]
    fn test_recent_success_rate() {
        let mut perf = SourcePerformance::new();

        // 添加 10 次成功
        for _ in 0..10 {
            perf.record_test(true, 100);
        }

        // 添加 10 次失败
        for _ in 0..10 {
            perf.record_test(false, 0);
        }

        // 最近 20 次中有 10 次成功
        assert_eq!(perf.recent_success_rate(), 50.0);
    }

    #[test]
    fn test_source_ranking() {
        let manager = SourceIntelligenceManager::new();

        // 记录不同源的测试结果
        manager.record_test_result(SourceTestResult {
            source_key: "source1".to_string(),
            success: true,
            response_time_ms: 100,
            error_reason: None,
            timestamp: 0,
        });

        manager.record_test_result(SourceTestResult {
            source_key: "source2".to_string(),
            success: true,
            response_time_ms: 500,
            error_reason: None,
            timestamp: 0,
        });

        manager.record_test_result(SourceTestResult {
            source_key: "source3".to_string(),
            success: false,
            response_time_ms: 0,
            error_reason: Some("timeout".to_string()),
            timestamp: 0,
        });

        let ranked = manager.rank_sources(vec![
            "source1".to_string(),
            "source2".to_string(),
            "source3".to_string(),
        ]);

        // source1 应该排第一（响应快且成功）
        assert_eq!(ranked[0], "source1");
    }

    #[test]
    fn test_should_skip_source() {
        let manager = SourceIntelligenceManager::new();

        // 记录 3 次连续失败
        for _ in 0..3 {
            manager.record_test_result(SourceTestResult {
                source_key: "bad_source".to_string(),
                success: false,
                response_time_ms: 0,
                error_reason: Some("error".to_string()),
                timestamp: 0,
            });
        }

        assert!(manager.should_skip_source("bad_source"));
    }

    #[test]
    fn test_reset_consecutive_failures() {
        let manager = SourceIntelligenceManager::new();

        // 记录失败
        manager.record_test_result(SourceTestResult {
            source_key: "source1".to_string(),
            success: false,
            response_time_ms: 0,
            error_reason: Some("error".to_string()),
            timestamp: 0,
        });

        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 1);

        // 重置
        manager.reset_consecutive_failures("source1");

        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 0);
    }
}
