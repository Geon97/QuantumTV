use crate::adult::is_adult_source;
use serde_json::{json, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_CONFIG_SIZE: usize = 2 * 1024 * 1024; // 2MB hard limit
const DEFAULT_SOURCE_NAME: &str = "未命名源";

pub fn parse_admin_config(raw_json: &str) -> Result<Value, String> {
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        return Err("配置内容不能为空".to_string());
    }
    if trimmed.len() > MAX_CONFIG_SIZE {
        return Err("配置内容过大".to_string());
    }

    let value: Value = serde_json::from_str(trimmed).map_err(|_| "配置格式错误".to_string())?;
    normalize_admin_config_value(value)
}

pub fn normalize_source_config(source: &Value, default_from: &str) -> Result<Value, String> {
    let now_ms = current_time_ms();
    normalize_source_config_item(source, default_from, now_ms, 0)
        .ok_or_else(|| "视频源格式错误".to_string())
}

fn normalize_admin_config_value(value: Value) -> Result<Value, String> {
    match value {
        Value::Array(items) => {
            if !is_source_array(&items) {
                return Err("配置格式错误".to_string());
            }
            let sources = normalize_source_config_array(&items, "custom");
            Ok(build_config_with_sources(sources))
        }
        Value::Object(map) => {
            if is_admin_config(&map) {
                normalize_admin_config_object(&map)
            } else if let Some(sites) = map.get("sites").and_then(|v| v.as_array()) {
                let sources = normalize_source_config_array(sites, "config");
                Ok(build_config_with_sources(sources))
            } else if let Some(api_site) = map.get("api_site").and_then(|v| v.as_object()) {
                let sources = normalize_api_site_object(api_site);
                Ok(build_config_with_sources(sources))
            } else {
                Err("配置格式错误".to_string())
            }
        }
        _ => Err("配置格式错误".to_string()),
    }
}

fn normalize_admin_config_object(map: &Map<String, Value>) -> Result<Value, String> {
    let mut config = default_admin_config_value();
    let default_config = config.clone();

    if let Some(config_file) = get_value(map, &["ConfigFile", "config_file"]).and_then(|v| v.as_str()) {
        set_string(&mut config, "ConfigFile", config_file);
    }

    if let Some(config_sub) = get_value(map, &["ConfigSubscribtion", "config_subscribtion"]) {
        let default_sub = default_config
            .get("ConfigSubscribtion")
            .unwrap_or(&Value::Null);
        let merged = merge_object(default_sub, config_sub);
        set_value(&mut config, "ConfigSubscribtion", merged);
    }

    if let Some(user_prefs) = get_value(map, &["UserPreferences", "user_preferences"]) {
        let default_prefs = default_config
            .get("UserPreferences")
            .unwrap_or(&Value::Null);
        let merged = merge_object(default_prefs, user_prefs);
        set_value(&mut config, "UserPreferences", merged);
    }

    if let Some(user_config) = get_value(map, &["UserConfig", "user_config"]) {
        if user_config.is_object() {
            set_value(&mut config, "UserConfig", user_config.clone());
        }
    }

    if let Some(source_config) = get_value(map, &["SourceConfig", "source_config"]) {
        if let Some(arr) = source_config.as_array() {
            let sources = normalize_source_config_array(arr, "custom");
            set_value(&mut config, "SourceConfig", Value::Array(sources));
        }
    }

    if let Some(categories) = get_value(map, &["CustomCategories", "custom_categories"]) {
        if let Some(arr) = categories.as_array() {
            let normalized = normalize_custom_categories(arr, "custom");
            set_value(&mut config, "CustomCategories", Value::Array(normalized));
        }
    }

    let needs_sources = config
        .get("SourceConfig")
        .and_then(|v| v.as_array())
        .map(|arr| arr.is_empty())
        .unwrap_or(true);

    if needs_sources {
        if let Some(config_file) = config.get("ConfigFile").and_then(|v| v.as_str()) {
            if let Ok(config_file_value) = serde_json::from_str::<Value>(config_file) {
                if let Some(sources) = extract_sources_from_value(&config_file_value) {
                    set_value(&mut config, "SourceConfig", Value::Array(sources));
                }
            }
        }
    }

    Ok(config)
}

fn extract_sources_from_value(value: &Value) -> Option<Vec<Value>> {
    if let Some(api_site) = value.get("api_site").and_then(|v| v.as_object()) {
        return Some(normalize_api_site_object(api_site));
    }

    if let Some(sites) = value.get("sites").and_then(|v| v.as_array()) {
        return Some(normalize_source_config_array(sites, "config"));
    }

    if let Some(source_config) = value.get("SourceConfig").and_then(|v| v.as_array()) {
        return Some(normalize_source_config_array(source_config, "config"));
    }

    if let Some(arr) = value.as_array() {
        if is_source_array(arr) {
            return Some(normalize_source_config_array(arr, "config"));
        }
    }

    None
}

