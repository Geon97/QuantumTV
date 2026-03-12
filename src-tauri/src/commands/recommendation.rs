/// 智能推荐引擎
///
/// 功能：
/// 1. 冷启动推荐：基于热门内容、豆瓣榜单
/// 2. 个性化推荐：基于用户观看历史和收藏
/// 3. 内容相似度推荐：使用 ContentAnalyzer 找到相似内容
/// 4. 多源数据融合：使用 DataFusion 融合和去重豆瓣、搜索、历史等多源数据
/// 5. 智能搜索推荐：基于用户偏好从豆瓣API获取新内容
/// 6. 混合推荐策略：根据用户历史数量动态调整
/// 7. 隐私保护：推荐原因不暴露给前端
use crate::commands::content_analyzer::{ContentAnalyzer, QualityFactors, VideoCategory, VideoMetadata};
use crate::commands::data_fusion::{ConflictResolution, DataFusion, DeduplicationStrategy};
use crate::db::db_client::Db;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;

/// 用户偏好画像
#[derive(Debug, Clone)]
struct UserProfile {
    watched_titles: Vec<String>,
    favorite_categories: Vec<VideoCategory>,
    favorite_years: Vec<String>,
}

/// 推荐项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationItem {
    pub title: String,
    pub source_name: String,
    pub year: String,
    pub cover: String,
    pub score: f64,
    #[serde(skip)]
    #[allow(dead_code)] // 不暴露给前端，但用于内部逻辑
    pub reason: RecommendationReason,
}

/// 推荐原因（内部使用，不暴露给前端）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RecommendationReason {
    #[default]
    Popular, // 热门内容
    SimilarToHistory, // 基于历史相似
    Favorite,         // 基于收藏
    CategoryBased,    // 分类推荐
    NewContent,       // 新内容
    ContentDiscovery, // 内容发现（全新内容）
}

/// 用户历史统计
#[derive(Debug, Clone)]
struct UserHistoryStats {
    total_records: usize,
    #[allow(dead_code)]
    favorite_count: usize,
    #[allow(dead_code)]
    search_count: usize,
    last_update: Instant,
}

/// 全局热度统计
#[derive(Debug, Clone)]
struct GlobalStats {
    popular_keywords: Vec<(String, u32)>,
    popular_titles: Vec<(String, u32)>,
    last_update: Instant,
}

