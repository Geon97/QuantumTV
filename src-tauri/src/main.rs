// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 从 API 开始 替代 Next.js API Routes：选择一个简单的数据获取逻辑，比如 fetchVideoDetail。
//    * 在 Rust 中创建一个等效的命令。
//    * 在 React 中用 invoke 替换原来的 fetch。
// 2. 重构文件存储：将所有对 storage.json 的操作都通过 Rust 完成。
// 3. 最后考虑配置：将 lib 的内容移到rust。
//   1. 第一步: 从 crypto.ts 和 db.client.ts 开始，将加密和本地存储这两个最适合后端化的功能迁移到 Rust。
//   2. 第二步: 迁移 config.ts，建立起由 Rust 管理应用配置的基础。
//   3. 第三步: 集中处理所有网络请求，将 douban.client.ts、downstream.ts、search-cache.ts 和 bangumi.client.ts
//       的逻辑全部移入 Rust 后端。
//   4. 第四步: 迁移计算密集型任务 search-ranking.ts，优化搜索排序性能。
//   5. 最后: 考虑 spiderJar.ts 和 chinese.ts 等其他模块。
// 4. 清除配置源之外缓存
// 5. 应用级的核心状态（例如：当前用户、主题、配置、观看历史等）移到 Rust 中管理
// 6. 页面跳转卡顿
// 7. 老板键 Ctrl+Alt+X
fn main() {
    tauri_temp_lib::run()
}
