'use client';

import { useEffect, useRef } from 'react';

import { resetPreloadState } from '@/lib/imagePreloader';
import { clearPendingRequests } from '@/hooks/useProxyImage';

// 只有在隐藏超过该阈值后再次可见,才视为真正的"恢复",避免桌面端
// 短暂最小化/被其他窗口遮挡时触发 app-resumed,导致全站图片重新加载并卡住。
const RESUME_THRESHOLD_MS = 5_000; // 5秒，移动端更敏感

export function AppLifecycleWatcher() {
  const hiddenAtRef = useRef<number | null>(null);

  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'hidden') {
        hiddenAtRef.current = Date.now();
        return;
      }

      if (document.visibilityState !== 'visible') return;

      const hiddenAt = hiddenAtRef.current;
      hiddenAtRef.current = null;

      // 仅在长时间隐藏后才认为是真正的恢复(主要针对 Android WebView 场景)
      // 但即使未达到阈值，也要清理缓存，由 useProxyImage 中的 clearPendingRequests 处理
      const shouldClearCache =
        hiddenAt !== null && Date.now() - hiddenAt >= RESUME_THRESHOLD_MS;
      if (shouldClearCache) {
        clearPendingRequests();
        resetPreloadState();
      }

      // 总是发送事件，让各组件决定如何响应
      window.dispatchEvent(new CustomEvent('app-resumed'));
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  return null;
}