/// 推荐引擎管理器
pub struct RecommendationEngine {
    user_stats_cache: Arc<Mutex<Option<UserHistoryStats>>>,
    global_stats_cache: Arc<Mutex<Option<GlobalStats>>>,
    recommendation_cache: Arc<Mutex<HashMap<String, (Vec<RecommendationItem>, Instant)>>>,
    cache_ttl: Duration,
    analyzer: ContentAnalyzer, // 内容分析器
    fusion: DataFusion,         // 数据融合器
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            user_stats_cache: Arc::new(Mutex::new(None)),
            global_stats_cache: Arc::new(Mutex::new(None)),
            recommendation_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 分钟缓存
            analyzer: ContentAnalyzer::new(),
            fusion: DataFusion::new(),
        }
    }

    /// 获取用户历史统计
    fn get_user_stats(&self, db: &Db) -> Result<UserHistoryStats, String> {
        // 检查缓存
        {
            let cache = self.user_stats_cache.lock().unwrap();
            if let Some(stats) = cache.as_ref() {
                if stats.last_update.elapsed() < self.cache_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        // 从数据库查询
        let total_records: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM play_records", [], |row| row.get(0))
        })?;

        let favorite_count: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM favorites", [], |row| row.get(0))
        })?;

        let search_count: i64 = db.with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
        })?;

        let stats = UserHistoryStats {
            total_records: total_records as usize,
            favorite_count: favorite_count as usize,
            search_count: search_count as usize,
            last_update: Instant::now(),
        };

        // 更新缓存
        {
            let mut cache = self.user_stats_cache.lock().unwrap();
            *cache = Some(stats.clone());
        }

        Ok(stats)
    }

    /// 获取全局热度统计
    fn get_global_stats(&self, db: &Db) -> Result<GlobalStats, String> {
        // 检查缓存
        {
            let cache = self.global_stats_cache.lock().unwrap();
            if let Some(stats) = cache.as_ref() {
                if stats.last_update.elapsed() < self.cache_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        // 统计热门搜索关键词（最近 30 天）
        let thirty_days_ago = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 30 * 24 * 3600) as i64;

        let popular_keywords: Vec<(String, u32)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT keyword, COUNT(*) as count
                 FROM search_history
                 WHERE save_time > ?1
                 GROUP BY keyword
                 ORDER BY count DESC
                 LIMIT 20",
            )?;

            let rows = stmt
                .query_map([thirty_days_ago], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        // 统计热门播放标题
        let popular_titles: Vec<(String, u32)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, COUNT(*) as count
                 FROM play_records
                 WHERE save_time > ?1
                 GROUP BY title
                 ORDER BY count DESC
                 LIMIT 20",
            )?;

            let rows = stmt
                .query_map([thirty_days_ago], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        let stats = GlobalStats {
            popular_keywords,
            popular_titles,
            last_update: Instant::now(),
        };

        // 更新缓存
        {
            let mut cache = self.global_stats_cache.lock().unwrap();
            *cache = Some(stats.clone());
        }

        Ok(stats)
    }

    /// 冷启动推荐：用户无历史时的推荐策略
    fn cold_start_recommendations(&self, db: &Db) -> Result<Vec<RecommendationItem>, String> {
        let mut recommendations = Vec::new();

        eprintln!("冷启动推荐：结合用户习惯和新内容");

        // 策略1：分析用户观影习惯，找到相似的新内容
        let history_videos = self.get_videos_from_history(db)?;
        let favorite_videos = self.get_videos_from_favorites(db)?;
        let search_videos = self.get_videos_from_search_history(db)?;

        eprintln!("  用户历史: {} 个观看, {} 个收藏, {} 个搜索",
            history_videos.len(), favorite_videos.len(), search_videos.len());

        // 提取用户偏好的年份和分类
        let mut preferred_years = Vec::new();
        let mut preferred_categories = Vec::new();

        for video in history_videos.iter().chain(favorite_videos.iter()) {
            if let Some(year) = video.year {
                preferred_years.push(year.to_string());
            }
            preferred_categories.push(video.category.clone());
        }

        eprintln!("  偏好年份: {:?}", preferred_years.iter().take(3).collect::<Vec<_>>());
        eprintln!("  偏好分类: {:?}", preferred_categories.iter().take(3).collect::<Vec<_>>());

        // 策略2：使用 find_similar_content 找到与用户历史相似的新内容
        let watched_titles: Vec<String> = history_videos.iter()
            .chain(favorite_videos.iter())
            .map(|v| v.title.clone())
            .collect();

        if !watched_titles.is_empty() {
            let similar_content = self.find_similar_content(db, &watched_titles)?;
            eprintln!("  通过相似度匹配找到 {} 个推荐", similar_content.len());
            recommendations.extend(similar_content);
        }

        // 策略3：从 content_pool 获取与用户偏好匹配的新内容
        let pool_content: Vec<(String, String, String, String, f64, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, source_name, year, cover, rating, category
                 FROM content_pool
                 WHERE title NOT IN (SELECT title FROM play_records)
                 AND title NOT IN (SELECT title FROM favorites)
                 ORDER BY popularity_score DESC, created_at DESC
                 LIMIT 50"
            )?;

            let rows: Vec<(String, String, String, String, f64, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2).unwrap_or_default(),
                        row.get(3).unwrap_or_default(),
                        row.get(4).unwrap_or(0.0),
                        row.get(5).unwrap_or_default(),
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        // eprintln!("  从 content_pool 获取 {} 条候选内容", pool_content.len());

        // 根据用户偏好对内容进行评分
        for (title, source, year, cover, rating, category) in pool_content {
            let mut score = if rating > 0.0 { rating } else { 7.0 };

            // 年份匹配加分
            if !year.is_empty() && preferred_years.contains(&year) {
                score += 1.5;
                eprintln!("    {} 年份匹配 +1.5", title);
            }

            // 分类匹配加分 - 智能分类识别
            let db_category = if !category.is_empty() {
                // 如果数据库有分类，直接映射
                match category.as_str() {
                    "Anime" | "动漫" | "动画" => crate::commands::content_analyzer::VideoCategory::Anime,
                    "Movie" | "电影" | "movie" => crate::commands::content_analyzer::VideoCategory::Movie,
                    "TvSeries" | "电视剧" | "剧集" | "tv" => crate::commands::content_analyzer::VideoCategory::TvSeries,
                    "Variety" | "综艺" | "show" => crate::commands::content_analyzer::VideoCategory::Variety,
                    "Documentary" | "纪录片" => crate::commands::content_analyzer::VideoCategory::Documentary,
                    _ => {
                        // 如果数据库分类不明确，使用 ContentAnalyzer 分析
                        self.analyzer.classify_video(&title, Some(&category))
                    }
                }
            } else {
                // 如果数据库没有分类，使用智能分析
                // 1. 先尝试从标题分析
                let analyzed_category = self.analyzer.classify_video(&title, None);

                // 2. 如果分析结果是 Unknown，尝试从来源判断
                if analyzed_category == crate::commands::content_analyzer::VideoCategory::Unknown {
                    // 根据来源判断
                    if source.contains("Bangumi") {
                        crate::commands::content_analyzer::VideoCategory::Anime
                    } else {
                        // 默认为电影
                        crate::commands::content_analyzer::VideoCategory::Movie
                    }
                } else {
                    analyzed_category
                }
            };

            if preferred_categories.contains(&db_category) {
                score += 2.0;
                eprintln!("    {} 分类匹配 +2.0 (分类: {:?})", title, db_category);
            }

            // 与用户历史内容计算相似度
            for history_video in history_videos.iter().take(5) {
                let similarity = self.analyzer.calculate_similarity(&title, &history_video.title);
                if similarity > 0.6 {
                    score += similarity * 1.5;
                    eprintln!("    {} 与 {} 相似度 {:.2} +{:.2}", title, history_video.title, similarity, similarity * 1.5);
                }
            }

            recommendations.push(RecommendationItem {
                title,
                source_name: source,
                year,
                cover,
                score: score.min(10.0),
                reason: RecommendationReason::NewContent,
            });
        }

        // 策略4：使用 get_incomplete_videos 找到用户可能感兴趣但未完成的内容
        // 注意：这些内容不会被推荐，但会用于分析用户偏好
        let incomplete_videos = self.get_incomplete_videos(db)?;
        eprintln!("  发现 {} 个未完成视频（用于偏好分析）", incomplete_videos.len());

        // 将未完成视频的分类和年份也加入偏好分析
        for video in incomplete_videos.iter() {
            preferred_categories.push(video.category.clone());
            if let Some(year) = video.year {
                preferred_years.push(year.to_string());
            }
        }

        eprintln!("  增强后偏好分类: {:?}", preferred_categories.iter().take(5).collect::<Vec<_>>());
        eprintln!("  增强后偏好年份: {:?}", preferred_years.iter().take(5).collect::<Vec<_>>());

        // 策略5：如果用户数据不足，使用全局热门统计
        if recommendations.len() < 10 {
            let global_stats = self.get_global_stats(db)?;
            eprintln!("  补充全局热门内容: {} 个关键词, {} 个标题",
                global_stats.popular_keywords.len(), global_stats.popular_titles.len());

            // 基于热门标题推荐（但排除用户已看过的）
            for (title, count) in global_stats.popular_titles.iter().take(5) {
                if !watched_titles.contains(title) {
                    let score = (*count as f64 / 10.0).min(8.0);
                    recommendations.push(RecommendationItem {
                        title: title.clone(),
                        source_name: "热门推荐".to_string(),
                        year: String::new(),
                        cover: String::new(),
                        score,
                        reason: RecommendationReason::Popular,
                    });
                }
            }
        }

        // 策略6：如果 content_pool 内容不足，从 image_cache 补充
        if recommendations.len() < 20 {
            let cache_content: Vec<(String, String, String, String)> = db.with_conn(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT DISTINCT title, source_name, year, url
                     FROM image_cache
                     WHERE title IS NOT NULL
                     AND title != ''
                     AND title NOT IN (SELECT title FROM play_records)
                     AND title NOT IN (SELECT title FROM favorites)
                     ORDER BY access_count DESC, last_accessed DESC
                     LIMIT 20"
                )?;

                let rows: Vec<(String, String, String, String)> = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1).unwrap_or_else(|_| "未知来源".to_string()),
                            row.get(2).unwrap_or_default(),
                            row.get(3)?,
                        ))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                Ok(rows)
            })?;

            eprintln!("  从 image_cache 补充 {} 条内容", cache_content.len());

            for (title, source, year, cover) in cache_content {
                if !recommendations.iter().any(|r| r.title == title) {
                    recommendations.push(RecommendationItem {
                        title,
                        source_name: source,
                        year,
                        cover,
                        score: 6.5,
                        reason: RecommendationReason::NewContent,
                    });
                }
            }
        }

        // 策略7：如果仍然没有内容，返回预设推荐
        if recommendations.is_empty() {
            eprintln!("  没有找到新内容，使用预设推荐");
            recommendations = self.get_default_recommendations();
        }

        Ok(recommendations)
    }

    /// 预设推荐（当没有任何数据时）
    fn get_default_recommendations(&self) -> Vec<RecommendationItem> {
        vec![
            RecommendationItem {
                title: "肖申克的救赎".to_string(),
                source_name: "经典推荐".to_string(),
                year: "1994".to_string(),
                cover: "".to_string(),
                score: 9.7,
                reason: RecommendationReason::CategoryBased,
            },
            RecommendationItem {
                title: "霸王别姬".to_string(),
                source_name: "经典推荐".to_string(),
                year: "1993".to_string(),
                cover: "".to_string(),
                score: 9.6,
                reason: RecommendationReason::CategoryBased,
            },
            RecommendationItem {
                title: "阿甘正传".to_string(),
                source_name: "经典推荐".to_string(),
                year: "1994".to_string(),
                cover: "".to_string(),
                score: 9.5,
                reason: RecommendationReason::CategoryBased,
            },
            RecommendationItem {
                title: "泰坦尼克号".to_string(),
                source_name: "经典推荐".to_string(),
                year: "1997".to_string(),
                cover: "".to_string(),
                score: 9.4,
                reason: RecommendationReason::CategoryBased,
            },
            RecommendationItem {
                title: "千与千寻".to_string(),
                source_name: "经典推荐".to_string(),
                year: "2001".to_string(),
                cover: "".to_string(),
                score: 9.4,
                reason: RecommendationReason::CategoryBased,
            },
        ]
    }

    /// 个性化推荐：基于用户历史推荐相似的新内容
    fn personalized_recommendations(&self, db: &Db) -> Result<Vec<RecommendationItem>, String> {
        let mut recommendations = Vec::new();

        eprintln!("个性化推荐：基于用户观影习惯");

        // 1. 获取用户观影数据
        let history_videos = self.get_videos_from_history(db)?;
        let favorite_videos = self.get_videos_from_favorites(db)?;
        let search_videos = self.get_videos_from_search_history(db)?;

        eprintln!("  历史: {}, 收藏: {}, 搜索: {}",
            history_videos.len(), favorite_videos.len(), search_videos.len());

        // 2. 合并所有用户偏好视频
        let mut all_user_videos = Vec::new();
        all_user_videos.extend(history_videos);
        all_user_videos.extend(favorite_videos);
        all_user_videos.extend(search_videos);

        if all_user_videos.is_empty() {
            eprintln!("  没有用户数据，跳过个性化推荐");
            return Ok(recommendations);
        }

        // 3. 从 content_pool 获取候选内容
        let pool_content: Vec<(String, String, String, String, f64, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, source_name, year, cover, rating, category
                 FROM content_pool
                 WHERE title NOT IN (SELECT title FROM play_records)
                 AND title NOT IN (SELECT title FROM favorites)
                 ORDER BY popularity_score DESC
                 LIMIT 100"
            )?;

            let rows: Vec<(String, String, String, String, f64, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2).unwrap_or_default(),
                        row.get(3).unwrap_or_default(),
                        row.get(4).unwrap_or(0.0),
                        row.get(5).unwrap_or_default(),
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        eprintln!("  候选内容: {} 个", pool_content.len());

        // 4. 使用 ContentAnalyzer 计算相似度
        for (title, source, year, cover, rating, _category) in pool_content {
            let candidate = self.analyzer.analyze_video(&title, Some(&year), Some(&cover), None, &source);

            let mut max_similarity = 0.0;
            let mut similar_to = String::new();

            // 与用户所有视频计算相似度
            for user_video in &all_user_videos {
                let similarity = self.analyzer.calculate_similarity(&candidate.title, &user_video.title);
                if similarity > max_similarity {
                    max_similarity = similarity;
                    similar_to = user_video.title.clone();
                }
            }

            // 只推荐相似度高的内容
            if max_similarity > 0.5 {
                let score = (rating * 0.5 + max_similarity * 10.0 * 0.5).min(10.0);

                eprintln!("    {} 与 {} 相似度 {:.2}, 分数 {:.2}",
                    title, similar_to, max_similarity, score);

                recommendations.push(RecommendationItem {
                    title,
                    source_name: source,
                    year,
                    cover,
                    score,
                    reason: RecommendationReason::SimilarToHistory,
                });
            }
        }

        eprintln!("  生成 {} 个个性化推荐", recommendations.len());

        // 按分数排序
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// 基于用户观看历史查找相似但未看过的内容
    fn find_similar_content(
        &self,
        db: &Db,
        watched_titles: &[String],
    ) -> Result<Vec<RecommendationItem>, String> {
        let mut similar_recommendations = Vec::new();

        // 获取所有可能的候选内容（排除用户已看过的）
        let candidates: Vec<(String, String, String, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT title, source_name, year, cover
                 FROM (
                     SELECT title, source_name, year, cover FROM content_pool
                     WHERE title NOT IN (SELECT title FROM play_records)
                     AND title NOT IN (SELECT title FROM favorites)
                     UNION
                     SELECT title, source_name, year, url as cover FROM image_cache
                     WHERE title IS NOT NULL
                     AND title != ''
                     AND title NOT IN (SELECT title FROM play_records)
                     AND title NOT IN (SELECT title FROM favorites)
                 )
                 ORDER BY title
                 LIMIT 100"
            )?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1).unwrap_or_default(),
                        row.get(2).unwrap_or_default(),
                        row.get(3).unwrap_or_default(),
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        if candidates.is_empty() {
            return Ok(similar_recommendations);
        }

        let candidate_titles: Vec<String> = candidates.iter().map(|(title, _, _, _)| title.clone()).collect();

        // 对用户观看过的每个视频找相似的
        for user_title in watched_titles.iter().take(10) {
            let similar_videos = self.analyzer.find_similar_videos(
                user_title,
                &candidate_titles,
                0.3, // 相似度阈值
            );

            for (candidate_title, similarity) in similar_videos.into_iter().take(5) {
                // 找到对应的完整信息
                if let Some((_, source, year, cover)) = candidates.iter().find(|(t, _, _, _)| t == &candidate_title) {
                    let has_year = self.analyzer.extract_year(&candidate_title).is_some();
                    let has_cover = !cover.is_empty();

                    let quality_factors = QualityFactors {
                        has_year,
                        has_cover,
                        has_description: false,
                        title_length: candidate_title.len(),
                        source_reliability: 0.7,
                        metadata_completeness: if has_year && has_cover { 0.67 } else if has_year || has_cover { 0.33 } else { 0.0 },
                    };

                    let quality_score = self.analyzer.calculate_quality_score(&quality_factors);

                    // 相似度越高，分数加成越多
                    let final_score = quality_score * (1.0 + similarity * 0.5);

                    similar_recommendations.push(RecommendationItem {
                        title: candidate_title.clone(),
                        source_name: source.clone(),
                        year: year.clone(),
                        cover: cover.clone(),
                        score: final_score,
                        reason: RecommendationReason::SimilarToHistory,
                    });
                }
            }
        }

        // 按分数排序，取前10个
        similar_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        similar_recommendations.truncate(10);

        Ok(similar_recommendations)
    }

    /// 基于分类的推荐
    fn category_based_recommendations(&self, db: &Db) -> Result<Vec<RecommendationItem>, String> {
        let mut recommendations = Vec::new();

        eprintln!("分类推荐：基于用户偏好分类");

        // 1. 获取用户观影数据
        let history_videos = self.get_videos_from_history(db)?;
        let favorite_videos = self.get_videos_from_favorites(db)?;

        if history_videos.is_empty() && favorite_videos.is_empty() {
            eprintln!("  没有用户数据，跳过分类推荐");
            return Ok(recommendations);
        }

        // 2. 统计用户最喜欢的分类
        let mut category_counts = std::collections::HashMap::new();
        for video in history_videos.iter().chain(favorite_videos.iter()) {
            let category_name = format!("{:?}", video.category);
            *category_counts.entry(category_name).or_insert(0) += 1;
        }

        // 按出现次数排序
        let mut sorted_categories: Vec<_> = category_counts.into_iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(&a.1));

        eprintln!("  用户偏好分类: {:?}", sorted_categories.iter().take(3).collect::<Vec<_>>());

        // 3. 为每个热门分类推荐内容
        for (category_name, count) in sorted_categories.iter().take(3) {
            eprintln!("  处理分类: {} (出现 {} 次)", category_name, count);

            // 从 content_pool 和 image_cache 获取该分类的新内容
            let category_items: Vec<(String, String, String, String, f64)> = db.with_conn(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT DISTINCT title, source_name, year, cover, rating
                     FROM (
                         SELECT title, source_name, year, cover, rating FROM content_pool
                         WHERE category = ?1
                         AND title NOT IN (SELECT title FROM play_records)
                         AND title NOT IN (SELECT title FROM favorites)
                         UNION
                         SELECT title, source_name, year, url as cover, rating FROM image_cache
                         WHERE category = ?1
                         AND title IS NOT NULL
                         AND title != ''
                         AND title NOT IN (SELECT title FROM play_records)
                         AND title NOT IN (SELECT title FROM favorites)
                     )
                     ORDER BY rating DESC
                     LIMIT 20"
                )?;

                let rows = stmt
                    .query_map([category_name], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1).unwrap_or_default(),
                            row.get(2).unwrap_or_default(),
                            row.get(3).unwrap_or_default(),
                            row.get(4).unwrap_or(7.0),
                        ))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                Ok(rows)
            })?;

            eprintln!("    找到 {} 个 {} 分类的内容", category_items.len(), category_name);

            // 添加到推荐列表
            for (title, source, year, cover, rating) in category_items {
                let score = rating.max(6.0); // 最低分数 6.0

                recommendations.push(RecommendationItem {
                    title,
                    source_name: source,
                    year,
                    cover,
                    score,
                    reason: RecommendationReason::CategoryBased,
                });
            }
        }

        eprintln!("  生成 {} 个分类推荐", recommendations.len());

        // 按分数排序，取前20个
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// 智能推荐：使用 DataFusion 融合多源数据，发现新内容
    async fn intelligent_recommendations(&self, db: &Db) -> Result<Vec<RecommendationItem>, String> {
        // 第一步：分析用户偏好（从历史数据中提取）
        let user_profile = self.build_user_profile(db)?;
        eprintln!("用户画像: 已观看 {} 个视频", user_profile.watched_titles.len());
        eprintln!("  偏好分类: {:?}", user_profile.favorite_categories.iter().take(3).collect::<Vec<_>>());
        eprintln!("  偏好年份: {:?}", user_profile.favorite_years.iter().take(3).collect::<Vec<_>>());

        // 第二步：从全局数据库中发现新内容（只包括用户未接触过的内容）
        let mut discovered_videos = self.discover_global_content(db, &user_profile)?;
        eprintln!("从 content_pool 发现 {} 个新内容", discovered_videos.len());

        let from_pool = self.discover_from_content_pool(db, &user_profile)?;
        eprintln!("从 content_pool (策略2) 发现 {} 个新内容", from_pool.len());
        discovered_videos.extend(from_pool);

        let from_cache = self.discover_from_image_cache(db, &user_profile)?;
        eprintln!("从 image_cache 发现 {} 个新内容", from_cache.len());
        discovered_videos.extend(from_cache);

        eprintln!("总共发现 {} 个待推荐内容", discovered_videos.len());

        // 第三步：使用 ContentAnalyzer 批量分析这些新内容
        let analyzed_videos = self.analyzer.batch_analyze(discovered_videos);

        // 第四步：根据用户偏好对分析后的视频进行加权
        let mut weighted_videos = Vec::new();
        for mut video in analyzed_videos {
            let mut bonus_score = 0.0;

            // 分类匹配加分
            if user_profile.favorite_categories.contains(&video.category) {
                bonus_score += 2.0;
                eprintln!("  {} 分类匹配用户偏好 +2.0", video.title);
            }

            // 年份匹配加分
            if let Some(year) = video.year {
                let year_str = year.to_string();
                if user_profile.favorite_years.contains(&year_str) {
                    bonus_score += 1.5;
                    eprintln!("  {} 年份匹配用户偏好 +1.5", video.title);
                }
            }

            // 应用加分
            video.quality_score = (video.quality_score + bonus_score).min(10.0);
            weighted_videos.push(video);
        }

        // 第五步：使用 DataFusion 标准化数据
        let normalized_videos = self.fusion.normalize_data(weighted_videos);

        // 第六步：使用 DataFusion 增强元数据（提取标签、分类、年份等）
        let enhanced_videos = self.fusion.batch_enhance(normalized_videos);

        // 第七步：使用 DataFusion 清洗数据（移除低质量内容）
        let cleaned_videos = self.fusion.clean_data(enhanced_videos, 5.0);

        // 第八步：查找重复项，用于发现相似内容
        let _duplicates = self.fusion.find_duplicates(&cleaned_videos);

        // 第九步：使用 DataFusion 去重（基于相似度）+ 冲突解决策略
        let original_count = cleaned_videos.len();
        let fused_videos = DataFusion::new()
            .with_dedup_strategy(DeduplicationStrategy::Moderate)
            .with_conflict_resolution(ConflictResolution::HighestQuality)
            .fuse_videos(cleaned_videos);

        // 第十步：转换为推荐项
        let mut recommendations: Vec<RecommendationItem> = fused_videos
            .into_iter()
            .map(|video| {
                // 融合后的视频有更高的质量分数和置信度
                let score = video.quality_score * video.confidence;

                RecommendationItem {
                    title: video.title,
                    source_name: format!("智能推荐 ({}源)", video.sources.len()),
                    year: video.year.map(|y| y.to_string()).unwrap_or_default(),
                    cover: video.cover,
                    score,
                    reason: RecommendationReason::ContentDiscovery,
                }
            })
            .collect();

        // 第十一步：获取融合统计信息
        let stats = self.fusion.get_fusion_stats(original_count, recommendations.len());

        // 记录去重率（用于调试）
        if stats.deduplication_rate > 0.0 {
            eprintln!("内容发现去重率: {:.1}%，发现 {} 个重复项",
                stats.deduplication_rate, stats.duplicate_count);
        }

        // 第十二步：按分数排序
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// 构建用户偏好画像
    fn build_user_profile(&self, db: &Db) -> Result<UserProfile, String> {
        let mut favorite_categories = Vec::new();
        let mut favorite_years = Vec::new();
        let mut watched_titles = Vec::new();

        // 从观看历史提取偏好
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, year FROM play_records
                 WHERE play_time > total_time * 0.5
                 ORDER BY save_time DESC LIMIT 50"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            for row in rows.filter_map(|r| r.ok()) {
                watched_titles.push(row.0);
                if !row.1.is_empty() {
                    favorite_years.push(row.1);
                }
            }
            Ok::<_, rusqlite::Error>(())
        }).map_err(|e| e.to_string())?;

        // 从收藏提取偏好
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, year FROM favorites ORDER BY save_time DESC LIMIT 30"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            for row in rows.filter_map(|r| r.ok()) {
                watched_titles.push(row.0);
                if !row.1.is_empty() {
                    favorite_years.push(row.1);
                }
            }
            Ok::<_, rusqlite::Error>(())
        }).map_err(|e| e.to_string())?;

        // 使用 ContentAnalyzer 分析用户偏好的分类
        for title in &watched_titles {
            let category = self.analyzer.classify_video(title, None);
            favorite_categories.push(category);
        }

        Ok(UserProfile {
            watched_titles,
            favorite_categories,
            favorite_years,
        })
    }

    /// 从全局数据库中发现新内容（用户从未接触过的）
    fn discover_global_content(
        &self,
        db: &Db,
        profile: &UserProfile
    ) -> Result<Vec<(String, Option<String>, Option<String>, Option<String>, String)>, String> {
        let mut discovered = Vec::new();

        eprintln!("discover_global_content: 用户已观看 {} 个视频", profile.watched_titles.len());
        eprintln!("  已观看列表: {:?}", profile.watched_titles);

        // 策略1：从 content_pool 中获取所有未看过的新内容
        let pool_content: Vec<(String, String, String, String, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, year, cover, description, source_name
                 FROM content_pool
                 WHERE title NOT IN (SELECT title FROM play_records)
                 AND title NOT IN (SELECT title FROM favorites)
                 ORDER BY created_at DESC
                 LIMIT 100"
            )?;

            let rows: Vec<(String, String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1).unwrap_or_default(),
                        row.get(2).unwrap_or_default(),
                        row.get(3).unwrap_or_default(),
                        row.get(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        eprintln!("  SQL 查询返回 {} 条记录", pool_content.len());
        if pool_content.len() > 0 {
            eprintln!("  前3条: {:?}", pool_content.iter().take(3).map(|(t, _, _, _, s)| format!("{} ({})", t, s)).collect::<Vec<_>>());
        }

        for (title, year, cover, desc, source) in pool_content {
            if !profile.watched_titles.contains(&title) {
                discovered.push((
                    title.clone(),
                    if year.is_empty() { None } else { Some(year) },
                    if cover.is_empty() { None } else { Some(cover) },
                    if desc.is_empty() { None } else { Some(desc) },
                    source,
                ));
            } else {
                eprintln!("  跳过已观看: {}", title);
            }
        }

        eprintln!("  最终发现 {} 个新内容", discovered.len());

        Ok(discovered)
    }

    /// 从观看历史获取视频
    fn get_videos_from_history(&self, db: &Db) -> Result<Vec<VideoMetadata>, String> {
        let history: Vec<(String, String, String, String, i32)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT title, source_name, year, cover, total_episodes
                FROM play_records
                ORDER BY save_time DESC
                LIMIT 30",
            )?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        Ok(history
            .into_iter()
            .map(|(title, source, year, cover, total_episodes)| {
                let mut video = self.analyzer.analyze_video(&title, Some(&year), Some(&cover), None, &source);

                // 根据集数判断分类
                if video.category == crate::commands::content_analyzer::VideoCategory::Unknown {
                    if total_episodes > 1 {
                        video.category = crate::commands::content_analyzer::VideoCategory::TvSeries;
                    } else {
                        video.category = crate::commands::content_analyzer::VideoCategory::Movie;
                    }
                }

                video
            })
            .collect())
    }

    /// 从收藏获取视频
    fn get_videos_from_favorites(&self, db: &Db) -> Result<Vec<VideoMetadata>, String> {
        let favorites: Vec<(String, String, String, String, i32)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT title, source_name, year, cover, total_episodes
                FROM favorites
                ORDER BY save_time DESC
                LIMIT 20",
            )?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        Ok(favorites
            .into_iter()
            .map(|(title, source, year, cover, total_episodes)| {
                let mut video = self.analyzer.analyze_video(&title, Some(&year), Some(&cover), None, &source);

                // 根据集数判断分类
                if video.category == crate::commands::content_analyzer::VideoCategory::Unknown {
                    if total_episodes > 1 {
                        video.category = crate::commands::content_analyzer::VideoCategory::TvSeries;
                    } else {
                        video.category = crate::commands::content_analyzer::VideoCategory::Movie;
                    }
                }

                video
            })
            .collect())
    }

    /// 从搜索历史获取视频（用户感兴趣但可能没看过的）
    fn get_videos_from_search_history(&self, db: &Db) -> Result<Vec<VideoMetadata>, String> {
        let searches: Vec<String> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT keyword
                FROM search_history
                ORDER BY save_time DESC
                LIMIT 50",
            )?;

            let rows = stmt
                .query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        Ok(searches
            .into_iter()
            .map(|keyword| {
                self.analyzer.analyze_video(&keyword, None, None, None, "搜索历史")
            })
            .collect())
    }

    /// 获取未完成的视频（可能感兴趣但没看完）
    fn get_incomplete_videos(&self, db: &Db) -> Result<Vec<VideoMetadata>, String> {
        let incomplete: Vec<(String, String, String, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT title, source_name, year, cover
                FROM play_records
                WHERE play_time > 0 AND play_time < total_time * 0.3
                ORDER BY save_time DESC
                LIMIT 20",
            )?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        Ok(incomplete
            .into_iter()
            .map(|(title, source, year, cover)| {
                self.analyzer.analyze_video(&title, Some(&year), Some(&cover), None, &source)
            })
            .collect())
    }
    /// 从内容池中发现新内容
    fn discover_from_content_pool(
        &self,
        db: &Db,
        profile: &UserProfile
    ) -> Result<Vec<(String, Option<String>, Option<String>, Option<String>, String)>, String> {
        let mut discovered = Vec::new();

        // 获取用户已观看的标题，用于排除
        let watched_titles: Vec<String> = profile.watched_titles.clone();

        // 策略1：推荐所有未看过的内容（不限制分类和评分，因为数据可能不完整）
        let all_content: Vec<(String, String, String, String, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT title, year, cover, description, source_name
                 FROM content_pool
                 WHERE title NOT IN (SELECT title FROM play_records)
                 AND title NOT IN (SELECT title FROM favorites)
                 ORDER BY created_at DESC
                 LIMIT 50"
            )?;

            let rows: Vec<(String, String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1).unwrap_or_default(),
                        row.get(2).unwrap_or_default(),
                        row.get(3).unwrap_or_default(),
                        row.get(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        for (title, year, cover, desc, source) in all_content {
            if !watched_titles.contains(&title) {
                discovered.push((
                    title,
                    if year.is_empty() { None } else { Some(year) },
                    if cover.is_empty() { None } else { Some(cover) },
                    if desc.is_empty() { None } else { Some(desc) },
                    source,
                ));
            }
        }

        Ok(discovered)
    }

    /// 从图片缓存中发现新内容（基于访问热度）
    fn discover_from_image_cache(
        &self,
        db: &Db,
        profile: &UserProfile
    ) -> Result<Vec<(String, Option<String>, Option<String>, Option<String>, String)>, String> {
        let mut discovered = Vec::new();

        // 获取用户已观看的标题
        let watched_titles: Vec<String> = profile.watched_titles.clone();

        // 推荐所有有标题的缓存内容（不限制访问次数和评分）
        let cached_content: Vec<(String, String, String, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT title, year, url, source_name
                 FROM image_cache
                 WHERE title IS NOT NULL
                 AND title != ''
                 AND title NOT IN (SELECT title FROM play_records)
                 AND title NOT IN (SELECT title FROM favorites)
                 ORDER BY last_accessed DESC
                 LIMIT 50"
            )?;

            let rows: Vec<(String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1).unwrap_or_default(),
                        row.get(2)?,
                        row.get(3).unwrap_or_default(),
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(rows)
        })?;

        for (title, year, cover_url, source) in cached_content {
            if !watched_titles.contains(&title) {
                discovered.push((
                    title,
                    if year.is_empty() { None } else { Some(year) },
                    Some(cover_url),
                    None,
                    if source.is_empty() { "图片缓存".to_string() } else { source },
                ));
            }
        }

        Ok(discovered)
    }

    /// 混合推荐策略
    pub async fn get_recommendations(&self, db: &Db) -> Result<Vec<RecommendationItem>, String> {
        // 检查缓存
        let cache_key = "main_recommendations".to_string();
        {
            let cache = self.recommendation_cache.lock().unwrap();
            if let Some((items, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed() < self.cache_ttl {
                    eprintln!("使用缓存的推荐结果 ({} 个)", items.len());
                    return Ok(items.clone());
                }
            }
        }

        eprintln!("生成新的推荐结果...");

        // 获取用户统计
        let user_stats = self.get_user_stats(db)?;
        eprintln!("用户统计: {} 条播放记录", user_stats.total_records);
        let mut recommendations = Vec::new();

        // 根据用户历史数量决定推荐策略
        if user_stats.total_records < 5 {
            // 新用户：100% 冷启动推荐
            recommendations = self.cold_start_recommendations(db)?;
        } else if user_stats.total_records < 20 {
            // 少量历史：30% 个性化 + 30% 智能推荐 + 20% 相似内容 + 20% 热门
            let personalized = self.personalized_recommendations(db)?;
            let intelligent = self.intelligent_recommendations(db).await?;
            let category_based = self.category_based_recommendations(db)?;
            let cold_start = self.cold_start_recommendations(db)?;

            let thirty_percent_p = (personalized.len() as f64 * 0.3) as usize;
            let thirty_percent_i = (intelligent.len() as f64 * 0.3) as usize;
            let twenty_percent_c = (category_based.len() as f64 * 0.2) as usize;
            let twenty_percent_h = (cold_start.len() as f64 * 0.2) as usize;

            recommendations.append(&mut personalized.into_iter().take(thirty_percent_p).collect());
            recommendations.append(&mut intelligent.into_iter().take(thirty_percent_i).collect());
            recommendations.append(&mut category_based.into_iter().take(twenty_percent_c).collect());
            recommendations.append(&mut cold_start.into_iter().take(twenty_percent_h).collect());
        } else {
            // 丰富历史：40% 智能推荐 + 30% 个性化 + 20% 分类推荐 + 10% 热门
            let intelligent = self.intelligent_recommendations(db).await?;
            let personalized = self.personalized_recommendations(db)?;
            let category_based = self.category_based_recommendations(db)?;
            let cold_start = self.cold_start_recommendations(db)?;

            let forty_percent = (intelligent.len() as f64 * 0.4) as usize;
            let thirty_percent = (personalized.len() as f64 * 0.3) as usize;
            let twenty_percent = (category_based.len() as f64 * 0.2) as usize;
            let ten_percent = (cold_start.len() as f64 * 0.1) as usize;

            recommendations.append(&mut intelligent.into_iter().take(forty_percent).collect());
            recommendations.append(&mut personalized.into_iter().take(thirty_percent).collect());
            recommendations.append(&mut category_based.into_iter().take(twenty_percent).collect());
            recommendations.append(&mut cold_start.into_iter().take(ten_percent).collect());
        }

        // 去重
        recommendations = self.deduplicate_recommendations(recommendations);
        eprintln!("去重后: {} 个推荐", recommendations.len());

        // 按分数排序
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // 限制返回数量为6个
        recommendations.truncate(6);

        eprintln!("最终推荐: {} 个", recommendations.len());
        for (i, item) in recommendations.iter().enumerate() {
            eprintln!("  {}. {} (来源: {}, 分数: {:.2})", i+1, item.title, item.source_name, item.score);
        }

        // 更新缓存
        {
            let mut cache = self.recommendation_cache.lock().unwrap();
            cache.insert(cache_key, (recommendations.clone(), Instant::now()));
        }

        Ok(recommendations)
    }

    /// 去重推荐结果
    fn deduplicate_recommendations(
        &self,
        recommendations: Vec<RecommendationItem>,
    ) -> Vec<RecommendationItem> {
        let mut seen = std::collections::HashSet::new();
        recommendations
            .into_iter()
            .filter(|item| seen.insert(item.title.clone()))
            .collect()
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        let mut user_cache = self.user_stats_cache.lock().unwrap();
        *user_cache = None;

        let mut global_cache = self.global_stats_cache.lock().unwrap();
        *global_cache = None;

        let mut rec_cache = self.recommendation_cache.lock().unwrap();
        rec_cache.clear();
    }
}

