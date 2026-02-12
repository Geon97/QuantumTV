import { invoke } from '@tauri-apps/api/core';
import { useCallback } from 'react';

export interface CacheStats {
  total: number;
  valid: number;
  expired: number;
}

/**
 * 页面缓存 Hook
 * 缓存有效期：24 小时
 */
export function usePageCache() {
  /**
   * 获取缓存数据
   * @param pageKey 页面唯一标识，如 'home', 'movie', 'tv', 'anime', 'variety'
   * @returns 缓存的数据（JSON 字符串）或 null
   */
  const getCache = useCallback(
    async <T = any>(pageKey: string): Promise<T | null> => {
      try {
        const cached = await invoke<string | null>('get_page_cache', {
          pageKey,
        });
        if (cached) {
          return JSON.parse(cached) as T;
        }
        return null;
      } catch (error) {
        console.error('获取页面缓存失败:', error);
        return null;
      }
    },
    [],
  );

  /**
   * 设置缓存数据
   * @param pageKey 页面唯一标识
   * @param data 要缓存的数据（会自动转为 JSON）
   */
  const setCache = useCallback(
    async (pageKey: string, data: any): Promise<void> => {
      try {
        const jsonData = JSON.stringify(data);
        await invoke('set_page_cache', { pageKey, data: jsonData });
      } catch (error) {
        console.error('设置页面缓存失败:', error);
      }
    },
    [],
  );

  /**
   * 删除指定页面的缓存
   * @param pageKey 页面唯一标识
   */
  const deleteCache = useCallback(async (pageKey: string): Promise<void> => {
    try {
      await invoke('delete_page_cache', { pageKey });
    } catch (error) {
      console.error('删除页面缓存失败:', error);
    }
  }, []);

  /**
   * 清理所有过期的缓存
   * @returns 删除的缓存数量
   */
  const cleanupExpired = useCallback(async (): Promise<number> => {
    try {
      return await invoke<number>('cleanup_expired_page_cache');
    } catch (error) {
      console.error('清理过期缓存失败:', error);
      return 0;
    }
  }, []);

  /**
   * 清空所有缓存
   */
  const clearAll = useCallback(async (): Promise<void> => {
    try {
      await invoke('clear_all_page_cache');
    } catch (error) {
      console.error('清空所有缓存失败:', error);
    }
  }, []);

  /**
   * 获取缓存统计信息
   */
  const getStats = useCallback(async (): Promise<CacheStats | null> => {
    try {
      return await invoke<CacheStats>('get_page_cache_stats');
    } catch (error) {
      console.error('获取缓存统计失败:', error);
      return null;
    }
  }, []);

  return {
    getCache,
    setCache,
    deleteCache,
    cleanupExpired,
    clearAll,
    getStats,
  };
}

/**
 * 带缓存的数据获取 Hook（支持 stale-while-revalidate）
 * @param pageKey 页面唯一标识
 * @param fetchFn 数据获取函数
 * @param options 配置选项
 */
export function useCachedData<T>(
  pageKey: string,
  fetchFn: () => Promise<T>,
  options?: {
    enabled?: boolean; // 是否启用缓存，默认 true
    forceRefresh?: boolean; // 是否强制刷新，默认 false
    staleWhileRevalidate?: boolean; // 是否使用 stale-while-revalidate 策略，默认 true
    onUpdate?: (data: T) => void; // 数据更新回调
  },
) {
  const { getCache, setCache } = usePageCache();
  const enabled = options?.enabled !== false;
  const forceRefresh = options?.forceRefresh || false;
  const staleWhileRevalidate = options?.staleWhileRevalidate !== false;
  const onUpdate = options?.onUpdate;

  const fetchData = useCallback(async (): Promise<T> => {
    // 如果禁用缓存或强制刷新，直接获取新数据
    if (!enabled || forceRefresh) {
      const data = await fetchFn();
      if (enabled) {
        await setCache(pageKey, data);
      }
      return data;
    }

    // 尝试从缓存获取
    const cached = await getCache<T>(pageKey);

    if (cached !== null) {
      // 如果启用 stale-while-revalidate，先返回缓存数据
      if (staleWhileRevalidate) {
        // 后台更新数据
        fetchFn()
          .then(async (freshData) => {
            await setCache(pageKey, freshData);
            // 如果有更新回调，通知新数据
            if (onUpdate) {
              onUpdate(freshData);
            }
          })
          .catch((error) => {
            console.error('后台更新数据失败:', error);
          });

        // 立即返回缓存数据
        return cached;
      }

      // 不使用 stale-while-revalidate，直接返回缓存
      return cached;
    }

    // 缓存未命中，获取新数据并缓存
    const data = await fetchFn();
    await setCache(pageKey, data);
    return data;
  }, [
    pageKey,
    fetchFn,
    enabled,
    forceRefresh,
    staleWhileRevalidate,
    onUpdate,
    getCache,
    setCache,
  ]);

  return { fetchData };
}
