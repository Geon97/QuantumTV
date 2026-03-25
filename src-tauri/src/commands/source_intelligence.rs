use crate::db::db_client::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

const MAX_RECENT_RESULTS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceTestResult {
    pub source_key: String,
    pub success: bool,
    pub response_time_ms: u64,
    pub error_reason: Option<String>,
    pub timestamp: u64,
}

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
    pub last_available_time: Option<u64>,
    pub consecutive_failures: u32,
    pub auto_degraded: bool,
    pub recent_results: Vec<SourceTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SourcePerformance {
    total_tests: u64,
    successful_tests: u64,
    total_response_time_ms: u64,
    last_success_time: Option<u64>,
    last_failure_time: Option<u64>,
    last_available_time: Option<u64>,
    consecutive_failures: u32,
    auto_degraded: bool,
    recent_results: Vec<SourceTestResult>,
}

impl SourcePerformance {
    fn new() -> Self {
        Self::default()
    }

    fn record_test(&mut self, mut result: SourceTestResult, max_consecutive_failures: u32) {
        if result.timestamp == 0 {
            result.timestamp = current_timestamp();
        }

        self.total_tests += 1;

        if result.success {
            self.successful_tests += 1;
            self.total_response_time_ms += result.response_time_ms;
            self.last_success_time = Some(result.timestamp);
            self.last_available_time = Some(result.timestamp);
            self.consecutive_failures = 0;
            self.auto_degraded = false;
        } else {
            self.last_failure_time = Some(result.timestamp);
            self.consecutive_failures += 1;
            self.auto_degraded = self.consecutive_failures >= max_consecutive_failures;
        }

        self.recent_results.push(result);
        if self.recent_results.len() > MAX_RECENT_RESULTS {
            let overflow = self.recent_results.len() - MAX_RECENT_RESULTS;
            self.recent_results.drain(0..overflow);
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

        let successes = self
            .recent_results
            .iter()
            .filter(|item| item.success)
            .count();
        (successes as f64 / self.recent_results.len() as f64) * 100.0
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
            last_success_time: self.last_success_time,
            last_failure_time: self.last_failure_time,
            last_available_time: self.last_available_time,
            consecutive_failures: self.consecutive_failures,
            auto_degraded: self.auto_degraded,
            recent_results: self.recent_results.clone(),
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn to_i64(value: u64) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("value {} exceeds i64 range", value))
}

fn option_to_i64(value: Option<u64>) -> Result<Option<i64>, String> {
    value.map(to_i64).transpose()
}

fn to_u64(value: i64) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("negative database value {}", value))
}

