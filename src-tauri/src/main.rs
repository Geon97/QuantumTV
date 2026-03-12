// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// TODO:
// 1. 插件系统
// 2. 边下边播
//   智能推荐系统 - 完整实现

//   核心特性

//   1. 混合推荐策略
//   - 用户历史数据（观看记录、收藏、搜索历史、未完成视频）
//   - 全局内容发现（从数据库中找到用户从未接触过的新内容）

//   2. 所有方法都已使用

//   ✅ ContentAnalyzer 方法
//   - batch_analyze() - 批量分析新发现的内容
//   - classify_video() - 分类用户偏好
//   - find_similar_videos() - 找到相似但未看过的内容

//   ✅ DataFusion 方法
//   - normalize_data() - 标准化数据格式
//   - batch_enhance() - 增强元数据
//   - clean_data() - 清洗低质量内容
//   - find_duplicates() - 发现重复内容
//   - fuse_videos() - 融合去重
//   - with_dedup_strategy() - 设置去重策略（Moderate）
//   - with_conflict_resolution() - 设置冲突解决策略（HighestQuality）
//   - get_fusion_stats() - 获取融合统计

//   ✅ 推荐引擎辅助方法
//   - get_videos_from_history() - 从观看历史获取
//   - get_videos_from_favorites() - 从收藏获取
//   - get_videos_from_search_history() - 从搜索历史获取
//   - get_incomplete_videos() - 从未完成视频获取

//   推荐流程（11步）

//   1. 收集用户历史数据（4个数据源）
//   2. 分析用户偏好画像
//   3. 从全局数据库发现新内容（3种策略）
//   4. 批量分析新内容质量
//   5. 合并历史和新发现内容
//   6. 标准化数据
//   7. 增强元数据
//   8. 清洗低质量内容
//   9. 查找重复项
//   10. 融合去重（使用冲突解决策略）
//   11. 按质量排序返回前20个

//   内容发现策略

//   策略1：热门搜索
//   - 从搜索历史中找到其他用户搜索但当前用户未看过的内容
//   - 使用 classify_video() 匹配用户偏好分类

//   策略2：相似内容
//   - 使用 find_similar_videos() 找到与用户观看历史相似但未看过的内容
//   - 相似度阈值：0.3

//   策略3：年份偏好
//   - 基于用户喜欢的年份发现新内容

//   现在推荐系统会真正发现新内容，而不仅仅是用户历史的重复！
fn main() {
    tauri_temp_lib::run()
}
