/// 统计分析模块
///
/// 功能：
/// 1. 用户行为统计（观看时长、偏好分析）
/// 2. 热门内容排行榜
/// 3. 观看趋势分析
/// 4. 数据报表生成
/// 5. 时间段分析
use crate::db::db_client::Db;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// 用户行为统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorStats {
    pub total_watch_time: i64,             // 总观看时长（秒）
    pub total_videos_watched: i64,         // 观看视频总数
    pub total_favorites: i64,              // 收藏总数
    pub total_searches: i64,               // 搜索总数
    pub avg_watch_time: f64,               // 平均观看时长
    pub completion_rate: f64,              // 完成率（看完的比例）
    pub most_active_hour: Option<u8>,      // 最活跃时段
    pub favorite_category: Option<String>, // 最喜欢的分类
}

/// 热门内容项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularItem {
    pub title: String,
    pub source_name: String,
    pub year: String,
    pub cover: String,
    pub play_count: i64,
    pub favorite_count: i64,
    pub popularity_score: f64,
}

/// 观看趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchTrend {
    pub date: String,        // 日期（YYYY-MM-DD）
    pub watch_count: i64,    // 观看次数
    pub watch_duration: i64, // 观看时长（秒）
    pub unique_videos: i64,  // 不同视频数
}

/// 分类统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub watch_count: i64,
    pub watch_duration: i64,
    pub percentage: f64,
}

/// 时段统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: u8,          // 小时（0-23）
    pub watch_count: i64,  // 观看次数
    pub avg_duration: f64, // 平均时长
}

/// 统计分析器
pub struct AnalyticsEngine {
    cache: Arc<Mutex<AnalyticsCache>>,
    cache_ttl: Duration,
}

/// 统计缓存
#[derive(Debug, Clone)]
struct AnalyticsCache {
    user_stats: Option<(UserBehaviorStats, Instant)>,
    popular_items: Option<(Vec<PopularItem>, Instant)>,
    watch_trends: Option<(Vec<WatchTrend>, Instant)>,
    category_stats: Option<(Vec<CategoryStats>, Instant)>,
    hourly_stats: Option<(Vec<HourlyStats>, Instant)>,
}

impl AnalyticsCache {
    fn new() -> Self {
        Self {
            user_stats: None,
            popular_items: None,
            watch_trends: None,
            category_stats: None,
            hourly_stats: None,
        }
    }
}

