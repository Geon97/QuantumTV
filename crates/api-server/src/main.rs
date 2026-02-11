// QuantumTV API Server (Axum)

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 配置 CORS - 允许所有来源
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建路由
    let app = Router::new().route("/", get(health_check)).layer(cors);

    // 绑定地址
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("QuantumTV API Server listening on {}", addr);

    // 启动服务器
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

/// 健康检查
async fn health_check() -> &'static str {
    "QuantumTV API Server is running"
}
