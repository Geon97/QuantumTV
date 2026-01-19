'use client'

import { type HTMLAttributes } from 'react'

import { cn } from '@/lib/utils'

// Base Skeleton
export interface SkeletonProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'circular' | 'text'
  width?: string | number
  height?: string | number
  animate?: boolean
}

export function Skeleton({
  variant = 'default',
  width,
  height,
  animate = true,
  className,
  style,
  ...props
}: SkeletonProps) {
  const variantClasses = {
    default: 'rounded-lg',
    circular: 'rounded-full',
    text: 'rounded h-4',
  }

  return (
    <div
      className={cn(
        'skeleton',
        variantClasses[variant],
        !animate && 'after:hidden',
        className
      )}
      style={{
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
        ...style,
      }}
      {...props}
    />
  )
}

// Video Card Skeleton
export function VideoCardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn('flex flex-col', className)}>
      <Skeleton className="aspect-[2/3] rounded-xl" />
      <div className="mt-3 space-y-2">
        <Skeleton variant="text" className="w-full" />
        <Skeleton variant="text" className="w-2/3" />
      </div>
    </div>
  )
}

// List Item Skeleton
export function ListItemSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn('flex items-center gap-4 p-4', className)}>
      <Skeleton variant="circular" width={48} height={48} />
      <div className="flex-1 space-y-2">
        <Skeleton variant="text" className="w-1/2" />
        <Skeleton variant="text" className="w-3/4" />
      </div>
    </div>
  )
}

// Text Block Skeleton
export interface TextSkeletonProps {
  lines?: number
  className?: string
}

export function TextSkeleton({ lines = 3, className }: TextSkeletonProps) {
  return (
    <div className={cn('space-y-2', className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          variant="text"
          className={i === lines - 1 ? 'w-2/3' : 'w-full'}
        />
      ))}
    </div>
  )
}

// Avatar Skeleton
export function AvatarSkeleton({
  size = 'md',
  className,
}: {
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}) {
  const sizes = {
    sm: 32,
    md: 40,
    lg: 48,
    xl: 64,
  }

  return (
    <Skeleton
      variant="circular"
      width={sizes[size]}
      height={sizes[size]}
      className={className}
    />
  )
}

// Page Skeleton (for full page loading)
export function PageSkeleton() {
  return (
    <div className="container-px py-6 space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-48" />
        <div className="flex gap-3">
          <Skeleton className="h-10 w-24" />
          <Skeleton className="h-10 w-24" />
        </div>
      </div>

      {/* Content Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
        {Array.from({ length: 12 }).map((_, i) => (
          <VideoCardSkeleton key={i} />
        ))}
      </div>
    </div>
  )
}

// Video Player Skeleton
export function PlayerSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn('relative', className)}>
      <Skeleton className="aspect-video w-full rounded-xl" />
      <div className="absolute bottom-0 left-0 right-0 p-4">
        <Skeleton className="h-1 w-full mb-4" />
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Skeleton variant="circular" width={40} height={40} />
            <Skeleton className="h-4 w-20" />
          </div>
          <div className="flex items-center gap-2">
            <Skeleton variant="circular" width={32} height={32} />
            <Skeleton variant="circular" width={32} height={32} />
            <Skeleton variant="circular" width={32} height={32} />
          </div>
        </div>
      </div>
    </div>
  )
}

// Spinner
export interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}

export function Spinner({ size = 'md', className }: SpinnerProps) {
  const sizeClasses = {
    sm: 'spinner-sm',
    md: '',
    lg: 'spinner-lg',
    xl: 'spinner-xl',
  }

  return <div className={cn('spinner', sizeClasses[size], className)} />
}

// Dots Loader
export function DotsLoader({ className }: { className?: string }) {
  return (
    <div className={cn('dots-loader', className)}>
      <span />
      <span />
      <span />
    </div>
  )
}

// Cinematic Loader
export function CinematicLoader({ className }: { className?: string }) {
  return <div className={cn('cinematic-loader', className)} />
}

// Page Loading
export function PageLoading({ message }: { message?: string }) {
  return (
    <div className="page-loading">
      <CinematicLoader />
      {message && <p className="text-foreground-muted text-sm">{message}</p>}
    </div>
  )
}

export default Skeleton