// ========== Tauri 命令 ==========

/// 获取推荐内容
#[tauri::command]
pub async fn get_recommendations(
    db: State<'_, Db>,
    engine: State<'_, RecommendationEngine>,
) -> Result<Vec<RecommendationItem>, String> {
    let result = engine.get_recommendations(&db).await?;

    // 输出推荐结果到控制台
    eprintln!("\n========== 推荐结果 ==========");
    eprintln!("返回 {} 个推荐", result.len());
    for (i, item) in result.iter().take(10).enumerate() {
        eprintln!("  {}. {} (来源: {}, 年份: {}, 分数: {:.2})",
            i+1, item.title, item.source_name, item.year, item.score);
    }
    eprintln!("==============================\n");

    Ok(result)
}

/// 清除推荐缓存
#[tauri::command]
pub fn clear_recommendation_cache(engine: State<'_, RecommendationEngine>) -> Result<(), String> {
    engine.clear_cache();
    Ok(())
}

/// 添加内容到内容池
#[tauri::command]
pub fn add_to_content_pool(
    db: State<'_, Db>,
    title: String,
    source_name: String,
    year: Option<String>,
    cover: Option<String>,
    category: Option<String>,
    rating: Option<f64>,
    description: Option<String>,
    tags: Option<String>,
) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO content_pool
             (title, source_name, year, cover, category, rating, description, tags, popularity_score, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                title,
                source_name,
                year.unwrap_or_default(),
                cover.unwrap_or_default(),
                category.unwrap_or_default(),
                rating.unwrap_or(0.0),
                description.unwrap_or_default(),
                tags.unwrap_or_default(),
                rating.unwrap_or(0.0) * 10.0, // 简单的热度计算
                now,
                now,
            ],
        )?;
        Ok(())
    })
}

