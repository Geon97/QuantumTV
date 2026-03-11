use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[cfg(test)]
    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

/// 播放器初始化错误
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerInitError {
    /// 网络超时
    NetworkTimeout {
        source: String,
        timeout_ms: u64,
        stage: String, // "fetch_detail" | "test_source" | "search_fallback"
    },
    /// 源不可达
    SourceUnreachable {
        source: String,
        reason: String,
        http_status: Option<u16>,
    },
    /// 解析失败
    ParseFailed {
        source: String,
        error: String,
        content_preview: Option<String>,
    },
    /// 无可用源
    NoAvailableSources {
        attempted_sources: Vec<String>,
        last_error: Option<String>,
    },
    /// 搜索失败
    SearchFailed {
        query: String,
        reason: String,
    },
    /// 数据库错误
    DatabaseError {
        operation: String,
        error: String,
    },
    /// 配置错误
    ConfigError {
        field: String,
        error: String,
    },
}

impl PlayerInitError {
    /// 转换为用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            PlayerInitError::NetworkTimeout { source, timeout_ms, stage } => {
                format!("源 {} 在 {} 阶段超时（{}ms），请检查网络连接", source, stage, timeout_ms)
            }
            PlayerInitError::SourceUnreachable { source, reason, http_status } => {
                if let Some(status) = http_status {
                    format!("源 {} 不可达（HTTP {}）：{}", source, status, reason)
                } else {
                    format!("源 {} 不可达：{}", source, reason)
                }
            }
            PlayerInitError::ParseFailed { source, error, .. } => {
                format!("源 {} 数据解析失败：{}", source, error)
            }
            PlayerInitError::NoAvailableSources { attempted_sources, last_error } => {
                let sources = attempted_sources.join(", ");
                if let Some(err) = last_error {
                    format!("所有源均不可用（尝试了：{}）。最后错误：{}", sources, err)
                } else {
                    format!("所有源均不可用（尝试了：{}）", sources)
                }
            }
            PlayerInitError::SearchFailed { query, reason } => {
                format!("搜索 \"{}\" 失败：{}", query, reason)
            }
            PlayerInitError::DatabaseError { operation, error } => {
                format!("数据库操作 {} 失败：{}", operation, error)
            }
            PlayerInitError::ConfigError { field, error } => {
                format!("配置项 {} 错误：{}", field, error)
            }
        }
    }

    /// 获取错误代码（用于日志和埋点）
    pub fn error_code(&self) -> &'static str {
        match self {
            PlayerInitError::NetworkTimeout { .. } => "PLAYER_NETWORK_TIMEOUT",
            PlayerInitError::SourceUnreachable { .. } => "PLAYER_SOURCE_UNREACHABLE",
            PlayerInitError::ParseFailed { .. } => "PLAYER_PARSE_FAILED",
            PlayerInitError::NoAvailableSources { .. } => "PLAYER_NO_SOURCES",
            PlayerInitError::SearchFailed { .. } => "PLAYER_SEARCH_FAILED",
            PlayerInitError::DatabaseError { .. } => "PLAYER_DATABASE_ERROR",
            PlayerInitError::ConfigError { .. } => "PLAYER_CONFIG_ERROR",
        }
    }
}

impl fmt::Display for PlayerInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for PlayerInitError {}

impl From<PlayerInitError> for String {
    fn from(err: PlayerInitError) -> String {
        err.user_message()
    }
}

/// 超时配置
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// 快速测速超时（毫秒）
    pub quick_test: u64,
    /// 正常加载超时（毫秒）
    pub normal_load: u64,
    /// 降级搜索超时（毫秒）
    pub fallback_search: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            quick_test: 500,      // 500ms 快速测速
            normal_load: 2000,    // 2s 正常加载
            fallback_search: 5000, // 5s 降级搜索
        }
    }
}

impl TimeoutConfig {
    /// 获取超时配置（可从环境变量或配置文件读取）
    pub fn from_env() -> Self {
        Self {
            quick_test: std::env::var("PLAYER_QUICK_TEST_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            normal_load: std::env::var("PLAYER_NORMAL_LOAD_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),
            fallback_search: std::env::var("PLAYER_FALLBACK_SEARCH_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_init_error_user_message() {
        let error = PlayerInitError::NetworkTimeout {
            source: "test_source".to_string(),
            timeout_ms: 2000,
            stage: "fetch_detail".to_string(),
        };
        assert!(error.user_message().contains("test_source"));
        assert!(error.user_message().contains("2000"));
    }

    #[test]
    fn test_player_init_error_code() {
        let error = PlayerInitError::NetworkTimeout {
            source: "test".to_string(),
            timeout_ms: 1000,
            stage: "test".to_string(),
        };
        assert_eq!(error.error_code(), "PLAYER_NETWORK_TIMEOUT");
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.quick_test, 500);
        assert_eq!(config.normal_load, 2000);
        assert_eq!(config.fallback_search, 5000);
    }

    #[test]
    fn test_no_available_sources_error() {
        let error = PlayerInitError::NoAvailableSources {
            attempted_sources: vec!["source1".to_string(), "source2".to_string()],
            last_error: Some("Connection refused".to_string()),
        };
        let msg = error.user_message();
        assert!(msg.contains("source1"));
        assert!(msg.contains("source2"));
        assert!(msg.contains("Connection refused"));
    }

    #[test]
    fn test_source_unreachable_with_http_status() {
        let error = PlayerInitError::SourceUnreachable {
            source: "test_source".to_string(),
            reason: "Not found".to_string(),
            http_status: Some(404),
        };
        let msg = error.user_message();
        assert!(msg.contains("404"));
        assert!(msg.contains("test_source"));
    }

    #[test]
    fn test_parse_failed_error() {
        let error = PlayerInitError::ParseFailed {
            source: "test_source".to_string(),
            error: "Invalid JSON".to_string(),
            content_preview: Some("{ broken".to_string()),
        };
        assert!(error.user_message().contains("解析失败"));
        assert_eq!(error.error_code(), "PLAYER_PARSE_FAILED");
    }
}
