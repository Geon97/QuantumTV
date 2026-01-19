'use client'

import { type ReactNode, type HTMLAttributes, forwardRef } from 'react'
import { cn } from '@/lib/utils'
import { ChevronRight, Play } from 'lucide-react'
import Link from 'next/link'

// Content Row - 首页内容行
export interface ContentRowProps extends HTMLAttributes<HTMLElement> {
  title: string
  subtitle?: string
  href?: string
  action?: ReactNode
}

export const ContentRow = forwardRef<HTMLElement, ContentRowProps>(
  ({ title, subtitle, href, action, className, children, ...props }, ref) => {
    return (
      <section ref={ref} className={cn('py-6', className)} {...props}>
        {/* Header */}
        <div className="flex items-center justify-between mb-4 container-px">
          <div className="flex-1 min-w-0">
            <h2 className="text-xl font-semibold text-foreground font-heading truncate">
              {title}
            </h2>
            {subtitle && (
              <p className="text-sm text-foreground-muted mt-0.5">{subtitle}</p>
            )}
          </div>
          {(href || action) && (
            <div className="shrink-0 ml-4">
              {href ? (
                <Link
                  href={href}
                  className="inline-flex items-center gap-1 text-sm text-primary-500 hover:text-primary-600 transition-colors"
                >
                  查看更多
                  <ChevronRight size={16} />
                </Link>
              ) : (
                action
              )}
            </div>
          )}
        </div>

        {/* Content */}
        {children}
      </section>
    )
  }
)

ContentRow.displayName = 'ContentRow'

// Hero Banner - 首页大横幅
export interface HeroBannerProps extends HTMLAttributes<HTMLDivElement> {
  image: string
  title: string
  subtitle?: string
  description?: string
  rating?: number
  year?: string
  tags?: string[]
  onPlay?: () => void
  onDetail?: () => void
}

