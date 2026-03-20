/// 内容分析器
///
/// 功能：
/// 1. 视频元数据标准化和提取
/// 2. 智能标签分类和提取
/// 3. 内容质量评分系统
/// 4. 视频相似度计算
/// 5. 年份、类型、演员等信息解析
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 视频元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct VideoMetadata {
    pub title: String,
    pub normalized_title: String, // 标准化后的标题
    pub year: Option<u16>,
    pub category: VideoCategory,
    pub tags: Vec<String>,
    pub quality_score: f64,
    pub actors: Vec<String>,
    pub director: Option<String>,
    pub description: Option<String>,
}

/// 视频分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum VideoCategory {
    Movie,       // 电影
    TvSeries,    // 电视剧
    Anime,       // 动漫
    Variety,     // 综艺
    Documentary, // 纪录片
    Unknown,     // 未知
}

/// 内容质量评分因素
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QualityFactors {
    pub has_year: bool,
    pub has_cover: bool,
    pub has_description: bool,
    pub title_length: usize,
    pub source_reliability: f64,
    pub metadata_completeness: f64,
}

/// 内容分析器
#[allow(dead_code)]
pub struct ContentAnalyzer {
    // 分类关键词映射
    category_keywords: HashMap<VideoCategory, Vec<String>>,
    // 质量评分权重
    quality_weights: QualityWeights,
    // 停用词（用于标题标准化）
    stop_words: HashSet<String>,
}

/// 质量评分权重
#[derive(Debug, Clone)]
struct QualityWeights {
    year: f64,
    cover: f64,
    description: f64,
    title_length: f64,
    source_reliability: f64,
    metadata_completeness: f64,
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            year: 0.15,
            cover: 0.20,
            description: 0.15,
            title_length: 0.10,
            source_reliability: 0.25,
            metadata_completeness: 0.15,
        }
    }
}

impl ContentAnalyzer {
    pub fn new() -> Self {
        let mut category_keywords = HashMap::new();

        // 电影关键词
        category_keywords.insert(
            VideoCategory::Movie,
            vec![
                "电影".to_string(),
                "影片".to_string(),
                "院线".to_string(),
                "4K".to_string(),
                "蓝光".to_string(),
                "HD".to_string(),
            ],
        );

        // 电视剧关键词
        category_keywords.insert(
            VideoCategory::TvSeries,
            vec![
                "电视剧".to_string(),
                "剧集".to_string(),
                "连续剧".to_string(),
                "美剧".to_string(),
                "韩剧".to_string(),
                "日剧".to_string(),
                "港剧".to_string(),
                "国产剧".to_string(),
            ],
        );

        // 动漫关键词
        category_keywords.insert(
            VideoCategory::Anime,
            vec![
                "动漫".to_string(),
                "动画".to_string(),
                "番剧".to_string(),
                "漫画".to_string(),
                "新番".to_string(),
            ],
        );

        // 综艺关键词
        category_keywords.insert(
            VideoCategory::Variety,
            vec![
                "综艺".to_string(),
                "真人秀".to_string(),
                "脱口秀".to_string(),
                "晚会".to_string(),
                "访谈".to_string(),
            ],
        );

        // 纪录片关键词
        category_keywords.insert(
            VideoCategory::Documentary,
            vec![
                "纪录片".to_string(),
                "纪实".to_string(),
                "探索".to_string(),
                "自然".to_string(),
            ],
        );

        // 停用词
        let stop_words: HashSet<String> = vec![
            "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上",
            "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            category_keywords,
            quality_weights: QualityWeights::default(),
            stop_words,
        }
    }

    /// 标准化标题
    pub fn normalize_title(&self, title: &str) -> String {
        let mut normalized = title.to_string();

        // 移除特殊字符
        normalized = normalized
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '·')
            .collect();

        // 移除多余空格
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        // 转小写（用于比较）
        normalized = normalized.to_lowercase();

