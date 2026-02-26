// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
// 5. 继续播放 源无法打开或存在集数不对
// 6. 搜索页面排序未生效
fn main() {
    tauri_temp_lib::run()
}
