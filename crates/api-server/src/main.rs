use axum::{routing::get, Router};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
mod tvbox;
use crate::tvbox::{
    get_config_handler, proxy_m3u8_handler, CachedSubscription, FailedSources, SpiderInfo,
};

static SERVER_IP: LazyLock<String> =
    LazyLock::new(|| std::env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string()));

#[derive(Clone)]
struct AppState {
    spider_info: Arc<Mutex<SpiderInfo>>,
    subscription_cache: Arc<Mutex<Option<CachedSubscription>>>,
    failed_sources: Arc<Mutex<FailedSources>>,
}
async fn health_check() -> &'static str {
    "QuantumTV API Server is running"
}

// ================== Main ==================
#[tokio::main]
async fn main() {
    // 1. 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 2. 初始化应用状态
    let state = AppState {
        spider_info: Arc::new(Mutex::new(SpiderInfo {
            buffer: None,
            md5: String::new(),
            source: String::new(),
            success: false,
            cached: false,
            timestamp: std::time::SystemTime::now(),
            size: 0,
            tried: 0,
        })),
        subscription_cache: Arc::new(Mutex::new(None)),
        failed_sources: Arc::new(Mutex::new(FailedSources {
            sources: std::collections::HashSet::new(),
            last_reset: std::time::SystemTime::now(),
        })),
    };

    // 3. 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 4. 创建路由
    let app = Router::new()
        .route("/", get(health_check))
        // 注册配置接口路由
        .route("/api/tvbox", get(get_config_handler))
        // M3U8 代理路由（带广告过滤）
        .route("/api/proxy/m3u8", get(proxy_m3u8_handler))
        .layer(cors)
        // 5. 注入状态
        .with_state(state);

    let addr = SocketAddr::from((SERVER_IP.parse::<Ipv4Addr>().unwrap(), 3000));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("QuantumTV API Server listening on {}", addr);

    // 启动服务器
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
