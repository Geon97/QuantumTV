import { invoke } from '@tauri-apps/api/core';

// 预加载队列
const preloadQueue = new Set<string>();
const preloadInProgress = new Set<string>();
const MAX_CONCURRENT_PRELOAD = 5;

/**
 * 预加载图片到 SQLite 缓存
 */
export async function preloadImage(url: string): Promise<void> {
  if (!url || preloadInProgress.has(url)) {
    return;
  }

  // 检查是否需要代理
  const needsProxy =
    url.includes('doubanio.com') ||
    (typeof window !== 'undefined' &&
      window.location.protocol === 'https:' &&
      url.startsWith('http://'));

  if (!needsProxy) {
    return;
  }

  preloadQueue.add(url);
  processPreloadQueue();
}

/**
 * 批量预加载图片
 */
export function preloadImages(urls: string[]): void {
  urls.forEach((url) => {
    if (url) {
      preloadQueue.add(url);
    }
  });
  processPreloadQueue();
}

/**
 * 处理预加载队列
 */
async function processPreloadQueue(): Promise<void> {
  if (preloadInProgress.size >= MAX_CONCURRENT_PRELOAD) {
    return;
  }

  const urlsToPreload = Array.from(preloadQueue).slice(
    0,
    MAX_CONCURRENT_PRELOAD - preloadInProgress.size
  );

  for (const url of urlsToPreload) {
    preloadQueue.delete(url);
    preloadInProgress.add(url);

    // 异步预加载，不阻塞
    invoke<number[]>('proxy_image', { url })
      .then(() => {
        preloadInProgress.delete(url);
        // 继续处理队列
        if (preloadQueue.size > 0) {
          processPreloadQueue();
        }
      })
      .catch((err) => {
        console.warn('Preload failed:', url, err);
        preloadInProgress.delete(url);
        // 继续处理队列
        if (preloadQueue.size > 0) {
          processPreloadQueue();
        }
      });
  }
}

/**
 * 清空预加载队列
 */
export function clearPreloadQueue(): void {
  preloadQueue.clear();
}