fn normalize_source_config_array(items: &[Value], default_from: &str) -> Vec<Value> {
    let now_ms = current_time_ms();
    items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| normalize_source_config_item(item, default_from, now_ms, idx))
        .collect()
}

fn normalize_source_config_item(
    item: &Value,
    default_from: &str,
    now_ms: i64,
    index: usize,
) -> Option<Value> {
    let mut obj = item.as_object().cloned().unwrap_or_default();

    let api = obj
        .get("api")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if api.is_empty() {
        return None;
    }

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_SOURCE_NAME)
        .trim()
        .to_string();

    let key = obj
        .get("key")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let key = if key.is_empty() {
        let slug = normalize_key_from_name(&name);
        if slug.is_empty() {
            fallback_key(now_ms, index)
        } else {
            slug
        }
    } else {
        key
    };

    let from = obj
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or(default_from)
        .to_string();

    let disabled = obj.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let existing_adult = obj
        .get("is_adult")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let is_adult = existing_adult || is_adult_source(&name);

    let detail = obj
        .get("detail")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    obj.insert("key".to_string(), Value::String(key));
    obj.insert("name".to_string(), Value::String(name));
    obj.insert("api".to_string(), Value::String(api));
    obj.insert("detail".to_string(), Value::String(detail));
    obj.insert("from".to_string(), Value::String(from));
    obj.insert("disabled".to_string(), Value::Bool(disabled));
    obj.insert("is_adult".to_string(), Value::Bool(is_adult));

    Some(Value::Object(obj))
}

fn normalize_api_site_object(api_site: &Map<String, Value>) -> Vec<Value> {
    let mut sources = Vec::new();
    for (key, value) in api_site {
        let api = value
            .get("api")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if api.is_empty() {
            continue;
        }

        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .trim()
            .to_string();
        let detail = value
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let disabled = value.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
        let existing_adult = value
            .get("is_adult")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_adult = existing_adult || is_adult_source(&name);

        let mut obj = Map::new();
        obj.insert("key".to_string(), Value::String(key.clone()));
        obj.insert("name".to_string(), Value::String(name));
        obj.insert("api".to_string(), Value::String(api));
        obj.insert("detail".to_string(), Value::String(detail));
        obj.insert("from".to_string(), Value::String("config".to_string()));
        obj.insert("disabled".to_string(), Value::Bool(disabled));
        obj.insert("is_adult".to_string(), Value::Bool(is_adult));

        sources.push(Value::Object(obj));
    }
    sources
}

fn normalize_custom_categories(items: &[Value], default_from: &str) -> Vec<Value> {
    items
        .iter()
        .filter_map(|item| {
            let mut obj = item.as_object().cloned()?;
            let from = obj
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or(default_from)
                .to_string();
            obj.insert("from".to_string(), Value::String(from));
            Some(Value::Object(obj))
        })
        .collect()
}

fn build_config_with_sources(sources: Vec<Value>) -> Value {
    let mut config = default_admin_config_value();
    set_value(&mut config, "SourceConfig", Value::Array(sources));
    config
}

fn default_admin_config_value() -> Value {
    json!({
        "ConfigFile": "",
        "ConfigSubscribtion": {
            "URL": "",
            "AutoUpdate": false,
            "LastCheck": ""
        },
        "UserPreferences": {
            "site_name": "QuantumTV",
            "announcement": "本应用仅提供影视信息搜索服务，所有内容均来自第三方网站。",
            "search_downstream_max_page": 5,
            "site_interface_cache_time": 7200,
            "disable_yellow_filter": false,
            "douban_data_source": "cmliussss-cdn-tencent",
            "douban_proxy_url": "",
            "douban_image_proxy_type": "cmliussss-cdn-tencent",
            "douban_image_proxy_url": "",
            "enable_optimization": true,
            "fluid_search": true,
            "player_buffer_mode": "standard",
            "has_seen_announcement": ""
        },
        "UserConfig": {
            "Users": [
                {
                    "username": "admin",
                    "role": "owner",
                    "banned": false
                }
            ]
        },
        "SourceConfig": [],
        "CustomCategories": []
    })
}

fn merge_object(default_obj: &Value, override_obj: &Value) -> Value {
    let mut merged = default_obj.as_object().cloned().unwrap_or_default();
    if let Some(overrides) = override_obj.as_object() {
        for (k, v) in overrides {
            merged.insert(k.clone(), v.clone());
        }
    }
    Value::Object(merged)
}

fn is_admin_config(map: &Map<String, Value>) -> bool {
    get_value(map, &["SourceConfig", "source_config"]).is_some()
        || get_value(map, &["ConfigSubscribtion", "config_subscribtion"]).is_some()
        || get_value(map, &["UserPreferences", "user_preferences"]).is_some()
}

