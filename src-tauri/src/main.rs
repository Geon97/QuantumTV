// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 添加内存缓存（moka），清理缓存，图片缓存
// 2. 插件系统
// 3. 历史记录、观影信息（配置有，但主题/当前观看等仍在前端）
// 4. 下载系统
// 5. 视频预加载 （初步实现）
// 6. 将图片预加载队列管理移入 Rust
// 7. 播放页面返回搜索页面直接显示上次搜索结果，无需重复搜索
// 8. 搜索迁移到 Rust提升速度
fn main() {
    tauri_temp_lib::run()
}