        normalized
    }

    /// 从标题中提取年份
    pub fn extract_year(&self, title: &str) -> Option<u16> {
        // 匹配 4 位数字年份（1900-2099）
        let year_regex = regex::Regex::new(r"(19\d{2}|20\d{2})").ok()?;

        if let Some(captures) = year_regex.captures(title) {
            if let Some(year_str) = captures.get(1) {
                return year_str.as_str().parse::<u16>().ok();
            }
        }

        None
    }

    /// 分类视频
    pub fn classify_video(&self, title: &str, description: Option<&str>) -> VideoCategory {
        let text = format!("{} {}", title, description.unwrap_or("")).to_lowercase();

        let mut scores: HashMap<VideoCategory, usize> = HashMap::new();

        // 统计每个分类的关键词出现次数
        for (category, keywords) in &self.category_keywords {
            let mut score = 0;
            for keyword in keywords {
                if text.contains(&keyword.to_lowercase()) {
                    score += 1;
                }
            }
            scores.insert(category.clone(), score);
        }

        // 返回得分最高的分类
        scores
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(category, score)| {
                if score > 0 {
                    category
                } else {
                    VideoCategory::Unknown
                }
            })
            .unwrap_or(VideoCategory::Unknown)
    }

    /// 提取标签
    pub fn extract_tags(&self, title: &str, description: Option<&str>) -> Vec<String> {
        let mut tags = Vec::new();
        let text = format!("{} {}", title, description.unwrap_or(""));

        // 提取年份标签
        if let Some(year) = self.extract_year(&text) {
            tags.push(format!("{}年", year));
        }

        // 提取分类标签
        let category = self.classify_video(title, description);
        match category {
            VideoCategory::Movie => tags.push("电影".to_string()),
            VideoCategory::TvSeries => tags.push("电视剧".to_string()),
            VideoCategory::Anime => tags.push("动漫".to_string()),
            VideoCategory::Variety => tags.push("综艺".to_string()),
            VideoCategory::Documentary => tags.push("纪录片".to_string()),
            VideoCategory::Unknown => {}
        }

        // 提取质量标签
        if text.contains("4K") || text.contains("4k") {
            tags.push("4K".to_string());
        }
        if text.contains("蓝光") || text.contains("BluRay") {
            tags.push("蓝光".to_string());
        }
        if text.contains("HD") || text.contains("高清") {
            tags.push("高清".to_string());
        }

        // 提取地区标签
        if text.contains("美剧") || text.contains("美国") {
            tags.push("美国".to_string());
        }
        if text.contains("韩剧") || text.contains("韩国") {
            tags.push("韩国".to_string());
        }
        if text.contains("日剧") || text.contains("日本") {
            tags.push("日本".to_string());
        }
        if text.contains("港剧") || text.contains("香港") {
            tags.push("香港".to_string());
        }
        if text.contains("国产") || text.contains("大陆") {
            tags.push("国产".to_string());
        }

        tags
    }

    /// 计算内容质量评分
    pub fn calculate_quality_score(&self, factors: &QualityFactors) -> f64 {
        let mut score = 0.0;

        // 年份得分
        if factors.has_year {
            score += self.quality_weights.year;
        }

        // 封面得分
        if factors.has_cover {
            score += self.quality_weights.cover;
        }

        // 描述得分
        if factors.has_description {
            score += self.quality_weights.description;
        }

        // 标题长度得分（5-50 字符为最佳）
        let title_score = if factors.title_length >= 5 && factors.title_length <= 50 {
            1.0
        } else if factors.title_length < 5 {
            factors.title_length as f64 / 5.0
        } else {
            50.0 / factors.title_length as f64
        };
        score += title_score * self.quality_weights.title_length;

        // 源可靠性得分
        score += factors.source_reliability * self.quality_weights.source_reliability;

        // 元数据完整性得分
        score += factors.metadata_completeness * self.quality_weights.metadata_completeness;

        // 归一化到 0-10 分
        (score * 10.0).min(10.0).max(0.0)
    }

    /// 计算两个标题的相似度（Jaccard 相似度）
    pub fn calculate_similarity(&self, title1: &str, title2: &str) -> f64 {
        let normalized1 = self.normalize_title(title1);
        let normalized2 = self.normalize_title(title2);

        // 分词（简单按空格分割）
        let words1: HashSet<String> = normalized1
            .split_whitespace()
            .filter(|w| !self.stop_words.contains(*w))
            .map(|s| s.to_string())
            .collect();

        let words2: HashSet<String> = normalized2
            .split_whitespace()
            .filter(|w| !self.stop_words.contains(*w))
            .map(|s| s.to_string())
            .collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        // Jaccard 相似度 = 交集 / 并集
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        intersection as f64 / union as f64
    }

    /// 分析视频元数据
    pub fn analyze_video(
        &self,
        title: &str,
        year: Option<&str>,
        cover: Option<&str>,
        description: Option<&str>,
        source_name: &str,
    ) -> VideoMetadata {
        let normalized_title = self.normalize_title(title);

        // 提取年份
        let year_value = year
            .and_then(|y| y.parse::<u16>().ok())
            .or_else(|| self.extract_year(title));

        // 分类
        let category = self.classify_video(title, description);

        // 提取标签
        let tags = self.extract_tags(title, description);

        // 计算质量评分
        let quality_factors = QualityFactors {
            has_year: year_value.is_some(),
            has_cover: cover.is_some() && !cover.unwrap().is_empty(),
            has_description: description.is_some() && !description.unwrap().is_empty(),
            title_length: title.len(),
            source_reliability: self.get_source_reliability(source_name),
            metadata_completeness: self.calculate_metadata_completeness(
                year_value.is_some(),
                cover.is_some(),
                description.is_some(),
            ),
        };

        let quality_score = self.calculate_quality_score(&quality_factors);

        VideoMetadata {
            title: title.to_string(),
            normalized_title,
            year: year_value,
            category,
            tags,
            quality_score,
            actors: Vec::new(), // 需要更复杂的解析
            director: None,     // 需要更复杂的解析
            description: description.map(|s| s.to_string()),
        }
    }

    /// 获取源的可靠性评分
    fn get_source_reliability(&self, source_name: &str) -> f64 {
        // 简单的启发式规则，可以后续基于用户反馈优化
        let source_lower = source_name.to_lowercase();

        if source_lower.contains("官方") || source_lower.contains("正版") {
            1.0
        } else if source_lower.contains("高清") || source_lower.contains("hd") {
            0.8
        } else if source_lower.contains("vip") {
            0.7
        } else {
            0.5
        }
    }

    /// 计算元数据完整性
    fn calculate_metadata_completeness(
        &self,
        has_year: bool,
        has_cover: bool,
        has_description: bool,
    ) -> f64 {
        let mut completeness = 0.0;
        let total_fields = 3.0;

        if has_year {
            completeness += 1.0;
        }
        if has_cover {
            completeness += 1.0;
        }
        if has_description {
            completeness += 1.0;
        }

        completeness / total_fields
    }

    /// 批量分析视频
    pub fn batch_analyze(
        &self,
        videos: Vec<(
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
        )>,
    ) -> Vec<VideoMetadata> {
        videos
            .into_iter()
            .map(|(title, year, cover, description, source_name)| {
                self.analyze_video(
                    &title,
                    year.as_deref(),
                    cover.as_deref(),
                    description.as_deref(),
                    &source_name,
                )
            })
            .collect()
    }

    /// 查找相似视频
    pub fn find_similar_videos(
        &self,
        target_title: &str,
        candidates: &[String],
        threshold: f64,
    ) -> Vec<(String, f64)> {
        candidates
            .iter()
            .map(|candidate| {
                let similarity = self.calculate_similarity(target_title, candidate);
                (candidate.clone(), similarity)
            })
            .filter(|(_, similarity)| *similarity >= threshold)
            .collect()
    }
}

