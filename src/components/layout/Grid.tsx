'use client'

import { type ReactNode, type HTMLAttributes, forwardRef, useRef, useState, useEffect } from 'react'
import { cn } from '@/lib/utils'
import { ChevronLeft, ChevronRight } from 'lucide-react'

// Scrollable Row - 水平可滚动行
export interface ScrollableRowProps extends HTMLAttributes<HTMLDivElement> {
  showControls?: boolean
  gap?: 'sm' | 'md' | 'lg'
  padding?: boolean
}

const rowGaps = {
  sm: 'gap-2',
  md: 'gap-4',
  lg: 'gap-6',
}

export const ScrollableRow = forwardRef<HTMLDivElement, ScrollableRowProps>(
  ({ showControls = true, gap = 'md', padding = true, className, children, ...props }, ref) => {
    const scrollRef = useRef<HTMLDivElement>(null)
    const [canScrollLeft, setCanScrollLeft] = useState(false)
    const [canScrollRight, setCanScrollRight] = useState(false)

    const checkScroll = () => {
      if (scrollRef.current) {
        const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current
        setCanScrollLeft(scrollLeft > 0)
        setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 1)
      }
    }

    useEffect(() => {
      checkScroll()
      const el = scrollRef.current
      if (el) {
        el.addEventListener('scroll', checkScroll)
        const resizeObserver = new ResizeObserver(checkScroll)
        resizeObserver.observe(el)
        return () => {
          el.removeEventListener('scroll', checkScroll)
          resizeObserver.disconnect()
        }
      }
    }, [children])

    const scroll = (direction: 'left' | 'right') => {
      if (scrollRef.current) {
        const scrollAmount = scrollRef.current.clientWidth * 0.8
        scrollRef.current.scrollBy({
          left: direction === 'left' ? -scrollAmount : scrollAmount,
          behavior: 'smooth',
        })
      }
    }

    return (
      <div ref={ref} className={cn('relative group', className)} {...props}>
        {/* Scroll Controls */}
        {showControls && canScrollLeft && (
          <button
            onClick={() => scroll('left')}
            className="absolute left-0 top-1/2 -translate-y-1/2 z-10
              w-10 h-10 flex items-center justify-center
              bg-background/90 backdrop-blur border border-border rounded-full
              shadow-lg opacity-0 group-hover:opacity-100 transition-opacity
              hover:bg-background-muted"
            aria-label="向左滚动"
          >
            <ChevronLeft size={20} />
          </button>
        )}
        {showControls && canScrollRight && (
          <button
            onClick={() => scroll('right')}
            className="absolute right-0 top-1/2 -translate-y-1/2 z-10
              w-10 h-10 flex items-center justify-center
              bg-background/90 backdrop-blur border border-border rounded-full
              shadow-lg opacity-0 group-hover:opacity-100 transition-opacity
              hover:bg-background-muted"
            aria-label="向右滚动"
          >
            <ChevronRight size={20} />
          </button>
        )}

        {/* Scroll Container */}
        <div
          ref={scrollRef}
          className={cn(
            'flex overflow-x-auto scrollbar-hide',
            rowGaps[gap],
            padding && '-mx-4 px-4'
          )}
        >
          {children}
        </div>

        {/* Gradient Masks */}
        {canScrollLeft && (
          <div className="absolute left-0 top-0 bottom-0 w-12 bg-gradient-to-r from-background to-transparent pointer-events-none" />
        )}
        {canScrollRight && (
          <div className="absolute right-0 top-0 bottom-0 w-12 bg-gradient-to-l from-background to-transparent pointer-events-none" />
        )}
      </div>
    )
  }
)

ScrollableRow.displayName = 'ScrollableRow'

// Video Grid - 专门用于视频卡片的网格
export interface VideoGridProps extends HTMLAttributes<HTMLDivElement> {
  cols?: 2 | 3 | 4 | 5 | 6
}

const videoGridCols = {
  2: 'grid-cols-2',
  3: 'grid-cols-2 sm:grid-cols-3',
  4: 'grid-cols-2 sm:grid-cols-3 lg:grid-cols-4',
  5: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5',
  6: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6',
}

export const VideoGrid = forwardRef<HTMLDivElement, VideoGridProps>(
  ({ cols = 5, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn('grid gap-4', videoGridCols[cols], className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

VideoGrid.displayName = 'VideoGrid'

// Episode Grid - 剧集选择网格
export interface EpisodeGridProps extends HTMLAttributes<HTMLDivElement> {
  cols?: number
}

export const EpisodeGrid = forwardRef<HTMLDivElement, EpisodeGridProps>(
  ({ cols = 10, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn('episode-grid', className)}
        style={{
          gridTemplateColumns: `repeat(auto-fill, minmax(${Math.max(48, 100 / cols)}px, 1fr))`,
        }}
        {...props}
      >
        {children}
      </div>
    )
  }
)

EpisodeGrid.displayName = 'EpisodeGrid'

// Media Object - 媒体对象布局
export interface MediaObjectProps extends HTMLAttributes<HTMLDivElement> {
  media: ReactNode
  mediaPosition?: 'left' | 'right' | 'top'
  gap?: 'sm' | 'md' | 'lg'
}

const mediaGaps = {
  sm: 'gap-2',
  md: 'gap-4',
  lg: 'gap-6',
}

export const MediaObject = forwardRef<HTMLDivElement, MediaObjectProps>(
  ({ media, mediaPosition = 'left', gap = 'md', className, children, ...props }, ref) => {
    if (mediaPosition === 'top') {
      return (
        <div ref={ref} className={cn('flex flex-col', mediaGaps[gap], className)} {...props}>
          <div className="shrink-0">{media}</div>
          <div className="flex-1 min-w-0">{children}</div>
        </div>
      )
    }

    return (
      <div
        ref={ref}
        className={cn(
          'flex',
          mediaPosition === 'right' && 'flex-row-reverse',
          mediaGaps[gap],
          className
        )}
        {...props}
      >
        <div className="shrink-0">{media}</div>
        <div className="flex-1 min-w-0">{children}</div>
      </div>
    )
  }
)

MediaObject.displayName = 'MediaObject'

// Empty State - 空状态
export interface EmptyStateProps extends HTMLAttributes<HTMLDivElement> {
  icon?: ReactNode
  title: string
  description?: string
  action?: ReactNode
}

export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
  ...props
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center py-12 px-4 text-center',
        className
      )}
      {...props}
    >
      {icon && (
        <div className="w-16 h-16 flex items-center justify-center rounded-full bg-background-muted text-foreground-muted mb-4">
          {icon}
        </div>
      )}
      <h3 className="text-lg font-semibold text-foreground">{title}</h3>
      {description && (
        <p className="text-sm text-foreground-muted mt-1 max-w-sm">{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  )
}

// Masonry Grid - 瀑布流布局
export interface MasonryGridProps extends HTMLAttributes<HTMLDivElement> {
  columns?: number
  gap?: number
}

export function MasonryGrid({
  columns = 4,
  gap = 16,
  className,
  children,
  style,
  ...props
}: MasonryGridProps) {
  return (
    <div
      className={cn('columns-2 sm:columns-3 md:columns-4', className)}
      style={{
        columnCount: columns,
        columnGap: gap,
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  )
}

export default ScrollableRow