/// 批量添加内容到内容池
#[tauri::command]
pub fn batch_add_to_content_pool(
    db: State<'_, Db>,
    items: Vec<serde_json::Value>,
) -> Result<usize, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut count = 0;

    for item in items {
        let title = item["title"].as_str().unwrap_or("").to_string();
        let source_name = item["source_name"].as_str().unwrap_or("").to_string();

        if title.is_empty() || source_name.is_empty() {
            continue;
        }

        let year = item["year"].as_str().map(|s| s.to_string());
        let cover = item["cover"].as_str().map(|s| s.to_string());
        let category = item["category"].as_str().map(|s| s.to_string());
        let rating = item["rating"].as_f64();
        let description = item["description"].as_str().map(|s| s.to_string());
        let tags = item["tags"].as_str().map(|s| s.to_string());

        let result = db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO content_pool
                 (title, source_name, year, cover, category, rating, description, tags, popularity_score, created_at, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    title,
                    source_name,
                    year.unwrap_or_default(),
                    cover.unwrap_or_default(),
                    category.unwrap_or_default(),
                    rating.unwrap_or(0.0),
                    description.unwrap_or_default(),
                    tags.unwrap_or_default(),
                    rating.unwrap_or(0.0) * 10.0,
                    now,
                    now,
                ],
            )?;
            Ok(())
        });

        if result.is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

