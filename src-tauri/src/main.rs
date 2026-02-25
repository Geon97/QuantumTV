// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
// 3. 把“订阅拉取 + 解析”也完全移到 Rust（前端只传 URL/JSON）
// 4. 订阅源解析/成人过滤/清洗逻辑完全 Rust 化
//     目标：将 src/app/admin/page.tsx 的解析逻辑搬到 Rust，前端只做展示与保存。
//     实现方式：新增 parse_subscription_config（Tauri 命令），输入 JSON 或 URL，输出标准化 AdminConfig/SourceConfig。
//     质量要求：把解析逻辑抽到 crates/core，写单测覆盖 SourceConfig/sites/api_site/数组格式/ConfigFile 嵌套格式。
fn main() {
    tauri_temp_lib::run()
}
