use axum::{routing::get, Router};
use moka::future::Cache;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
mod tvbox;
use crate::tvbox::{
    get_config_handler, proxy_m3u8_handler, proxy_spider_jar_handler, proxy_ts_handler,
    CachedSubscription, FailedSources, SpiderInfo,
};

static SERVER_IP: LazyLock<String> =
    LazyLock::new(|| std::env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string()));

#[derive(Clone)]
struct AppState {
    spider_info: Arc<Mutex<SpiderInfo>>,
    subscription_cache: Arc<Mutex<Option<CachedSubscription>>>,
    failed_sources: Arc<Mutex<FailedSources>>,
    // TS 视频片段缓存（URL -> 片段数据）
    ts_cache: Cache<String, Vec<u8>>,
}
async fn health_check() -> &'static str {
    "QuantumTV API Server is running"
}

// ================== Main ==================
#[tokio::main]
async fn main() {
    // 0. 加载 .env 文件
    dotenvy::dotenv().ok();

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
        // TS 片段缓存：最大 500 个片段，每个最大 10MB，总共约 5GB
        ts_cache: Cache::builder()
            .max_capacity(500)
            .time_to_live(std::time::Duration::from_secs(3600)) // 1 小时过期
            .build(),
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
        // M3U8 代理路由（带广告过滤和 URL 重写）
        .route("/api/proxy/m3u8", get(proxy_m3u8_handler))
        // TS 视频片段代理路由（带缓存加速）
        .route("/api/proxy/ts", get(proxy_ts_handler))
        // Spider JAR 代理路由
        .route("/api/proxy/spider.jar", get(proxy_spider_jar_handler))
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