/// 更新图片缓存的元数据（用于推荐）
#[tauri::command]
pub fn update_image_cache_metadata(
    db: State<'_, Db>,
    url: String,
    title: Option<String>,
    source_name: Option<String>,
    year: Option<String>,
    category: Option<String>,
    rating: Option<f64>,
) -> Result<(), String> {
    // 智能推断 category（如果没有提供）
    let inferred_category = if category.is_none() || category.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
        if let Some(ref t) = title {
            let src = source_name.as_deref().unwrap_or("");
            infer_category(t, src)
        } else {
            None
        }
    } else {
        category
    };

    db.with_conn(|conn| {
        conn.execute(
            "UPDATE image_cache
             SET title = COALESCE(?2, title),
                 source_name = COALESCE(?3, source_name),
                 year = COALESCE(?4, year),
                 category = COALESCE(?5, category),
                 rating = COALESCE(?6, rating)
             WHERE url = ?1",
            rusqlite::params![
                url,
                title,
                source_name,
                year,
                inferred_category,
                rating,
            ],
        )?;
        Ok(())
    })
}

/// 智能推断分类（公开方法，供 douban_client 等模块调用）
pub fn infer_category(_title: &str, source_name: &str) -> Option<String> {
    // Bangumi 来源 → Anime
    if source_name.contains("Bangumi") {
        return Some("Anime".to_string());
    }

    // 豆瓣来源默认 → Movie
    if source_name.contains("豆瓣") {
        return Some("Movie".to_string());
    }

    None
}

