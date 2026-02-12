# QuantumTV API Server Dockerfile
# 仅打包 api-server + core，不包含 Tauri 桌面应用

FROM rust:1.91 as builder

WORKDIR /app

# 复制 workspace 配置和锁文件（优化构建缓存）
COPY Cargo.toml Cargo.lock ./

# 复制 crates（只需要 core 和 api-server）
COPY crates/core ./crates/core
COPY crates/api-server ./crates/api-server

# 构建 release 版本
RUN cargo build --release -p quantumtv-api

# 运行时镜像 - 使用更小的基础镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 quantumtv

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/quantumtv-api /usr/local/bin/

# 切换到非 root 用户
USER quantumtv

# 暴露端口
EXPOSE 3000

# 设置默认环境变量
ENV RUST_LOG=info

# 启动 API 服务器
CMD ["quantumtv-api"]