fn option_to_u64(value: Option<i64>) -> Result<Option<u64>, String> {
    value.map(to_u64).transpose()
}

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

    pub fn load_from_db(&self, db: &Db) -> Result<(), String> {
        let rows: Vec<(
            String,
            i64,
            i64,
            i64,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            u32,
            i32,
            String,
        )> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    source_key,
                    total_tests,
                    successful_tests,
                    total_response_time_ms,
                    last_success_time,
                    last_failure_time,
                    last_available_time,
                    consecutive_failures,
                    auto_degraded,
                    recent_results_json
                 FROM source_intelligence_stats",
            )?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, u32>(7)?,
                        row.get::<_, i32>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(rows)
        })?;

        let mut performances = self.performances.lock().unwrap();
        performances.clear();

        for (
            source_key,
            total_tests,
            successful_tests,
            total_response_time_ms,
            last_success_time,
            last_failure_time,
            last_available_time,
            consecutive_failures,
            auto_degraded,
            recent_results_json,
        ) in rows
        {
            let recent_results =
                serde_json::from_str::<Vec<SourceTestResult>>(&recent_results_json)
                    .unwrap_or_default();

            performances.insert(
                source_key,
                SourcePerformance {
                    total_tests: to_u64(total_tests)?,
                    successful_tests: to_u64(successful_tests)?,
                    total_response_time_ms: to_u64(total_response_time_ms)?,
                    last_success_time: option_to_u64(last_success_time)?,
                    last_failure_time: option_to_u64(last_failure_time)?,
                    last_available_time: option_to_u64(last_available_time)?,
                    consecutive_failures,
                    auto_degraded: auto_degraded != 0,
                    recent_results,
                },
            );
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn record_test_result(&self, result: SourceTestResult) {
        self.apply_test_result(result);
    }

    pub fn record_test_result_persisted(
        &self,
        db: &Db,
        result: SourceTestResult,
    ) -> Result<(), String> {
        let (source_key, snapshot) = self.apply_test_result(result);
        self.persist_source_performance(db, &source_key, &snapshot)
    }

    pub fn record_runtime_test_result_persisted(
        &self,
        db: &Db,
        source_key: String,
        success: bool,
        response_time_ms: u64,
        error_reason: Option<String>,
    ) -> Result<(), String> {
        self.record_test_result_persisted(
            db,
            SourceTestResult {
                source_key,
                success,
                response_time_ms,
                error_reason,
                timestamp: current_timestamp(),
            },
        )
    }

    pub fn ensure_source_persisted(&self, db: &Db, source_key: String) -> Result<(), String> {
        let snapshot = {
            let mut performances = self.performances.lock().unwrap();
            performances
                .entry(source_key.clone())
                .or_insert_with(SourcePerformance::new)
                .clone()
        };

        self.persist_source_performance(db, &source_key, &snapshot)
    }

    pub fn ensure_sources_persisted(
        &self,
        db: &Db,
        source_keys: Vec<String>,
    ) -> Result<(), String> {
        let mut unique_keys = Vec::new();
        for key in source_keys {
            if !key.trim().is_empty() && !unique_keys.iter().any(|item| item == &key) {
                unique_keys.push(key);
            }
        }

        for source_key in unique_keys {
            self.ensure_source_persisted(db, source_key)?;
        }

        Ok(())
    }

    fn apply_test_result(&self, result: SourceTestResult) -> (String, SourcePerformance) {
        let mut performances = self.performances.lock().unwrap();
        let perf = performances
            .entry(result.source_key.clone())
            .or_insert_with(SourcePerformance::new);
        perf.record_test(result.clone(), self.max_consecutive_failures);
        (result.source_key, perf.clone())
    }

    fn persist_source_performance(
        &self,
        db: &Db,
        source_key: &str,
        perf: &SourcePerformance,
    ) -> Result<(), String> {
        let recent_results_json =
            serde_json::to_string(&perf.recent_results).map_err(|e| e.to_string())?;
        let total_tests = to_i64(perf.total_tests)?;
        let successful_tests = to_i64(perf.successful_tests)?;
        let total_response_time_ms = to_i64(perf.total_response_time_ms)?;
        let last_success_time = option_to_i64(perf.last_success_time)?;
        let last_failure_time = option_to_i64(perf.last_failure_time)?;
        let last_available_time = option_to_i64(perf.last_available_time)?;
        let updated_at = to_i64(current_timestamp())?;

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO source_intelligence_stats (
                    source_key,
                    total_tests,
                    successful_tests,
                    total_response_time_ms,
                    last_success_time,
                    last_failure_time,
                    last_available_time,
                    consecutive_failures,
                    auto_degraded,
                    recent_results_json,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(source_key) DO UPDATE SET
                    total_tests = excluded.total_tests,
                    successful_tests = excluded.successful_tests,
                    total_response_time_ms = excluded.total_response_time_ms,
                    last_success_time = excluded.last_success_time,
                    last_failure_time = excluded.last_failure_time,
                    last_available_time = excluded.last_available_time,
                    consecutive_failures = excluded.consecutive_failures,
                    auto_degraded = excluded.auto_degraded,
                    recent_results_json = excluded.recent_results_json,
                    updated_at = excluded.updated_at",
                params![
                    source_key,
                    total_tests,
                    successful_tests,
                    total_response_time_ms,
                    last_success_time,
                    last_failure_time,
                    last_available_time,
                    perf.consecutive_failures,
                    if perf.auto_degraded { 1 } else { 0 },
                    recent_results_json,
                    updated_at,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_all_stats(&self) -> Vec<SourceStats> {
        let performances = self.performances.lock().unwrap();
        let mut stats: Vec<SourceStats> = performances
            .iter()
            .map(|(key, perf)| perf.to_stats(key.clone()))
            .collect();

        stats.sort_by(|a, b| {
            let a_time = if a.avg_response_time_ms == 0 {
                u64::MAX
            } else {
                a.avg_response_time_ms
            };
            let b_time = if b.avg_response_time_ms == 0 {
                u64::MAX
            } else {
                b.avg_response_time_ms
            };

            a.auto_degraded
                .cmp(&b.auto_degraded)
                .then(a_time.cmp(&b_time))
                .then_with(|| b.success_rate.partial_cmp(&a.success_rate).unwrap())
                .then_with(|| a.source_key.cmp(&b.source_key))
        });

        stats
    }

    pub fn get_source_stats(&self, source_key: &str) -> Option<SourceStats> {
        let performances = self.performances.lock().unwrap();
        performances
            .get(source_key)
            .map(|perf| perf.to_stats(source_key.to_string()))
    }

    pub fn rank_sources(&self, source_keys: Vec<String>) -> Vec<String> {
        let performances = self.performances.lock().unwrap();
        let mut sources_with_scores: Vec<(String, f64)> = source_keys
            .into_iter()
            .map(|key| {
                let score = if let Some(perf) = performances.get(&key) {
                    if perf.auto_degraded
                        || perf.consecutive_failures >= self.max_consecutive_failures
                    {
                        -1000.0
                    } else {
                        let recent_rate = perf.recent_success_rate();
                        let overall_rate = perf.success_rate();
                        let avg_time = perf.avg_response_time();

                        let rate_score = recent_rate * 0.5 + overall_rate * 0.3;
                        let time_score = if avg_time == 0 {
                            20.0
                        } else if avg_time > 5000 {
                            0.0
                        } else {
                            ((5000 - avg_time) as f64 / 5000.0) * 20.0
                        };

                        rate_score + time_score
                    }
                } else {
                    50.0
                };

                (key, score)
            })
            .collect();

        sources_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sources_with_scores
            .into_iter()
            .map(|(key, _)| key)
            .collect()
    }

    pub fn should_skip_source(&self, source_key: &str) -> bool {
        let performances = self.performances.lock().unwrap();
        performances
            .get(source_key)
            .map(|perf| perf.auto_degraded)
            .unwrap_or(false)
    }

    pub fn has_stats(&self, source_key: &str) -> bool {
        let performances = self.performances.lock().unwrap();
        performances
            .get(source_key)
            .map(|perf| perf.total_tests > 0)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn clear_source_stats(&self, source_key: &str) {
        let mut performances = self.performances.lock().unwrap();
        performances.remove(source_key);
    }

    #[allow(dead_code)]
    pub fn clear_source_stats_persisted(&self, db: &Db, source_key: &str) -> Result<(), String> {
        self.clear_source_stats(source_key);
        db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM source_intelligence_stats WHERE source_key = ?1",
                params![source_key],
            )?;
            Ok(())
        })
    }

    pub fn clear_all_stats(&self) {
        let mut performances = self.performances.lock().unwrap();
        performances.clear();
    }

    pub fn clear_all_stats_persisted(&self, db: &Db) -> Result<(), String> {
        self.clear_all_stats();
        db.with_conn(|conn| {
            conn.execute("DELETE FROM source_intelligence_stats", [])?;
            Ok(())
        })
    }

    #[allow(dead_code)]
    pub fn reset_consecutive_failures(&self, source_key: &str) {
        let mut performances = self.performances.lock().unwrap();
        if let Some(perf) = performances.get_mut(source_key) {
            perf.consecutive_failures = 0;
            perf.auto_degraded = false;
        }
    }
}

#[tauri::command]
pub fn record_source_test(
    result: SourceTestResult,
    manager: State<'_, SourceIntelligenceManager>,
    db: State<'_, Db>,
) -> Result<(), String> {
    manager.record_test_result_persisted(&db, result)
}

#[tauri::command]
pub fn get_all_source_stats(
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Vec<SourceStats>, String> {
    Ok(manager.get_all_stats())
}

#[tauri::command]
pub fn get_source_stats(
    source_key: String,
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Option<SourceStats>, String> {
    Ok(manager.get_source_stats(&source_key))
}

#[tauri::command]
pub fn rank_sources(
    source_keys: Vec<String>,
    manager: State<'_, SourceIntelligenceManager>,
) -> Result<Vec<String>, String> {
    Ok(manager.rank_sources(source_keys))
}

#[tauri::command]
pub fn clear_all_source_stats(
    manager: State<'_, SourceIntelligenceManager>,
    db: State<'_, Db>,
) -> Result<(), String> {
    manager.clear_all_stats_persisted(&db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Db {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE video_sources (
                source_key TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                api TEXT NOT NULL,
                detail TEXT NOT NULL DEFAULT '',
                from_type TEXT NOT NULL DEFAULT 'custom',
                disabled INTEGER NOT NULL DEFAULT 0,
                is_adult INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE source_intelligence_stats (
                source_key TEXT PRIMARY KEY REFERENCES video_sources(source_key) ON DELETE CASCADE,
                total_tests INTEGER NOT NULL DEFAULT 0,
                successful_tests INTEGER NOT NULL DEFAULT 0,
                total_response_time_ms INTEGER NOT NULL DEFAULT 0,
                last_success_time INTEGER,
                last_failure_time INTEGER,
                last_available_time INTEGER,
                consecutive_failures INTEGER NOT NULL DEFAULT 0,
                auto_degraded INTEGER NOT NULL DEFAULT 0,
                recent_results_json TEXT NOT NULL DEFAULT '[]',
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .expect("init source intelligence schema");
        Db::new(conn)
    }

    fn seed_source(db: &Db, source_key: &str) {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO video_sources (source_key, name, api, detail, from_type, disabled, is_adult, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '', 'custom', 0, 0, 0, 1, 1)",
                params![source_key, source_key, format!("https://{}.example.com", source_key)],
            )?;
            Ok(())
        })
        .unwrap();
    }

    fn make_result(
        source_key: &str,
        success: bool,
        response_time_ms: u64,
        timestamp: u64,
    ) -> SourceTestResult {
        SourceTestResult {
            source_key: source_key.to_string(),
            success,
            response_time_ms,
            error_reason: if success {
                None
            } else {
                Some("timeout".to_string())
            },
            timestamp,
        }
    }

    #[test]
    fn test_source_performance_record() {
        let mut perf = SourcePerformance::new();

        perf.record_test(make_result("source1", true, 100, 1000), 3);
        assert_eq!(perf.total_tests, 1);
        assert_eq!(perf.successful_tests, 1);
        assert_eq!(perf.consecutive_failures, 0);

        perf.record_test(make_result("source1", false, 0, 1100), 3);
        assert_eq!(perf.total_tests, 2);
        assert_eq!(perf.successful_tests, 1);
        assert_eq!(perf.consecutive_failures, 1);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut perf = SourcePerformance::new();
        perf.record_test(make_result("source1", true, 100, 1000), 3);
        perf.record_test(make_result("source1", true, 200, 1100), 3);
        perf.record_test(make_result("source1", false, 0, 1200), 3);

        assert_eq!(perf.success_rate(), 66.66666666666666);
        assert_eq!(perf.avg_response_time(), 150);
    }

    #[test]
    fn test_recent_success_rate() {
        let mut perf = SourcePerformance::new();
        for i in 0..10 {
            perf.record_test(make_result("source1", true, 100, 1000 + i), 3);
        }
        for i in 0..10 {
            perf.record_test(make_result("source1", false, 0, 2000 + i), 3);
        }

        assert_eq!(perf.recent_success_rate(), 50.0);
    }

    #[test]
    fn test_recent_results_limit() {
        let mut perf = SourcePerformance::new();
        for i in 0..25 {
            perf.record_test(make_result("source1", i % 2 == 0, 100, 1000 + i), 3);
        }

        assert_eq!(perf.recent_results.len(), 20);
        assert_eq!(perf.recent_results[0].timestamp, 1005);
    }

    #[test]
    fn test_source_ranking() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("source1", true, 100, 1));
        manager.record_test_result(make_result("source2", true, 500, 2));
        manager.record_test_result(make_result("source3", false, 0, 3));

        let ranked = manager.rank_sources(vec![
            "source1".to_string(),
            "source2".to_string(),
            "source3".to_string(),
        ]);

        assert_eq!(ranked[0], "source1");
    }

    #[test]
    fn test_ranking_with_no_history() {
        let manager = SourceIntelligenceManager::new();
        let ranked =
            manager.rank_sources(vec!["new_source1".to_string(), "new_source2".to_string()]);
        assert_eq!(ranked.len(), 2);
    }

    #[test]
    fn test_ranking_with_consecutive_failures() {
        let manager = SourceIntelligenceManager::new();
        for i in 0..3 {
            manager.record_test_result(make_result("source1", false, 0, 100 + i));
        }
        manager.record_test_result(make_result("source2", true, 200, 200));

        let ranked = manager.rank_sources(vec!["source1".to_string(), "source2".to_string()]);
        assert_eq!(ranked[0], "source2");
        assert_eq!(ranked[1], "source1");
    }

    #[test]
    fn test_should_skip_source() {
        let manager = SourceIntelligenceManager::new();
        for i in 0..3 {
            manager.record_test_result(make_result("bad_source", false, 0, 100 + i));
        }

        assert!(manager.should_skip_source("bad_source"));
        assert!(!manager.should_skip_source("unknown_source"));
    }

    #[test]
    fn test_consecutive_failures_reset_on_success() {
        let manager = SourceIntelligenceManager::new();
        for i in 0..2 {
            manager.record_test_result(make_result("source1", false, 0, 100 + i));
        }

        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 2);

        manager.record_test_result(make_result("source1", true, 100, 200));
        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 0);
        assert!(!stats.auto_degraded);
    }

    #[test]
    fn test_reset_consecutive_failures() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("source1", false, 0, 100));
        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 1);

        manager.reset_consecutive_failures("source1");
        let stats = manager.get_source_stats("source1").unwrap();
        assert_eq!(stats.consecutive_failures, 0);
    }

    #[test]
    fn test_clear_source_stats() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("source1", true, 100, 100));
        assert!(manager.get_source_stats("source1").is_some());

        manager.clear_source_stats("source1");
        assert!(manager.get_source_stats("source1").is_none());
    }

    #[test]
    fn test_clear_all_stats() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("source1", true, 100, 100));
        manager.record_test_result(make_result("source2", true, 200, 200));

        assert_eq!(manager.get_all_stats().len(), 2);
        manager.clear_all_stats();
        assert_eq!(manager.get_all_stats().len(), 0);
    }

    #[test]
    fn test_get_all_stats() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("source1", true, 100, 100));
        manager.record_test_result(make_result("source2", false, 0, 200));

        let all_stats = manager.get_all_stats();
        assert_eq!(all_stats.len(), 2);

        let source1_stats = all_stats
            .iter()
            .find(|s| s.source_key == "source1")
            .unwrap();
        assert_eq!(source1_stats.success_rate, 100.0);

        let source2_stats = all_stats
            .iter()
            .find(|s| s.source_key == "source2")
            .unwrap();
        assert_eq!(source2_stats.success_rate, 0.0);
    }

    #[test]
    fn test_response_time_scoring() {
        let manager = SourceIntelligenceManager::new();
        manager.record_test_result(make_result("fast_source", true, 100, 100));
        manager.record_test_result(make_result("slow_source", true, 4000, 200));

        let ranked =
            manager.rank_sources(vec!["slow_source".to_string(), "fast_source".to_string()]);
        assert_eq!(ranked[0], "fast_source");
    }

    #[test]
    fn test_to_stats_conversion() {
        let mut perf = SourcePerformance::new();
        perf.record_test(make_result("test_source", true, 100, 100), 3);
        perf.record_test(make_result("test_source", true, 200, 200), 3);
        perf.record_test(make_result("test_source", false, 0, 300), 3);

        let stats = perf.to_stats("test_source".to_string());
        assert_eq!(stats.source_key, "test_source");
        assert_eq!(stats.total_tests, 3);
        assert_eq!(stats.successful_tests, 2);
        assert_eq!(stats.failed_tests, 1);
        assert_eq!(stats.success_rate, 66.66666666666666);
        assert_eq!(stats.avg_response_time_ms, 150);
        assert_eq!(stats.consecutive_failures, 1);
        assert_eq!(stats.last_available_time, Some(200));
        assert_eq!(stats.recent_results.len(), 3);
    }

    #[test]
    fn test_persist_and_load_source_stats() {
        let db = setup_test_db();
        seed_source(&db, "source1");
        let manager = SourceIntelligenceManager::new();

        manager
            .record_test_result_persisted(&db, make_result("source1", true, 123, 1000))
            .unwrap();
        manager
            .record_test_result_persisted(&db, make_result("source1", false, 0, 1005))
            .unwrap();

        let reloaded = SourceIntelligenceManager::new();
        reloaded.load_from_db(&db).unwrap();

        let stats = reloaded.get_source_stats("source1").unwrap();
        assert_eq!(stats.total_tests, 2);
        assert_eq!(stats.successful_tests, 1);
        assert_eq!(stats.avg_response_time_ms, 123);
        assert_eq!(stats.last_available_time, Some(1000));
        assert_eq!(stats.last_failure_time, Some(1005));
        assert_eq!(stats.recent_results.len(), 2);
        assert_eq!(
            stats.recent_results[1].error_reason.as_deref(),
            Some("timeout")
        );
    }

    #[test]
    fn test_clear_all_stats_persisted() {
        let db = setup_test_db();
        seed_source(&db, "source1");
        let manager = SourceIntelligenceManager::new();

        manager
            .record_test_result_persisted(&db, make_result("source1", true, 123, 1000))
            .unwrap();
        manager.clear_all_stats_persisted(&db).unwrap();

        let reloaded = SourceIntelligenceManager::new();
        reloaded.load_from_db(&db).unwrap();
        assert!(reloaded.get_all_stats().is_empty());
    }

    #[test]
    fn test_ensure_source_persisted_creates_empty_stats() {
        let db = setup_test_db();
        seed_source(&db, "source1");
        let manager = SourceIntelligenceManager::new();

        manager
            .ensure_source_persisted(&db, "source1".to_string())
            .unwrap();

        let reloaded = SourceIntelligenceManager::new();
        reloaded.load_from_db(&db).unwrap();

        let stats = reloaded.get_source_stats("source1").unwrap();
        assert_eq!(stats.total_tests, 0);
        assert_eq!(stats.avg_response_time_ms, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert!(!stats.auto_degraded);
    }
}
