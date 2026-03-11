'use client';

import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

interface ImageCacheStats {
  totalImages: number;
  cacheSize: number;
  hitRate: number;
  avgLoadTime: number;
}

interface ImageCacheEntry {
  url: string;
  data: string;
  size: number;
  timestamp: number;
  hitCount: number;
}

/**
 * 图片缓存预热管理器
 *
 * 功能：
 * 1. 智能预加载即将显示的图片
 * 2. 基于用户行为预测需要预热的图片
 * 3. 管理缓存容量和过期策略
 */
export class ImageCachePreheater {
  private preheatingQueue: Set<string> = new Set();
  private preheatedUrls: Set<string> = new Set();
  private maxConcurrent = 3;
  private currentPreheating = 0;

  /**
   * 预热图片列表
   */
  async preheatImages(urls: string[], priority: 'high' | 'normal' | 'low' = 'normal'): Promise<void> {
    const filteredUrls = urls.filter((url) => !this.preheatedUrls.has(url) && !this.preheatingQueue.has(url));

    if (filteredUrls.length === 0) return;

    // 根据优先级决定并发数
    const concurrency = priority === 'high' ? 5 : priority === 'normal' ? 3 : 1;

    for (const url of filteredUrls) {
      this.preheatingQueue.add(url);
    }

    await this.processPreheatQueue(concurrency);
  }

  /**
   * 处理预热队列
   */
  private async processPreheatQueue(concurrency: number): Promise<void> {
    const urls = Array.from(this.preheatingQueue).slice(0, concurrency);

    await Promise.allSettled(
      urls.map(async (url) => {
        try {
          this.currentPreheating++;
          await this.preheatSingleImage(url);
          this.preheatedUrls.add(url);
        } catch (error) {
          console.warn(`Failed to preheat image: ${url}`, error);
        } finally {
          this.preheatingQueue.delete(url);
          this.currentPreheating--;
        }
      })
    );

    // 如果队列还有剩余，继续处理
    if (this.preheatingQueue.size > 0) {
      await this.processPreheatQueue(concurrency);
    }
  }

  /**
   * 预热单个图片
   */
  private async preheatSingleImage(url: string): Promise<void> {
    try {
      // 通过代理加载图片到缓存
      await invoke('proxy_image', { url });
    } catch (error) {
      throw new Error(`Failed to preheat: ${url}`);
    }
  }

  /**
   * 智能预测需要预热的图片
   * 基于当前可见区域和滚动方向
   */
  predictNextImages(
    allImages: string[],
    visibleIndices: number[],
    scrollDirection: 'up' | 'down' | 'none',
    lookahead = 10
  ): string[] {
    if (visibleIndices.length === 0) return [];

    const maxVisible = Math.max(...visibleIndices);
    const minVisible = Math.min(...visibleIndices);

    let predictedIndices: number[] = [];

    if (scrollDirection === 'down') {
      // 向下滚动：预加载后面的图片
      predictedIndices = Array.from({ length: lookahead }, (_, i) => maxVisible + i + 1);
    } else if (scrollDirection === 'up') {
      // 向上滚动：预加载前面的图片
      predictedIndices = Array.from({ length: lookahead }, (_, i) => minVisible - i - 1);
    } else {
      // 静止：预加载前后各一半
      const half = Math.floor(lookahead / 2);
      predictedIndices = [
        ...Array.from({ length: half }, (_, i) => minVisible - i - 1),
        ...Array.from({ length: half }, (_, i) => maxVisible + i + 1),
      ];
    }

    return predictedIndices
      .filter((idx) => idx >= 0 && idx < allImages.length)
      .map((idx) => allImages[idx])
      .filter((url) => !this.preheatedUrls.has(url));
  }

  /**
   * 清除预热缓存
   */
  clearPreheated(): void {
    this.preheatedUrls.clear();
    this.preheatingQueue.clear();
  }

  /**
   * 获取预热状态
   */
  getStatus() {
    return {
      queueSize: this.preheatingQueue.size,
      preheatedCount: this.preheatedUrls.size,
      currentPreheating: this.currentPreheating,
    };
  }
}

/**
 * 图片缓存预热 Hook
 */
export function useImageCachePreheater() {
  const [preheater] = useState(() => new ImageCachePreheater());
  const [status, setStatus] = useState(preheater.getStatus());

  useEffect(() => {
    const interval = setInterval(() => {
      setStatus(preheater.getStatus());
    }, 1000);

    return () => clearInterval(interval);
  }, [preheater]);

  return {
    preheatImages: (urls: string[], priority?: 'high' | 'normal' | 'low') =>
      preheater.preheatImages(urls, priority),
    predictNextImages: (
      allImages: string[],
      visibleIndices: number[],
      scrollDirection: 'up' | 'down' | 'none',
      lookahead?: number
    ) => preheater.predictNextImages(allImages, visibleIndices, scrollDirection, lookahead),
    clearPreheated: () => preheater.clearPreheated(),
    status,
  };
}

/**
 * 获取图片缓存统计
 */
export async function getImageCacheStats(): Promise<ImageCacheStats | null> {
  try {
    const stats = await invoke<ImageCacheStats>('get_image_cache_stats');
    return stats;
  } catch (error) {
    console.error('Failed to get image cache stats:', error);
    return null;
  }
}

/**
 * 清除图片缓存
 */
export async function clearImageCache(): Promise<void> {
  try {
    await invoke('clear_image_cache');
  } catch (error) {
    console.error('Failed to clear image cache:', error);
    throw error;
  }
}