export function HeroBanner({
  image,
  title,
  subtitle,
  description,
  rating,
  year,
  tags,
  onPlay,
  onDetail,
  className,
  ...props
}: HeroBannerProps) {
  return (
    <div
      className={cn(
        'relative w-full aspect-[21/9] sm:aspect-[21/8] lg:aspect-[21/7] overflow-hidden rounded-2xl',
        className
      )}
      {...props}
    >
      {/* Background Image */}
      <img
        src={image}
        alt={title}
        className="absolute inset-0 w-full h-full object-cover"
      />

      {/* Gradient Overlay */}
      <div className="absolute inset-0 bg-gradient-to-r from-black/80 via-black/40 to-transparent" />
      <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent" />

      {/* Content */}
      <div className="absolute inset-0 flex flex-col justify-end p-6 sm:p-8 lg:p-10">
        <div className="max-w-xl">
          {/* Tags & Meta */}
          <div className="flex items-center gap-2 mb-3 flex-wrap">
            {rating && (
              <span className="rating-badge rating-badge-high">
                {rating.toFixed(1)}
              </span>
            )}
            {year && (
              <span className="text-sm text-white/70">{year}</span>
            )}
            {tags?.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="px-2 py-0.5 text-xs bg-white/20 text-white rounded-full"
              >
                {tag}
              </span>
            ))}
          </div>

          {/* Title */}
          <h1 className="text-2xl sm:text-3xl lg:text-4xl font-bold text-white mb-2 line-clamp-2">
            {title}
          </h1>

          {subtitle && (
            <p className="text-lg text-white/80 mb-2">{subtitle}</p>
          )}

          {description && (
            <p className="text-sm text-white/70 line-clamp-2 mb-4 max-w-md">
              {description}
            </p>
          )}

          {/* Actions */}
          <div className="flex items-center gap-3">
            {onPlay && (
              <button
                onClick={onPlay}
                className="btn btn-lg bg-white text-black hover:bg-white/90"
              >
                <Play size={20} className="fill-current" />
                立即播放
              </button>
            )}
            {onDetail && (
              <button
                onClick={onDetail}
                className="btn btn-lg bg-white/20 text-white hover:bg-white/30 backdrop-blur"
              >
                了解详情
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

// Continue Watching Card
export interface ContinueWatchingCardProps extends HTMLAttributes<HTMLDivElement> {
  poster: string
  title: string
  episode?: string
  progress: number
  totalTime?: string
  onPlay?: () => void
  onRemove?: () => void
}

export function ContinueWatchingCard({
  poster,
  title,
  episode,
  progress,
  totalTime,
  onPlay,
  onRemove,
  className,
  ...props
}: ContinueWatchingCardProps) {
  return (
    <div
      className={cn(
        'relative group flex-shrink-0 w-[180px] sm:w-[200px] lg:w-[220px]',
        'rounded-xl overflow-hidden bg-background-subtle cursor-pointer',
        'transition-transform duration-300 hover:scale-[1.02]',
        className
      )}
      onClick={onPlay}
      {...props}
    >
      {/* Poster */}
      <div className="relative aspect-video">
        <img
          src={poster}
          alt={title}
          className="w-full h-full object-cover"
          loading="lazy"
        />
        {/* Play Overlay */}
        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
          <div className="w-12 h-12 rounded-full bg-white/90 flex items-center justify-center">
            <Play size={24} className="fill-black text-black ml-1" />
          </div>
        </div>
        {/* Progress Bar */}
        <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
          <div
            className="h-full bg-primary-500"
            style={{ width: `${Math.min(progress, 100)}%` }}
          />
        </div>
      </div>

      {/* Info */}
      <div className="p-3">
        <h3 className="font-medium text-sm text-foreground line-clamp-1">{title}</h3>
        <div className="flex items-center justify-between mt-1">
          {episode && (
            <span className="text-xs text-foreground-muted">{episode}</span>
          )}
          {totalTime && (
            <span className="text-xs text-foreground-subtle">{totalTime}</span>
          )}
        </div>
      </div>

      {/* Remove Button */}
      {onRemove && (
        <button
          onClick={(e) => {
            e.stopPropagation()
            onRemove()
          }}
          className="absolute top-2 right-2 w-6 h-6 rounded-full bg-black/60 text-white opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center hover:bg-black/80"
          aria-label="移除"
        >
          ×
        </button>
      )}
    </div>
  )
}

// Category Filter Bar
export interface CategoryFilterProps {
  categories: { id: string; label: string; icon?: ReactNode }[]
  activeCategory: string
  onCategoryChange: (id: string) => void
  className?: string
}

export function CategoryFilter({
  categories,
  activeCategory,
  onCategoryChange,
  className,
}: CategoryFilterProps) {
  return (
    <div className={cn('flex gap-2 overflow-x-auto scrollbar-hide py-2', className)}>
      {categories.map((category) => (
        <button
          key={category.id}
          onClick={() => onCategoryChange(category.id)}
          className={cn(
            'shrink-0 inline-flex items-center gap-2 px-4 py-2 rounded-full',
            'text-sm font-medium transition-all duration-200',
            activeCategory === category.id
              ? 'bg-primary-500 text-white shadow-lg shadow-primary-500/30'
              : 'bg-background-muted text-foreground-muted hover:bg-border hover:text-foreground'
          )}
        >
          {category.icon}
          {category.label}
        </button>
      ))}
    </div>
  )
}

// Ranking List
export interface RankingItemProps {
  rank: number
  title: string
  subtitle?: string
  poster?: string
  trending?: 'up' | 'down' | 'stable'
  onClick?: () => void
}

export function RankingItem({
  rank,
  title,
  subtitle,
  poster,
  trending,
  onClick,
}: RankingItemProps) {
  const rankColors = {
    1: 'text-amber-500',
    2: 'text-gray-400',
    3: 'text-amber-700',
  }

  return (
    <div
      className={cn(
        'flex items-center gap-3 p-3 rounded-xl transition-colors',
        onClick && 'cursor-pointer hover:bg-background-muted'
      )}
      onClick={onClick}
    >
      {/* Rank */}
      <span
        className={cn(
          'w-8 text-center font-bold text-lg',
          rankColors[rank as keyof typeof rankColors] || 'text-foreground-muted'
        )}
      >
        {rank}
      </span>

      {/* Poster */}
      {poster && (
        <img
          src={poster}
          alt={title}
          className="w-12 h-16 object-cover rounded-lg shrink-0"
        />
      )}

      {/* Content */}
      <div className="flex-1 min-w-0">
        <h4 className="font-medium text-sm text-foreground line-clamp-1">{title}</h4>
        {subtitle && (
          <p className="text-xs text-foreground-muted mt-0.5">{subtitle}</p>
        )}
      </div>

      {/* Trending */}
      {trending && (
        <span
          className={cn(
            'text-xs',
            trending === 'up' && 'text-success',
            trending === 'down' && 'text-error',
            trending === 'stable' && 'text-foreground-muted'
          )}
        >
          {trending === 'up' && '↑'}
          {trending === 'down' && '↓'}
          {trending === 'stable' && '−'}
        </span>
      )}
    </div>
  )
}

export interface RankingListProps extends HTMLAttributes<HTMLDivElement> {
  title: string
  items: RankingItemProps[]
}

export function RankingList({ title, items, className, ...props }: RankingListProps) {
  return (
    <div className={cn('glass-card p-4', className)} {...props}>
      <h3 className="font-semibold text-lg mb-4 font-heading">{title}</h3>
      <div className="space-y-1">
        {items.map((item, index) => (
          <RankingItem key={index} {...item} rank={item.rank || index + 1} />
        ))}
      </div>
    </div>
  )
}

export default ContentRow
