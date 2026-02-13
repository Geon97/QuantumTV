// 是否为成人源
const ADULT_KEYWORDS: [&str; 6] = ["adult", "18+", "nsfw", "成人", "情色", "🔞"];

pub fn is_adult_source(source: &str) -> bool {
    let source = source.to_lowercase();
    ADULT_KEYWORDS
        .iter()
        .any(|&keyword| source.contains(keyword))
}