fn is_source_array(items: &[Value]) -> bool {
    items
        .iter()
        .any(|item| item.get("api").and_then(|v| v.as_str()).is_some())
}

fn normalize_key_from_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;
    for ch in name.to_lowercase().chars() {
        if ch.is_whitespace() {
            if !last_was_underscore {
                out.push('_');
                last_was_underscore = true;
            }
        } else {
            out.push(ch);
            last_was_underscore = false;
        }
    }
    out
}

fn fallback_key(now_ms: i64, index: usize) -> String {
    if index == 0 {
        format!("source_{}", now_ms)
    } else {
        format!("source_{}_{}", now_ms, index)
    }
}

fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn get_value<'a>(map: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(value) = map.get(*key) {
            return Some(value);
        }
    }
    None
}

fn set_string(target: &mut Value, key: &str, value: &str) {
    if let Some(obj) = target.as_object_mut() {
        obj.insert(key.to_string(), Value::String(value.to_string()));
    }
}

fn set_value(target: &mut Value, key: &str, value: Value) {
    if let Some(obj) = target.as_object_mut() {
        obj.insert(key.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_invalid_json() {
        let result = parse_admin_config("{");
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_unknown_format() {
        let input = json!({ "foo": "bar" }).to_string();
        let result = parse_admin_config(&input);
        assert!(result.is_err());
    }

    #[test]
    fn parse_from_array_creates_sources() {
        let input = json!([
            { "name": "Test Source", "api": "http://example.com/api" }
        ])
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0].as_object().unwrap();
        assert_eq!(source.get("from").unwrap(), "custom");
        assert_eq!(source.get("api").unwrap(), "http://example.com/api");
        assert!(source.get("key").unwrap().as_str().unwrap().starts_with("test_source"));
    }

    #[test]
    fn parse_from_sites_format() {
        let input = json!({
            "sites": [
                { "name": "SiteA", "api": "http://a.com/api" }
            ]
        })
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0].as_object().unwrap();
        assert_eq!(source.get("from").unwrap(), "config");
        assert_eq!(source.get("name").unwrap(), "SiteA");
    }

    #[test]
    fn parse_from_api_site_format() {
        let input = json!({
            "api_site": {
                "example.com": {
                    "name": "Example",
                    "api": "http://example.com/api"
                }
            }
        })
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0].as_object().unwrap();
        assert_eq!(source.get("key").unwrap(), "example.com");
        assert_eq!(source.get("from").unwrap(), "config");
    }

    #[test]
    fn parse_admin_config_uses_config_file_when_sources_empty() {
        let embedded = json!({
            "sites": [
                { "name": "EmbeddedSite", "api": "http://embedded/api" }
            ]
        })
        .to_string();

        let input = json!({
            "ConfigFile": embedded,
            "ConfigSubscribtion": {
                "URL": "http://example.com/config.json"
            }
        })
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        assert_eq!(sources.len(), 1);
        let source = sources[0].as_object().unwrap();
        assert_eq!(source.get("name").unwrap(), "EmbeddedSite");
        assert_eq!(source.get("from").unwrap(), "config");
    }

    #[test]
    fn parse_config_file_invalid_does_not_error() {
        let input = json!({
            "ConfigFile": "{invalid_json}",
            "ConfigSubscribtion": {
                "URL": "http://example.com/config.json"
            }
        })
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        assert_eq!(sources.len(), 0);
    }

    #[test]
    fn parse_marks_adult_source() {
        let input = json!([
            { "name": "成人资源", "api": "http://adult/api" }
        ])
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
        let source = sources[0].as_object().unwrap();
        assert_eq!(source.get("is_adult").unwrap(), true);
    }

    #[test]
    fn parse_merges_defaults_for_admin_config() {
        let input = json!({
            "ConfigSubscribtion": { "URL": "http://example.com" }
        })
        .to_string();

        let result = parse_admin_config(&input).unwrap();
        let prefs = result.get("UserPreferences").unwrap().as_object().unwrap();
        assert_eq!(prefs.get("site_name").unwrap(), "QuantumTV");
        let sub = result.get("ConfigSubscribtion").unwrap().as_object().unwrap();
        assert_eq!(sub.get("URL").unwrap(), "http://example.com");
        assert_eq!(sub.get("AutoUpdate").unwrap(), false);
    }

    #[test]
    fn normalize_source_config_rejects_missing_api() {
        let input = json!({ "name": "NoApi" });
        let result = normalize_source_config(&input, "custom");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_source_config_sets_adult_flag() {
        let input = json!({ "name": "成人资源", "api": "http://adult/api" });
        let result = normalize_source_config(&input, "custom").unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("is_adult").unwrap(), true);
        assert_eq!(obj.get("from").unwrap(), "custom");
    }
}
