'use client';

import { useEffect, useRef } from 'react';

import { resetPreloadState } from '@/lib/imagePreloader';

// 仅在隐藏超过该阈值后再次可见才视为真正的"恢复"，避免桌面端短暂遮挡误触发。
// 图片已改为自定义协议直载（WebView 自行管理重取），恢复时不再强制重载全站图片；
// 这里只清理预加载队列，避免后台被中断的预载长期占用并发额度。
const RESUME_THRESHOLD_MS = 5_000;

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

      if (hiddenAt !== null && Date.now() - hiddenAt >= RESUME_THRESHOLD_MS) {
        resetPreloadState();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  return null;
}
