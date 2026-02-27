import { invoke } from '@tauri-apps/api/core';
import { useEffect, useRef, useState } from 'react';

// 正在进行的请求，避免重复请求
const pendingRequests = new Map<string, Promise<Uint8Array>>();

async function getImageData(originalUrl: string): Promise<Uint8Array> {
  // 检查是否有正在进行的请求
  const pending = pendingRequests.get(originalUrl);
  if (pending) {
    return pending;
  }

  // 创建新请求（proxy_image 内部会自动处理 SQLite 缓存）
  const request = (async () => {
    try {
      const imageData = await invoke<number[]>('proxy_image', { url: originalUrl });
      const data = new Uint8Array(imageData);
      pendingRequests.delete(originalUrl);
      return data;
    } catch (err) {
      console.error('Failed to load image:', err);
      pendingRequests.delete(originalUrl);
      throw err;
    }
  })();

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

        // 每次挂载都创建新的 blob URL，避免 Android WebView 二次进入后复用失效 URL
        const blob = new Blob([imageData] as any, { type: 'image/jpeg' });
        const newBlobUrl = URL.createObjectURL(blob);
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

      // 组件卸载时释放当前实例创建的 URL
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
        blobUrlRef.current = null;
      }
    };
  }, [originalUrl]);

  return { url, isLoading, error };
}
