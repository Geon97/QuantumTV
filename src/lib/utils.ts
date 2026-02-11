/* eslint-disable @typescript-eslint/no-explicit-any */
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * 合并 CSS 类名，支持条件类名和 Tailwind CSS 类名冲突解决
 * @example cn('px-2 py-1', isActive && 'bg-blue-500', 'hover:bg-blue-600')
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// 生成存储 key
export function generateStorageKey(source: string, id: string): string {
  return `${source}+${id}`;
}

// 事件订阅辅助函数（用于组件间通信）
type CacheUpdateEvent = 'searchHistoryUpdated' | 'playRecordsUpdated' | 'favoritesUpdated';

export function subscribeToDataUpdates<T>(
  eventType: CacheUpdateEvent,
  callback: (data: T) => void,
): () => void {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const handleUpdate = (event: CustomEvent) => {
    callback(event.detail);
  };

  window.addEventListener(eventType, handleUpdate as EventListener);

  return () => {
    window.removeEventListener(eventType, handleUpdate as EventListener);
  };
}