impl Default for ContentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        let analyzer = ContentAnalyzer::new();

        let title = "肖申克的救赎 (1994)";
        let normalized = analyzer.normalize_title(title);
        assert_eq!(normalized, "肖申克的救赎 1994");
    }

    #[test]
    fn test_extract_year() {
        let analyzer = ContentAnalyzer::new();

        assert_eq!(analyzer.extract_year("肖申克的救赎 (1994)"), Some(1994));
        assert_eq!(analyzer.extract_year("2024年新片"), Some(2024));
        assert_eq!(analyzer.extract_year("没有年份的标题"), None);
    }

    #[test]
    fn test_classify_video() {
        let analyzer = ContentAnalyzer::new();

        assert_eq!(
            analyzer.classify_video("复仇者联盟 电影", None),
            VideoCategory::Movie
        );
        assert_eq!(
            analyzer.classify_video("权力的游戏 美剧", None),
            VideoCategory::TvSeries
        );
        assert_eq!(
            analyzer.classify_video("进击的巨人 动漫", None),
            VideoCategory::Anime
        );
    }

    #[test]
    fn test_extract_tags() {
        let analyzer = ContentAnalyzer::new();

        let tags = analyzer.extract_tags("复仇者联盟4 (2019) 4K蓝光", None);
        assert!(tags.contains(&"2019年".to_string()));
        assert!(tags.contains(&"电影".to_string()));
        assert!(tags.contains(&"4K".to_string()));
        assert!(tags.contains(&"蓝光".to_string()));
    }

    #[test]
    fn test_calculate_quality_score() {
        let analyzer = ContentAnalyzer::new();

        let factors = QualityFactors {
            has_year: true,
            has_cover: true,
            has_description: true,
            title_length: 20,
            source_reliability: 0.8,
            metadata_completeness: 1.0,
        };

        let score = analyzer.calculate_quality_score(&factors);
        assert!(score > 7.0);
        assert!(score <= 10.0);
    }

    #[test]
    fn test_calculate_similarity() {
        let analyzer = ContentAnalyzer::new();

        let similarity = analyzer.calculate_similarity("肖申克的救赎", "肖申克的救赎 1994");
        assert!(similarity > 0.3); // 降低阈值，因为添加了年份

        let similarity = analyzer.calculate_similarity("肖申克的救赎", "阿甘正传");
        assert!(similarity < 0.5);

        // 完全相同的标题
        let similarity = analyzer.calculate_similarity("肖申克的救赎", "肖申克的救赎");
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_analyze_video() {
        let analyzer = ContentAnalyzer::new();

        let metadata = analyzer.analyze_video(
            "肖申克的救赎 (1994)",
            Some("1994"),
            Some("https://example.com/cover.jpg"),
            Some("经典电影"),
            "高清影院",
        );

        assert_eq!(metadata.title, "肖申克的救赎 (1994)");
        assert_eq!(metadata.year, Some(1994));
        assert_eq!(metadata.category, VideoCategory::Movie);
        assert!(metadata.quality_score > 5.0);
        assert!(!metadata.tags.is_empty());
    }

    #[test]
    fn test_find_similar_videos() {
        let analyzer = ContentAnalyzer::new();

        let candidates = vec![
            "肖申克的救赎 1994".to_string(),
            "肖申克的救赎 高清版".to_string(),
            "阿甘正传".to_string(),
        ];

        let similar = analyzer.find_similar_videos("肖申克的救赎", &candidates, 0.5);

        assert_eq!(similar.len(), 2);
    }

    #[test]
    fn test_batch_analyze() {
        let analyzer = ContentAnalyzer::new();

        let videos = vec![
            (
                "肖申克的救赎".to_string(),
                Some("1994".to_string()),
                Some("cover1.jpg".to_string()),
                Some("经典电影".to_string()),
                "源1".to_string(),
            ),
            (
                "阿甘正传".to_string(),
                Some("1994".to_string()),
                Some("cover2.jpg".to_string()),
                Some("励志电影".to_string()),
                "源2".to_string(),
            ),
        ];

        let results = analyzer.batch_analyze(videos);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|m| m.quality_score > 0.0));
    }

    #[test]
    fn test_source_reliability() {
        let analyzer = ContentAnalyzer::new();

        assert_eq!(analyzer.get_source_reliability("官方源"), 1.0);
        assert_eq!(analyzer.get_source_reliability("高清影院"), 0.8);
        assert_eq!(analyzer.get_source_reliability("VIP影院"), 0.7);
        assert_eq!(analyzer.get_source_reliability("普通源"), 0.5);
    }

    #[test]
    fn test_metadata_completeness() {
        let analyzer = ContentAnalyzer::new();

        assert_eq!(
            analyzer.calculate_metadata_completeness(true, true, true),
            1.0
        );
        assert_eq!(
            analyzer.calculate_metadata_completeness(true, true, false),
            2.0 / 3.0
        );
        assert_eq!(
            analyzer.calculate_metadata_completeness(false, false, false),
            0.0
        );
    }
}
