use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchResult {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub poster: String,
    #[serde(default)]
    pub episodes: Vec<String>,
    #[serde(default)]
    pub episodes_titles: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub source_name: String,
    pub class: Option<String>,
    pub year: Option<String>,
    pub desc: Option<String>,
    pub type_name: Option<String>,
    pub douban_id: Option<i32>,
}
