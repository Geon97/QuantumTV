import { invoke } from '@tauri-apps/api/core';
import { useRef } from 'react';

interface VideoItem {
  title: string;
  source_name: string;
  year?: string;
  cover?: string;
  category?: string;
  rating?: number;
  description?: string;
  tags?: string;
}

/**
 * 自动同步内容到内容池的 Hook
 *
 * 功能：
 * 1. 监听页面上的视频卡片数据
 * 2. 自动提取元数据并填充到内容池
 * 3. 更新图片缓存的元数据
 */
export function useContentPoolSync() {
  const syncedItems = useRef(new Set<string>());

  /**
   * 同步单个视频到内容池
   */
  const syncToContentPool = async (item: VideoItem) => {
    const key = `${item.title}-${item.source_name}`;

    // 避免重复同步
    if (syncedItems.current.has(key)) {
      return;
    }

    try {
      await invoke('add_to_content_pool', {
        title: item.title,
        sourceName: item.source_name,
        year: item.year || null,
        cover: item.cover || null,
        category: item.category || null,
        rating: item.rating || null,
        description: item.description || null,
        tags: item.tags || null,
      });

      // 如果有封面，更新图片缓存元数据
      if (item.cover) {
        await invoke('update_image_cache_metadata', {
          url: item.cover,
          title: item.title,
          sourceName: item.source_name,
          year: item.year || null,
          category: item.category || null,
          rating: item.rating || null,
        });
      }

      syncedItems.current.add(key);
    } catch (error) {
      console.error('同步内容池失败:', error);
    }
  };

  /**
   * 批量同步视频到内容池
   */
  const batchSyncToContentPool = async (items: VideoItem[]) => {
    const newItems = items.filter((item) => {
      const key = `${item.title}-${item.source_name}`;
      return !syncedItems.current.has(key);
    });

    if (newItems.length === 0) {
      return;
    }

    try {
      const count = await invoke<number>('batch_add_to_content_pool', {
        items: newItems.map((item) => ({
          title: item.title,
          source_name: item.source_name,
          year: item.year || '',
          cover: item.cover || '',
          category: item.category || '',
          rating: item.rating || 0,
          description: item.description || '',
          tags: item.tags || '',
        })),
      });

      console.log(`成功同步 ${count} 个内容到内容池`);

      // 批量更新图片缓存元数据
      for (const item of newItems) {
        if (item.cover) {
          try {
            await invoke('update_image_cache_metadata', {
              url: item.cover,
              title: item.title,
              sourceName: item.source_name,
              year: item.year || null,
              category: item.category || null,
              rating: item.rating || null,
            });
          } catch (error) {
            // 忽略单个更新失败
          }
        }

        const key = `${item.title}-${item.source_name}`;
        syncedItems.current.add(key);
      }
    } catch (error) {
      console.error('批量同步内容池失败:', error);
    }
  };

  /**
   * 清除同步记录（用于重置）
   */
  const clearSyncHistory = () => {
    syncedItems.current.clear();
  };

  return {
    syncToContentPool,
    batchSyncToContentPool,
    clearSyncHistory,
  };
}

/**
 * 从豆瓣数据提取视频信息
 */
export function extractFromDoubanItem(item: any): VideoItem {
  // 根据豆瓣的 type 字段智能判断分类
  let category = item.type || '';

  // 豆瓣的 type 可能是：movie, tv, show 等
  if (category === 'movie') {
    category = 'Movie';
  } else if (category === 'tv') {
    category = 'TvSeries';
  } else if (category === 'show') {
    category = 'Variety';
  }

  return {
    title: item.title || item.name || '',
    source_name: '豆瓣',
    year: item.year || '',
    cover: item.poster || item.cover || '',
    category,
    rating: item.rating || 0,
    description: item.intro || item.description || '',
    tags: item.genres?.join(',') || '',
  };
}

/**
 * 从搜索结果提取视频信息
 */
export function extractFromSearchResult(item: any): VideoItem {
  // 根据集数判断分类
  let category = item.type || item.vod_type || '';

  // 如果没有明确分类，根据集数判断
  if (!category && item.episodes) {
    category = item.episodes > 1 ? 'TvSeries' : 'Movie';
  }

  return {
    title: item.title || item.vod_name || '',
    source_name: item.source_name || item.source || '',
    year: item.year || item.vod_year || '',
    cover: item.cover || item.vod_pic || '',
    category,
    rating: 0,
    description: item.description || item.vod_content || '',
    tags: item.tags || '',
  };
}

/**
 * 从番剧数据提取视频信息
 */
export function extractFromBangumiItem(item: any): VideoItem {
  return {
    title: item.name || item.name_cn || '',
    source_name: 'Bangumi',
    year: item.air_date?.split('-')[0] || '',
    cover: item.images?.large || item.images?.common || '',
    category: 'Anime',
    rating: item.rating?.score || 0,
    description: item.summary || '',
    tags: item.tags?.map((t: any) => t.name).join(',') || '',
  };
}