/// 修复 image_cache 和 content_pool 的空数据
///
/// 1. 从 content_pool 回填 image_cache 的 title 和 category
/// 2. 智能推断 content_pool 的空 category
#[tauri::command]
pub fn fix_empty_metadata(db: State<'_, Db>) -> Result<String, String> {
    fix_empty_metadata_direct(&db)
}

/// 直接调用版本（启动时使用，不需要 State 包装）
pub fn fix_empty_metadata_direct(db: &Db) -> Result<String, String> {
    let mut stats = String::new();

    // 1. 从 content_pool 回填 image_cache 的元数据
    let updated_images = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT ic.url, cp.title, cp.source_name, cp.year, cp.category, cp.rating
             FROM image_cache ic
             LEFT JOIN content_pool cp ON ic.url = cp.cover
             WHERE (ic.title IS NULL OR ic.title = '')
               AND cp.title IS NOT NULL",
        )?;

        let rows: Vec<(String, String, String, String, String, f64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut count = 0;
        for (url, title, source_name, year, category, rating) in rows {
            conn.execute(
                "UPDATE image_cache
                 SET title = ?, source_name = ?, year = ?, category = ?, rating = ?
                 WHERE url = ?",
                (&title, &source_name, &year, &category, rating, &url),
            )?;
            count += 1;
        }

        Ok::<usize, rusqlite::Error>(count)
    })?;

    stats.push_str(&format!("回填 image_cache: {} 条记录\n", updated_images));

    // 2. 智能推断 content_pool 的空 category
    let updated_categories = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT title, source_name FROM content_pool
             WHERE category IS NULL OR category = ''",
        )?;

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut count = 0;
        for (title, source_name) in rows {
            if let Some(category) = infer_category(&title, &source_name) {
                conn.execute(
                    "UPDATE content_pool SET category = ? WHERE title = ? AND source_name = ?",
                    (&category, &title, &source_name),
                )?;
                count += 1;
            }
        }

        Ok::<usize, rusqlite::Error>(count)
    })?;

    stats.push_str(&format!(
        "推断 content_pool category: {} 条记录",
        updated_categories
    ));

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_recommendations() {
        let engine = RecommendationEngine::new();
        let recommendations = engine.get_default_recommendations();

        assert!(!recommendations.is_empty());
        assert!(recommendations.len() >= 5);
        assert!(recommendations.iter().all(|r| r.score > 0.0));
    }

    #[test]
    fn test_deduplicate_recommendations() {
        let engine = RecommendationEngine::new();
        let recommendations = vec![
            RecommendationItem {
                title: "Test1".to_string(),
                source_name: "Source1".to_string(),
                year: "2024".to_string(),
                cover: "".to_string(),
                score: 8.0,
                reason: RecommendationReason::Popular,
            },
            RecommendationItem {
                title: "Test1".to_string(), // 重复
                source_name: "Source2".to_string(),
                year: "2024".to_string(),
                cover: "".to_string(),
                score: 7.0,
                reason: RecommendationReason::Popular,
            },
            RecommendationItem {
                title: "Test2".to_string(),
                source_name: "Source1".to_string(),
                year: "2024".to_string(),
                cover: "".to_string(),
                score: 9.0,
                reason: RecommendationReason::Popular,
            },
        ];

        let deduplicated = engine.deduplicate_recommendations(recommendations);
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(deduplicated[0].title, "Test1");
        assert_eq!(deduplicated[1].title, "Test2");
    }

    #[test]
    fn test_cache_ttl() {
        let engine = RecommendationEngine::new();
        assert_eq!(engine.cache_ttl, Duration::from_secs(300));
    }
}
