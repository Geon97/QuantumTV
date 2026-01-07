'use client';

import { Star } from 'lucide-react';
import { memo, useMemo } from 'react';

interface RatingBadgeProps {
  /** 评分数值 (0-10) */
  rating: number | string | null | undefined;
  /** 尺寸: sm, md, lg */
  size?: 'sm' | 'md' | 'lg';
  /** 是否显示星星图标 */
  showIcon?: boolean;
  /** 自定义类名 */
  className?: string;
  /** 是否使用绝对定位 */
  absolute?: boolean;
}

/**
 * 高质感评分徽章组件
 * 参考 Netflix/IMDb 风格设计
 *
 * - 高分 (≥7): 金色/绿色渐变
 * - 中分 (5-6.9): 蓝色
 * - 低分 (<5): 灰色
 */
function RatingBadgeComponent({
  rating,
  size = 'md',
  showIcon = true,
  className = '',
  absolute = true,
}: RatingBadgeProps) {
  // 解析评分
  const numericRating = useMemo(() => {
    if (rating === null || rating === undefined || rating === '') {
      return null;
    }
    const parsed = typeof rating === 'string' ? parseFloat(rating) : rating;
    return isNaN(parsed) ? null : parsed;
  }, [rating]);

  // 如果没有有效评分，不显示
  if (numericRating === null || numericRating === 0) {
    return null;
  }

  // 格式化评分显示
  const displayRating =
    numericRating % 1 === 0
      ? numericRating.toFixed(0)
      : numericRating.toFixed(1);

  // 根据评分获取样式
  const getGradientStyle = () => {
    if (numericRating >= 8) {
      // 高分: 金色渐变
      return 'bg-gradient-to-br from-yellow-400 via-amber-500 to-orange-500';
    } else if (numericRating >= 7) {
      // 良好: 绿色渐变
      return 'bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500';
    } else if (numericRating >= 5) {
      // 中等: 蓝色
      return 'bg-gradient-to-br from-blue-400 via-indigo-500 to-purple-500';
    } else {
      // 低分: 灰色
      return 'bg-gradient-to-br from-gray-400 via-gray-500 to-gray-600';
    }
  };

  // 尺寸样式
  const sizeStyles = {
    sm: {
      container: 'px-1.5 py-0.5 gap-0.5',
      text: 'text-xs font-semibold',
      icon: 'w-2.5 h-2.5',
    },
    md: {
      container: 'px-2 py-1 gap-1',
      text: 'text-sm font-bold',
      icon: 'w-3 h-3',
    },
    lg: {
      container: 'px-2.5 py-1.5 gap-1',
      text: 'text-base font-bold',
      icon: 'w-4 h-4',
    },
  };

  const currentSize = sizeStyles[size];

  return (
    <div
      className={`
        inline-flex items-center
        ${currentSize.container}
        ${getGradientStyle()}
        rounded-md
        border border-white/20
        shadow-lg
        backdrop-blur-sm
        ${absolute ? 'absolute top-2 right-2 z-10' : ''}
        ${className}
      `}
      style={{
        boxShadow:
          numericRating >= 7
            ? '0 4px 14px rgba(0, 0, 0, 0.3), 0 0 20px rgba(255, 200, 50, 0.2)'
            : '0 4px 14px rgba(0, 0, 0, 0.3)',
      }}
    >
      {showIcon && (
        <Star
          className={`${currentSize.icon} text-white fill-white drop-shadow-sm`}
        />
      )}
      <span
        className={`
          ${currentSize.text}
          text-white
          drop-shadow-md
          tracking-tight
        `}
        style={{
          textShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
        }}
      >
        {displayRating}
      </span>
    </div>
  );
}

export const RatingBadge = memo(RatingBadgeComponent);

// ============================================================================
// 简化版评分徽章 (用于列表卡片)
// ============================================================================

interface SimpleRatingBadgeProps {
  rating: number | string | null | undefined;
  className?: string;
}

/**
 * 简化版评分徽章
 * 磨砂玻璃黑风格，适用于视频卡片
 */
function SimpleRatingBadgeComponent({
  rating,
  className = '',
}: SimpleRatingBadgeProps) {
  // 解析评分
  const numericRating = useMemo(() => {
    if (rating === null || rating === undefined || rating === '') {
      return null;
    }
    const parsed = typeof rating === 'string' ? parseFloat(rating) : rating;
    return isNaN(parsed) ? null : parsed;
  }, [rating]);

  if (numericRating === null || numericRating === 0) {
    return null;
  }

  const displayRating =
    numericRating % 1 === 0
      ? numericRating.toFixed(0)
      : numericRating.toFixed(1);

  // 星星颜色根据评分变化
  const getStarColor = () => {
    if (numericRating >= 8) return 'text-yellow-400';
    if (numericRating >= 7) return 'text-green-400';
    if (numericRating >= 5) return 'text-blue-400';
    return 'text-gray-400';
  };

  return (
    <div
      className={`
        absolute top-2 right-2 z-10
        inline-flex items-center gap-1
        px-2 py-1
        bg-black/60 backdrop-blur-md
        rounded-md
        border border-white/10
        shadow-lg
        ${className}
      `}
    >
      <Star className={`w-3 h-3 ${getStarColor()} fill-current`} />
      <span
        className='text-sm font-bold text-white drop-shadow-md'
        style={{ textShadow: '0 1px 2px rgba(0, 0, 0, 0.5)' }}
      >
        {displayRating}
      </span>
    </div>
  );
}

export const SimpleRatingBadge = memo(SimpleRatingBadgeComponent);

export default RatingBadge;
