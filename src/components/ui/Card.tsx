'use client'

import { forwardRef, type HTMLAttributes, type ReactNode } from 'react'
import { cn } from '@/lib/utils'

// Card variants
const cardVariants = {
  default: 'glass-card',
  solid: 'bg-background border border-border rounded-2xl',
  ghost: 'bg-transparent',
  elevated: 'bg-background-subtle border border-border rounded-2xl shadow-lg',
} as const

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: keyof typeof cardVariants
  hover?: boolean
  padding?: 'none' | 'sm' | 'md' | 'lg'
}

const paddingMap = {
  none: '',
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
}

export const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ className, variant = 'default', hover = false, padding = 'md', children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          cardVariants[variant],
          paddingMap[padding],
          hover && 'cursor-pointer transition-transform hover:scale-[1.02]',
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Card.displayName = 'Card'

// Card Header
export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
  title?: string
  subtitle?: string
  action?: ReactNode
}

export const CardHeader = forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ className, title, subtitle, action, children, ...props }, ref) => {
    if (children) {
      return (
        <div ref={ref} className={cn('flex items-center justify-between mb-4', className)} {...props}>
          {children}
        </div>
      )
    }

    return (
      <div ref={ref} className={cn('flex items-center justify-between mb-4', className)} {...props}>
        <div>
          {title && <h3 className="text-title text-lg font-semibold">{title}</h3>}
          {subtitle && <p className="text-caption text-sm mt-0.5">{subtitle}</p>}
        </div>
        {action && <div>{action}</div>}
      </div>
    )
  }
)

CardHeader.displayName = 'CardHeader'

// Card Body
export const CardBody = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn('', className)} {...props}>
        {children}
      </div>
    )
  }
)

CardBody.displayName = 'CardBody'

// Card Footer
export const CardFooter = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn('flex items-center gap-3 mt-4 pt-4 border-t border-border', className)} {...props}>
        {children}
      </div>
    )
  }
)

CardFooter.displayName = 'CardFooter'

// Video Card (specialized)
export interface VideoCardProps extends HTMLAttributes<HTMLDivElement> {
  poster?: string
  title: string
  subtitle?: string
  rating?: number
  progress?: number
  badge?: ReactNode
}

export const VideoCard = forwardRef<HTMLDivElement, VideoCardProps>(
  ({ className, poster, title, subtitle, rating, progress, badge, onClick, ...props }, ref) => {
    const getRatingClass = (r: number) => {
      if (r >= 8) return 'rating-badge-high'
      if (r >= 6) return 'rating-badge-medium'
      return 'rating-badge-low'
    }

    return (
      <div
        ref={ref}
        className={cn('video-card group', className)}
        onClick={onClick}
        role={onClick ? 'button' : undefined}
        tabIndex={onClick ? 0 : undefined}
        {...props}
      >
        {/* Poster */}
        <div className="relative aspect-[2/3] overflow-hidden">
          {poster ? (
            <img
              src={poster}
              alt={title}
              className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
              loading="lazy"
            />
          ) : (
            <div className="w-full h-full bg-background-muted flex items-center justify-center">
              <svg className="w-12 h-12 text-foreground-subtle" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
          )}

          {/* Overlay */}
          <div className="video-card-overlay" />

          {/* Rating Badge */}
          {rating !== undefined && rating > 0 && (
            <div className={cn('rating-badge absolute top-2 right-2', getRatingClass(rating))}>
              {rating.toFixed(1)}
            </div>
          )}

          {/* Custom Badge */}
          {badge && (
            <div className="absolute top-2 left-2">
              {badge}
            </div>
          )}

          {/* Progress Bar */}
          {progress !== undefined && progress > 0 && (
            <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
              <div
                className="h-full bg-gradient-to-r from-primary-500 to-aurora-fuchsia"
                style={{ width: `${Math.min(progress, 100)}%` }}
              />
            </div>
          )}
        </div>

        {/* Content */}
        <div className="p-3">
          <h4 className="font-medium text-sm line-clamp-2 text-foreground">{title}</h4>
          {subtitle && (
            <p className="text-xs text-foreground-muted mt-1 line-clamp-1">{subtitle}</p>
          )}
        </div>
      </div>
    )
  }
)

VideoCard.displayName = 'VideoCard'

export default Card
