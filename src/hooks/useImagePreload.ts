import { useEffect } from 'react';

import { preloadImages } from '@/lib/imagePreloader';

/**
 * 预加载可见区域附近的图片
 */
export function useImagePreload(images: string[], enabled: boolean = true) {
  useEffect(() => {
    if (!enabled || !images || images.length === 0) {
      return;
    }

    // 延迟预加载，避免阻塞首屏渲染
    const timer = setTimeout(() => {
      preloadImages(images);
    }, 500);

    return () => clearTimeout(timer);
  }, [images, enabled]);
}

/**
 * 使用 Intersection Observer 预加载即将进入视口的图片
 */
export function useIntersectionPreload(
  containerRef: React.RefObject<HTMLElement>,
  images: string[],
  // eslint-disable-next-line no-undef
  options?: IntersectionObserverInit
) {
  useEffect(() => {
    if (!containerRef.current || !images || images.length === 0) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            // 当容器进入视口时，预加载图片
            const imagesToPreload = images.slice(0, 10); // 预加载前 10 张
            preloadImages(imagesToPreload);
          }
        });
      },
      {
        rootMargin: '200px', // 提前 200px 开始预加载
        ...options,
      }
    );

    observer.observe(containerRef.current);

    return () => {
      observer.disconnect();
    };
  }, [containerRef, images, options]);
}
