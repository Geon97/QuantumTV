'use client';

import { useEffect, useState } from 'react';

interface ProgressiveImageOptions {
  lowQualitySrc?: string;
  highQualitySrc: string;
  placeholder?: string;
  onLoad?: () => void;
  onError?: (error: Error) => void;
}

interface ProgressiveImageState {
  src: string;
  isLoading: boolean;
  isLoaded: boolean;
  error: Error | null;
  quality: 'placeholder' | 'low' | 'high';
}

/**
 * 渐进式图片加载 Hook
 *
 * 加载流程：
 * 1. 显示占位符（可选）
 * 2. 加载低质量图片（可选）
 * 3. 加载高质量图片
 *
 * @example
 * const { src, isLoading, quality } = useProgressiveImage({
 *   lowQualitySrc: '/thumb.jpg',
 *   highQualitySrc: '/full.jpg',
 *   placeholder: 'data:image/svg+xml,...'
 * });
 */
export function useProgressiveImage({
  lowQualitySrc,
  highQualitySrc,
  placeholder,
  onLoad,
  onError,
}: ProgressiveImageOptions): ProgressiveImageState {
  const [state, setState] = useState<ProgressiveImageState>({
    src: placeholder || '',
    isLoading: true,
    isLoaded: false,
    error: null,
    quality: 'placeholder',
  });

  useEffect(() => {
    let isMounted = true;
    const images: HTMLImageElement[] = [];

    const loadImage = (src: string, quality: 'low' | 'high'): Promise<void> => {
      return new Promise((resolve, reject) => {
        const img = new Image();
        images.push(img);

        img.onload = () => {
          if (isMounted) {
            setState((prev) => ({
              ...prev,
              src,
              quality,
              isLoading: quality !== 'high',
              isLoaded: quality === 'high',
            }));
            if (quality === 'high') {
              onLoad?.();
            }
          }
          resolve();
        };

        img.onerror = () => {
          const error = new Error(`Failed to load ${quality} quality image: ${src}`);
          if (isMounted) {
            setState((prev) => ({
              ...prev,
              error,
              isLoading: false,
            }));
            onError?.(error);
          }
          reject(error);
        };

        img.src = src;
      });
    };

    const loadSequence = async () => {
      try {
        // 1. 加载低质量图片（如果提供）
        if (lowQualitySrc) {
          await loadImage(lowQualitySrc, 'low');
        }

        // 2. 加载高质量图片
        await loadImage(highQualitySrc, 'high');
      } catch (error) {
        // 错误已在 loadImage 中处理
        console.error('Progressive image loading failed:', error);
      }
    };

    loadSequence();

    return () => {
      isMounted = false;
      // 取消所有图片加载
      images.forEach((img) => {
        img.onload = null;
        img.onerror = null;
        img.src = '';
      });
    };
  }, [lowQualitySrc, highQualitySrc, placeholder, onLoad, onError]);

  return state;
}

/**
 * 生成 Base64 占位符（模糊效果）
 */
export function generatePlaceholder(width: number, height: number, color = '#e5e7eb'): string {
  const svg = `
    <svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">
      <rect width="100%" height="100%" fill="${color}"/>
      <text x="50%" y="50%" font-family="Arial" font-size="14" fill="#9ca3af" text-anchor="middle" dominant-baseline="middle">
        Loading...
      </text>
    </svg>
  `;
  return `data:image/svg+xml;base64,${btoa(svg)}`;
}

/**
 * 预加载图片
 */
export function preloadImage(src: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve();
    img.onerror = () => reject(new Error(`Failed to preload image: ${src}`));
    img.src = src;
  });
}

/**
 * 批量预加载图片
 */
export async function preloadImages(
  srcs: string[],
  options?: {
    concurrency?: number;
    onProgress?: (loaded: number, total: number) => void;
  }
): Promise<void> {
  const { concurrency = 3, onProgress } = options || {};
  const total = srcs.length;
  let loaded = 0;

  const loadBatch = async (batch: string[]) => {
    await Promise.allSettled(
      batch.map(async (src) => {
        try {
          await preloadImage(src);
          loaded++;
          onProgress?.(loaded, total);
        } catch (error) {
          console.warn(`Failed to preload: ${src}`, error);
          loaded++;
          onProgress?.(loaded, total);
        }
      })
    );
  };

  // 分批加载
  for (let i = 0; i < srcs.length; i += concurrency) {
    const batch = srcs.slice(i, i + concurrency);
    await loadBatch(batch);
  }
}
