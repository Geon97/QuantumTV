// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 从 API 开始 替代 Next.js API Routes：选择一个简单的数据获取逻辑，比如 fetchVideoDetail。
//    * 在 Rust 中创建一个等效的命令。
//    * 在 React 中用 invoke 替换原来的 fetch。
// 2. 重构文件存储：将所有对 storage.json 的操作都通过 Rust 完成。
// 3. 迁移计算任务：将 crypto.ts 或 search-ranking.ts 的逻辑用 Rust 实现。
// 4. 最后考虑配置：将 config.ts 的内容移到rust。
// 5. 清除配置源之外缓存
// 6. 应用级的核心状态（例如：当前用户、主题、配置、观看历史等）移到 Rust 中管理

fn main() {
    tauri_temp_lib::run()
}
