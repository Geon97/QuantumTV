import { invoke } from '@tauri-apps/api/core';
import { useEffect, useRef, useState } from 'react';

// 全局缓存：存储图片的原始数据（字节数组），而不是 blob URL
const imageDataCache = new Map<string, Uint8Array>();
// 正在进行的请求，避免重复请求
const pendingRequests = new Map<string, Promise<Uint8Array>>();
// 全局 Blob URL 缓存，避免重复创建
const blobUrlCache = new Map<string, string>();

async function getImageData(originalUrl: string): Promise<Uint8Array> {
  // 检查缓存
  const cached = imageDataCache.get(originalUrl);
  if (cached) {
    return cached;
  }

  // 检查是否有正在进行的请求
  const pending = pendingRequests.get(originalUrl);
  if (pending) {
    return pending;
  }

  // 创建新请求
  const request = invoke<number[]>('proxy_image', { url: originalUrl })
    .then((imageData) => {
      const data = new Uint8Array(imageData);
      // 缓存原始数据
      imageDataCache.set(originalUrl, data);
      pendingRequests.delete(originalUrl);
      return data;
    })
    .catch((err) => {
      console.error('Failed to load image via Tauri:', err);
      pendingRequests.delete(originalUrl);
      throw err;
    });

  pendingRequests.set(originalUrl, request);
  return request;
}

const PLACEHOLDER = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg"%3E%3C/svg%3E';

export function useProxyImage(originalUrl: string): {
  url: string;
  isLoading: boolean;
  error: Error | null;
} {
  const [url, setUrl] = useState(PLACEHOLDER);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const blobUrlRef = useRef<string | null>(null);

  useEffect(() => {
    if (!originalUrl) {
      setUrl('');
      setIsLoading(false);
      setError(null);
      return;
    }

    // 判断是否需要代理
    const needsProxy =
      originalUrl.includes('doubanio.com') ||
      (typeof window !== 'undefined' &&
        window.location.protocol === 'https:' &&
        originalUrl.startsWith('http://'));

    if (!needsProxy) {
      setUrl(originalUrl);
      setIsLoading(false);
      setError(null);
      return;
    }

    // 加载图片数据
    setIsLoading(true);
    setError(null);

    let cancelled = false;

    getImageData(originalUrl)
      .then((imageData) => {
        if (cancelled) return;

        // 检查是否已有缓存的 Blob URL
        let newBlobUrl = blobUrlCache.get(originalUrl);

        if (!newBlobUrl) {
          // 只在没有缓存时创建新的 blob URL
          const blob = new Blob([imageData] as any, { type: 'image/jpeg' });
          newBlobUrl = URL.createObjectURL(blob);
          blobUrlCache.set(originalUrl, newBlobUrl);
        }

        // 清理旧的 blob URL（如果与新的不同）
        if (blobUrlRef.current && blobUrlRef.current !== newBlobUrl) {
          URL.revokeObjectURL(blobUrlRef.current);
        }

        blobUrlRef.current = newBlobUrl;
        setUrl(newBlobUrl);
        setIsLoading(false);
      })
      .catch((err) => {
        if (cancelled) return;

        setError(err);
        setIsLoading(false);
        // Fallback 到原始 URL
        setUrl(originalUrl);
      });

    return () => {
      cancelled = true;
    };
  }, [originalUrl]);

  // 组件卸载时清理 blob URL
  useEffect(() => {
    return () => {
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
        blobUrlRef.current = null;
      }
    };
  }, []);

  return { url, isLoading, error };
}
