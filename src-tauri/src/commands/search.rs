use serde::{Deserialize, Serialize};
use tauri::State;

use crate::storage::StorageManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSuggestion {
    text: String,
}

/// Get search suggestions based on query from search history
#[tauri::command]
pub async fn get_search_suggestions(
    query: String,
    state: State<'_, StorageManager>,
) -> Result<Vec<SearchSuggestion>, String> {
    let data = state.get_data()?;

    // Get search history from storage
    let search_history = data
        .search_history
        .as_array()
        .ok_or("Search history is not an array")?;

    let query_lower = query.to_lowercase();

    // Filter and collect matching suggestions
    let mut suggestions: Vec<(String, usize)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for item in search_history.iter().rev() {
        if let Some(text) = item.as_str() {
            let text_lower = text.to_lowercase();

            // Check if the text contains the query
            if text_lower.contains(&query_lower) && !seen.contains(text) {
                seen.insert(text.to_string());

                // Calculate relevance score (lower is better)
                let score = if text_lower.starts_with(&query_lower) {
                    0 // Exact prefix match
                } else if text_lower.contains(&format!(" {}", query_lower)) {
                    1 // Word boundary match
                } else {
                    2 // Contains match
                };

                suggestions.push((text.to_string(), score));

                // Limit to 20 candidates before sorting
                if suggestions.len() >= 20 {
                    break;
                }
            }
        }
    }

    // Sort by relevance score, then alphabetically
    suggestions.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    // Take top 10 and convert to SearchSuggestion
    let result: Vec<SearchSuggestion> = suggestions
        .into_iter()
        .take(10)
        .map(|(text, _)| SearchSuggestion { text })
        .collect();

    Ok(result)
}
