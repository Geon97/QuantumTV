use quantumtv_core::parse_admin_config;
use serde_json::json;

#[test]
fn parse_admin_config_accepts_sites_payload() {
    let input = json!({
        "sites": [
            { "name": "SiteA", "api": "http://a.com/api" }
        ]
    })
    .to_string();

    let result = parse_admin_config(&input).unwrap();
    let sources = result.get("SourceConfig").unwrap().as_array().unwrap();
    assert_eq!(sources.len(), 1);
}
