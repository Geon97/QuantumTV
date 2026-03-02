// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
//   P2（收尾优化）
//   9. RUNTIME_CONFIG 读取与判定进一步下沉 Rust
//   TS 迁移点: layout.tsx, useSourceFilter.ts, douban/page.tsx
//   Rust 目标: 增加统一配置查询命令并返回稳定 DTO
//   收益: 减少前端环境分支与动态判断。

//   10. 引入命令类型自动生成（specta/tauri-specta）
//      TS 迁移点: 全局 invoke 调用点
//      Rust 目标: 命令签名统一导出，前端自动拿类型
//      收益: 少写 TS 接口和手写映射，长期维护收益高。
fn main() {
    tauri_temp_lib::run()
}
