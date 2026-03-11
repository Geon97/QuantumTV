'use client';

import Image from "next/image";
import { useState } from 'react';

import { cn } from '@/lib/utils';
import {
  generatePlaceholder,
  useProgressiveImage,
} from '@/hooks/useProgressiveImage';

interface ProgressiveImageProps {
  src: string;
  alt: string;
  lowQualitySrc?: string;
  width?: number;
  height?: number;
  className?: string;
  placeholderColor?: string;
  onLoad?: () => void;
  onError?: (error: Error) => void;
  enableBlur?: boolean;
}

/**
 * 渐进式图片组件
 *
 * 特性：
 * 1. 占位符 → 低质量 → 高质量的渐进式加载
 * 2. 平滑过渡动画
 * 3. 加载状态指示
 * 4. 错误处理
 */
export default function ProgressiveImage({
  src,
  alt,
  lowQualitySrc,
  width = 300,
  height = 200,
  className,
  placeholderColor,
  onLoad,
  onError,
  enableBlur = true,
}: ProgressiveImageProps) {
  const [placeholder] = useState(() =>
    generatePlaceholder(width, height, placeholderColor),
  );

  const {
    src: currentSrc,
    isLoading,
    quality,
    error,
  } = useProgressiveImage({
    lowQualitySrc,
    highQualitySrc: src,
    placeholder,
    onLoad,
    onError,
  });

  return (
    <div
      className={cn(
        'relative overflow-hidden bg-gray-100 dark:bg-gray-800',
        className,
      )}
    >
      {/* 图片 */}
      <Image
        src={currentSrc}
        alt={alt}
        className={cn(
          'w-full h-full object-cover transition-all duration-500',
          enableBlur && quality === 'low' && 'blur-sm scale-105',
          quality === 'high' && 'blur-0 scale-100',
        )}
        style={{
          opacity: currentSrc ? 1 : 0,
        }}
      />

      {/* 加载指示器 */}
      {isLoading && (
        <div className='absolute inset-0 flex items-center justify-center bg-gray-100/50 dark:bg-gray-800/50'>
          <div className='flex flex-col items-center gap-2'>
            <div className='w-8 h-8 border-3 border-blue-500 border-t-transparent rounded-full animate-spin' />
            <span className='text-xs text-gray-600 dark:text-gray-400'>
              {quality === 'placeholder' && '加载中...'}
              {quality === 'low' && '优化中...'}
            </span>
          </div>
        </div>
      )}

      {/* 错误状态 */}
      {error && (
        <div className='absolute inset-0 flex items-center justify-center bg-red-50 dark:bg-red-900/20'>
          <div className='text-center p-4'>
            <svg
              className='w-12 h-12 mx-auto text-red-500 mb-2'
              fill='none'
              viewBox='0 0 24 24'
              stroke='currentColor'
            >
              <path
                strokeLinecap='round'
                strokeLinejoin='round'
                strokeWidth={2}
                d='M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
              />
            </svg>
            <p className='text-sm text-red-600 dark:text-red-400'>
              图片加载失败
            </p>
          </div>
        </div>
      )}

      {/* 质量指示器（开发模式） */}
      {process.env.NODE_ENV === 'development' && !error && (
        <div className='absolute top-2 right-2 px-2 py-1 bg-black/50 text-white text-xs rounded'>
          {quality === 'placeholder' && '占位符'}
          {quality === 'low' && '低质量'}
          {quality === 'high' && '高质量'}
        </div>
      )}
    </div>
  );
}
