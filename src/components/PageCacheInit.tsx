'use client';

import { useEffect } from 'react';
import { usePageCache } from '@/hooks/usePageCache';

/**
 * 页面缓存初始化组件
 * - 应用启动时自动清理过期缓存
 */
export function PageCacheInit() {
  const { cleanupExpired } = usePageCache();

  useEffect(() => {
    // 应用启动时清理过期缓存
    const cleanup = async () => {
      try {
        const deletedCount = await cleanupExpired();
        if (deletedCount > 0) {
          console.log(`[PageCache] 清理了 ${deletedCount} 个过期缓存`);
        }
      } catch (error) {
        console.error('[PageCache] 清理过期缓存失败:', error);
      }
    };

    cleanup();
  }, [cleanupExpired]);

  return null; // 不渲染任何内容
}
