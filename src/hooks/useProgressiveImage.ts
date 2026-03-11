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
        if (lowQualitySrc) {
          await loadImage(lowQualitySrc, 'low');
        }
        await loadImage(highQualitySrc, 'high');
      } catch (error) {
        console.error('Progressive image loading failed:', error);
      }
    };

    loadSequence();

    return () => {
      isMounted = false;
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
 * 生成 Base64 占位符
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
