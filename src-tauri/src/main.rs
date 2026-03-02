// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
//   P1（第二批）
//   5. 搜索页初始化与查询合并为单编排命令
//   TS 迁移点: search/page.tsx
//   Rust 目标: search.rs 新增 search_page_open(query)
//   收益: 合并 bootstrap + query + filter categories，减少前端状态编排代码。

//   6. 设置页聚合命令（偏好+版本+更新+缓存统计）
//      TS 迁移点: UserMenu.tsx
//      Rust 目标: 新增 get_settings_bootstrap
//      收益: 减少多个 useEffect + invoke，UI 只管展示。
//   7. 页面缓存清理从前端初始化迁到 Rust 启动/定时任务
//      TS 迁移点: PageCacheInit.tsx
//      Rust 目标: 启动时在 lib.rs 或后台 task 调 cleanup_expired_page_cache
//      收益: 去掉纯运维型前端副作用逻辑。
//   8. “继续观看”数据结构直接返回可展示字段
//      收益: 去掉 parseKey/getProgress 等前端转换。

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
