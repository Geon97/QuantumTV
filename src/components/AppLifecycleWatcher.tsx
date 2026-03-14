'use client';

import { useEffect } from 'react';

import { resetPreloadState } from '@/lib/imagePreloader';
import { clearPendingRequests } from '@/hooks/useProxyImage';

export function AppLifecycleWatcher() {
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        clearPendingRequests();
        resetPreloadState();
        window.dispatchEvent(new CustomEvent('app-resumed'));
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  return null;
}