impl AnalyticsEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(AnalyticsCache::new())),
            cache_ttl: Duration::from_secs(600), // 10 分钟缓存
        }
    }

    /// 获取用户行为统计
    pub fn get_user_behavior_stats(&self, db: &Db) -> Result<UserBehaviorStats, String> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some((stats, timestamp)) = &cache.user_stats {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        // 从数据库查询
        let total_videos_watched: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
        })?;

        let total_favorites: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0))
        })?;

        let total_searches: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
        })?;

        // 计算总观看时长和完成率
        let (total_watch_time, completed_count): (i64, i64) = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    COALESCE(SUM(play_time), 0) as total_time,
                    COUNT(CASE WHEN play_time >= total_time * 0.9 THEN 1 END) as completed
                 FROM play_records
                 WHERE total_time > 0",
            )?;

            stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        })?;

        let avg_watch_time = if total_videos_watched > 0 {
            total_watch_time as f64 / total_videos_watched as f64
        } else {
            0.0
        };

        let completion_rate = if total_videos_watched > 0 {
            completed_count as f64 / total_videos_watched as f64
        } else {
            0.0
        };

        // 计算最活跃时段
        let most_active_hour = self.calculate_most_active_hour(db)?;

        // 计算最喜欢的分类
        let favorite_category = self.calculate_favorite_category(db)?;

        let stats = UserBehaviorStats {
            total_watch_time,
            total_videos_watched,
            total_favorites,
            total_searches,
            avg_watch_time,
            completion_rate,
            most_active_hour,
            favorite_category,
        };

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.user_stats = Some((stats.clone(), Instant::now()));
        }

        Ok(stats)
    }

    /// 计算最活跃时段
    fn calculate_most_active_hour(&self, db: &Db) -> Result<Option<u8>, String> {
        let result: Option<i64> = db
            .with_conn(|conn| {
                let result = conn.query_row(
                    "SELECT strftime('%H', datetime(save_time, 'unixepoch')) as hour
                 FROM play_records
                 GROUP BY hour
                 ORDER BY COUNT(*) DESC
                 LIMIT 1",
                    [],
                    |row| row.get(0),
                );
                Ok(result.ok())
            })?
            .flatten();

        Ok(result.and_then(|h| u8::try_from(h).ok()))
    }

    /// 计算最喜欢的分类
    fn calculate_favorite_category(&self, db: &Db) -> Result<Option<String>, String> {
        // 简单实现：基于搜索历史中的关键词
        let result: Option<String> = db
            .with_conn(|conn| {
                let result = conn.query_row(
                    "SELECT keyword
                 FROM search_history
                 GROUP BY keyword
                 ORDER BY COUNT(*) DESC
                 LIMIT 1",
                    [],
                    |row| row.get(0),
                );
                Ok(result.ok())
            })?
            .flatten();

        Ok(result)
    }

    /// 获取热门内容排行榜
    pub fn get_popular_items(&self, db: &Db, limit: usize) -> Result<Vec<PopularItem>, String> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some((items, timestamp)) = &cache.popular_items {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(items.iter().take(limit).cloned().collect());
                }
            }
        }

        // 从数据库查询
        let items: Vec<PopularItem> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    p.title,
                    p.source_name,
                    p.year,
                    p.cover,
                    COUNT(DISTINCT p.key) as play_count,
                    COUNT(DISTINCT f.key) as favorite_count
                 FROM play_records p
                 LEFT JOIN favorites f ON p.title = f.title
                 GROUP BY p.title
                 ORDER BY play_count DESC, favorite_count DESC
                 LIMIT ?1",
            )?;

            let rows = stmt
                .query_map([limit as i64], |row| {
                    let play_count: i64 = row.get(4)?;
                    let favorite_count: i64 = row.get(5)?;

                    // 计算热度分数：播放次数 * 0.7 + 收藏次数 * 0.3
                    let popularity_score =
                        (play_count as f64 * 0.7) + (favorite_count as f64 * 0.3);

                    Ok(PopularItem {
                        title: row.get(0)?,
                        source_name: row.get(1)?,
                        year: row.get(2)?,
                        cover: row.get(3)?,
                        play_count,
                        favorite_count,
                        popularity_score,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.popular_items = Some((items.clone(), Instant::now()));
        }

        Ok(items)
    }

    /// 获取观看趋势（最近 N 天）
    pub fn get_watch_trends(&self, db: &Db, days: i64) -> Result<Vec<WatchTrend>, String> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some((trends, timestamp)) = &cache.watch_trends {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(trends.clone());
                }
            }
        }

        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (days * 24 * 3600);

        let trends: Vec<WatchTrend> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    date(save_time, 'unixepoch') as date,
                    COUNT(*) as watch_count,
                    COALESCE(SUM(play_time), 0) as watch_duration,
                    COUNT(DISTINCT title) as unique_videos
                 FROM play_records
                 WHERE save_time > ?1
                 GROUP BY date
                 ORDER BY date DESC",
            )?;

            let rows = stmt
                .query_map([cutoff_time], |row| {
                    Ok(WatchTrend {
                        date: row.get(0)?,
                        watch_count: row.get(1)?,
                        watch_duration: row.get(2)?,
                        unique_videos: row.get(3)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.watch_trends = Some((trends.clone(), Instant::now()));
        }

        Ok(trends)
    }

    /// 获取分类统计
    pub fn get_category_stats(&self, db: &Db) -> Result<Vec<CategoryStats>, String> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some((stats, timestamp)) = &cache.category_stats {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        // 简单实现：基于标题关键词分类
        let total_count: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
        })?;

        if total_count == 0 {
            return Ok(Vec::new());
        }

        let categories = vec![
            ("电影", vec!["电影", "影片", "院线"]),
            ("电视剧", vec!["电视剧", "剧集", "美剧", "韩剧", "日剧"]),
            ("动漫", vec!["动漫", "动画", "番剧"]),
            ("综艺", vec!["综艺", "真人秀"]),
        ];

        let mut stats = Vec::new();

        for (category_name, keywords) in categories {
            let mut count = 0i64;
            let mut duration = 0i64;

            for keyword in keywords {
                let (c, d): (i64, i64) = db.with_conn(|conn| {
                    let mut stmt = conn.prepare(
                        "SELECT COUNT(*), COALESCE(SUM(play_time), 0)
                         FROM play_records
                         WHERE title LIKE ?1 OR search_title LIKE ?1",
                    )?;

                    let pattern = format!("%{}%", keyword);
                    stmt.query_row([&pattern], |row| Ok((row.get(0)?, row.get(1)?)))
                })?;

                count += c;
                duration += d;
            }

            if count > 0 {
                let percentage = (count as f64 / total_count as f64) * 100.0;
                stats.push(CategoryStats {
                    category: category_name.to_string(),
                    watch_count: count,
                    watch_duration: duration,
                    percentage,
                });
            }
        }

        // 按观看次数排序
        stats.sort_by(|a, b| b.watch_count.cmp(&a.watch_count));

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.category_stats = Some((stats.clone(), Instant::now()));
        }

        Ok(stats)
    }

    /// 获取时段统计
    pub fn get_hourly_stats(&self, db: &Db) -> Result<Vec<HourlyStats>, String> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some((stats, timestamp)) = &cache.hourly_stats {
                if timestamp.elapsed() < self.cache_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        let stats: Vec<HourlyStats> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    CAST(strftime('%H', datetime(save_time, 'unixepoch')) AS INTEGER) as hour,
                    COUNT(*) as watch_count,
                    AVG(play_time) as avg_duration
                 FROM play_records
                 GROUP BY hour
                 ORDER BY hour",
            )?;

            let rows = stmt
                .query_map([], |row| {
                    let hour: i64 = row.get(0)?;
                    Ok(HourlyStats {
                        hour: hour as u8,
                        watch_count: row.get(1)?,
                        avg_duration: row.get::<_, f64>(2).unwrap_or(0.0),
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.hourly_stats = Some((stats.clone(), Instant::now()));
        }

        Ok(stats)
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = AnalyticsCache::new();
    }

    /// 生成统计报表
    pub fn generate_report(&self, db: &Db) -> Result<AnalyticsReport, String> {
        Ok(AnalyticsReport {
            user_behavior: self.get_user_behavior_stats(db)?,
            popular_items: self.get_popular_items(db, 10)?,
            watch_trends: self.get_watch_trends(db, 30)?,
            category_stats: self.get_category_stats(db)?,
            hourly_stats: self.get_hourly_stats(db)?,
        })
    }
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 统计报表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub user_behavior: UserBehaviorStats,
    pub popular_items: Vec<PopularItem>,
    pub watch_trends: Vec<WatchTrend>,
    pub category_stats: Vec<CategoryStats>,
    pub hourly_stats: Vec<HourlyStats>,
}

// ========== Tauri 命令 ==========

/// 获取用户行为统计
#[tauri::command]
pub fn get_user_behavior_stats(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
) -> Result<UserBehaviorStats, String> {
    engine.get_user_behavior_stats(&db)
}

/// 获取热门内容排行榜
#[tauri::command]
pub fn get_popular_items(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
    limit: Option<usize>,
) -> Result<Vec<PopularItem>, String> {
    engine.get_popular_items(&db, limit.unwrap_or(10))
}

/// 获取观看趋势
#[tauri::command]
pub fn get_watch_trends(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
    days: Option<i64>,
) -> Result<Vec<WatchTrend>, String> {
    engine.get_watch_trends(&db, days.unwrap_or(30))
}

/// 获取分类统计
#[tauri::command]
pub fn get_category_stats(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
) -> Result<Vec<CategoryStats>, String> {
    engine.get_category_stats(&db)
}

/// 获取时段统计
#[tauri::command]
pub fn get_hourly_stats(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
) -> Result<Vec<HourlyStats>, String> {
    engine.get_hourly_stats(&db)
}

/// 生成完整统计报表
#[tauri::command]
pub fn generate_analytics_report(
    db: State<'_, Db>,
    engine: State<'_, AnalyticsEngine>,
) -> Result<AnalyticsReport, String> {
    engine.generate_report(&db)
}

/// 清除统计缓存
#[tauri::command]
pub fn clear_analytics_cache(engine: State<'_, AnalyticsEngine>) -> Result<(), String> {
    engine.clear_cache();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_engine_creation() {
        let engine = AnalyticsEngine::new();
        assert_eq!(engine.cache_ttl, Duration::from_secs(600));
    }

    #[test]
    fn test_analytics_cache_creation() {
        let cache = AnalyticsCache::new();
        assert!(cache.user_stats.is_none());
        assert!(cache.popular_items.is_none());
        assert!(cache.watch_trends.is_none());
        assert!(cache.category_stats.is_none());
        assert!(cache.hourly_stats.is_none());
    }

    #[test]
    fn test_user_behavior_stats_serialization() {
        let stats = UserBehaviorStats {
            total_watch_time: 3600,
            total_videos_watched: 10,
            total_favorites: 5,
            total_searches: 20,
            avg_watch_time: 360.0,
            completion_rate: 0.8,
            most_active_hour: Some(20),
            favorite_category: Some("电影".to_string()),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("3600"));
        assert!(json.contains("电影"));
    }

    #[test]
    fn test_popular_item_score_calculation() {
        let item = PopularItem {
            title: "Test".to_string(),
            source_name: "Source".to_string(),
            year: "2024".to_string(),
            cover: "".to_string(),
            play_count: 10,
            favorite_count: 5,
            popularity_score: (10.0 * 0.7) + (5.0 * 0.3),
        };

        assert_eq!(item.popularity_score, 8.5);
    }

    #[test]
    fn test_watch_trend_structure() {
        let trend = WatchTrend {
            date: "2024-01-01".to_string(),
            watch_count: 5,
            watch_duration: 1800,
            unique_videos: 3,
        };

        assert_eq!(trend.watch_count, 5);
        assert_eq!(trend.unique_videos, 3);
    }

    #[test]
    fn test_category_stats_percentage() {
        let stats = CategoryStats {
            category: "电影".to_string(),
            watch_count: 50,
            watch_duration: 18000,
            percentage: 50.0,
        };

        assert_eq!(stats.percentage, 50.0);
    }

    #[test]
    fn test_hourly_stats_range() {
        let stats = HourlyStats {
            hour: 20,
            watch_count: 10,
            avg_duration: 360.0,
        };

        assert!(stats.hour < 24);
        assert!(stats.avg_duration > 0.0);
    }

    #[test]
    fn test_analytics_report_structure() {
        let report = AnalyticsReport {
            user_behavior: UserBehaviorStats {
                total_watch_time: 3600,
                total_videos_watched: 10,
                total_favorites: 5,
                total_searches: 20,
                avg_watch_time: 360.0,
                completion_rate: 0.8,
                most_active_hour: Some(20),
                favorite_category: Some("电影".to_string()),
            },
            popular_items: vec![],
            watch_trends: vec![],
            category_stats: vec![],
            hourly_stats: vec![],
        };

        assert_eq!(report.user_behavior.total_videos_watched, 10);
    }

    #[test]
    fn test_clear_cache() {
        let engine = AnalyticsEngine::new();
        engine.clear_cache();

        let cache = engine.cache.lock().unwrap();
        assert!(cache.user_stats.is_none());
    }
}
