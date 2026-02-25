pub mod adult;
pub mod admin_config;
pub mod playback;
pub mod search_aggregation;
pub mod source_selection;
pub mod types;

pub use adult::{filter_adult_sources, is_adult_source};
pub use admin_config::parse_admin_config;
pub use admin_config::normalize_source_config;
pub use playback::{filter_ads_from_m3_u8, SkipAction, SkipDetection};
pub use search_aggregation::{
    aggregate_search_results, apply_filter, compute_group_stats, sort_by_year, AggregatedGroup,
    SearchFilter, YearOrder,
};
pub use source_selection::{
    calculate_source_score, prefer_best_source, test_video_source, SourceTestResult,
};
pub use types::SearchResult;
