// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
// 3. Code Health（不是新功能，但有价值）
//   - 结构化日志: 有 log = "0.4" 依赖但没初始化 subscriber，代码里用的 println!
//   - Rust 测试覆盖: video, douban_client, recommendation, analytics, DB 模块缺少 #[cfg(test)]

// 4. Potential New Features
//   - 结构化日志 + 日志查看器 — 初始化 tracing subscriber，前端可查看/导出日志
//   - 离线模式 — 检测网络状态，无网时从 SQLite 缓存提供完整体验
//   - 定时任务 — 后台自动更新订阅源、清理过期缓存、预热推荐数据
//   - 性能指标收集 — Rust 端计时各操作耗时，暴露给前端的 analytics 页面

fn main() {
    tauri_temp_lib::run()
}
