pub mod adult;
pub mod playback;
pub mod source_selection;
pub mod types;

pub use adult::{filter_adult_sources, is_adult_source};
pub use playback::filter_ads_from_m3_u8;
pub use source_selection::{
    calculate_source_score, prefer_best_source, test_video_source, SourceTestResult,
};
pub use types::SearchResult;
