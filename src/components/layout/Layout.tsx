'use client'

import { type ReactNode, type HTMLAttributes, forwardRef } from 'react'
import { cn } from '@/lib/utils'

// Container - 响应式容器
export interface ContainerProps extends HTMLAttributes<HTMLDivElement> {
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  padding?: boolean
}

const containerSizes = {
  sm: 'max-w-2xl',
  md: 'max-w-4xl',
  lg: 'max-w-6xl',
  xl: 'max-w-7xl',
  full: 'max-w-full',
}

export const Container = forwardRef<HTMLDivElement, ContainerProps>(
  ({ size = 'xl', padding = true, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          'mx-auto w-full',
          containerSizes[size],
          padding && 'container-px',
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Container.displayName = 'Container'

// Section - 内容区块
export interface SectionProps extends HTMLAttributes<HTMLElement> {
  title?: string
  subtitle?: string
  action?: ReactNode
  spacing?: 'sm' | 'md' | 'lg'
}

const sectionSpacing = {
  sm: 'py-4',
  md: 'py-6',
  lg: 'py-10',
}

export const Section = forwardRef<HTMLElement, SectionProps>(
  ({ title, subtitle, action, spacing = 'md', className, children, ...props }, ref) => {
    return (
      <section ref={ref} className={cn(sectionSpacing[spacing], className)} {...props}>
        {(title || subtitle || action) && (
          <div className="flex items-center justify-between mb-4">
            <div>
              {title && (
                <h2 className="text-xl font-semibold text-foreground font-heading">{title}</h2>
              )}
              {subtitle && (
                <p className="text-sm text-foreground-muted mt-0.5">{subtitle}</p>
              )}
            </div>
            {action && <div>{action}</div>}
          </div>
        )}
        {children}
      </section>
    )
  }
)

Section.displayName = 'Section'

// Grid - 响应式网格布局
export interface GridProps extends HTMLAttributes<HTMLDivElement> {
  cols?: 1 | 2 | 3 | 4 | 5 | 6
  gap?: 'sm' | 'md' | 'lg'
  responsive?: boolean
}

const gridCols = {
  1: 'grid-cols-1',
  2: 'grid-cols-2',
  3: 'grid-cols-3',
  4: 'grid-cols-4',
  5: 'grid-cols-5',
  6: 'grid-cols-6',
}

const responsiveCols = {
  1: 'grid-cols-1',
  2: 'grid-cols-1 sm:grid-cols-2',
  3: 'grid-cols-2 sm:grid-cols-3',
  4: 'grid-cols-2 sm:grid-cols-3 lg:grid-cols-4',
  5: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5',
  6: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6',
}

const gridGaps = {
  sm: 'gap-3',
  md: 'gap-4',
  lg: 'gap-6',
}

export const Grid = forwardRef<HTMLDivElement, GridProps>(
  ({ cols = 4, gap = 'md', responsive = true, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          'grid',
          responsive ? responsiveCols[cols] : gridCols[cols],
          gridGaps[gap],
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Grid.displayName = 'Grid'

// Stack - 堆叠布局
export interface StackProps extends HTMLAttributes<HTMLDivElement> {
  direction?: 'row' | 'column'
  gap?: 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  align?: 'start' | 'center' | 'end' | 'stretch'
  justify?: 'start' | 'center' | 'end' | 'between' | 'around'
  wrap?: boolean
}

const stackGaps = {
  none: 'gap-0',
  xs: 'gap-1',
  sm: 'gap-2',
  md: 'gap-4',
  lg: 'gap-6',
  xl: 'gap-8',
}

const stackAlign = {
  start: 'items-start',
  center: 'items-center',
  end: 'items-end',
  stretch: 'items-stretch',
}

const stackJustify = {
  start: 'justify-start',
  center: 'justify-center',
  end: 'justify-end',
  between: 'justify-between',
  around: 'justify-around',
}

export const Stack = forwardRef<HTMLDivElement, StackProps>(
  (
    {
      direction = 'column',
      gap = 'md',
      align = 'stretch',
      justify = 'start',
      wrap = false,
      className,
      children,
      ...props
    },
    ref
  ) => {
    return (
      <div
        ref={ref}
        className={cn(
          'flex',
          direction === 'row' ? 'flex-row' : 'flex-col',
          stackGaps[gap],
          stackAlign[align],
          stackJustify[justify],
          wrap && 'flex-wrap',
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Stack.displayName = 'Stack'

// Divider - 分隔线
export interface DividerProps extends HTMLAttributes<HTMLHRElement> {
  orientation?: 'horizontal' | 'vertical'
  variant?: 'solid' | 'dashed' | 'dotted'
  label?: string
}

export function Divider({
  orientation = 'horizontal',
  variant = 'solid',
  label,
  className,
  ...props
}: DividerProps) {
  const variantStyles = {
    solid: 'border-solid',
    dashed: 'border-dashed',
    dotted: 'border-dotted',
  }

  if (label) {
    return (
      <div className={cn('flex items-center gap-4', className)}>
        <hr
          className={cn(
            'flex-1 border-t border-border',
            variantStyles[variant]
          )}
          {...props}
        />
        <span className="text-sm text-foreground-muted shrink-0">{label}</span>
        <hr
          className={cn(
            'flex-1 border-t border-border',
            variantStyles[variant]
          )}
        />
      </div>
    )
  }

  return (
    <hr
      className={cn(
        orientation === 'horizontal'
          ? 'border-t border-border w-full'
          : 'border-l border-border h-full',
        variantStyles[variant],
        className
      )}
      {...props}
    />
  )
}

// Spacer - 空间占位
export interface SpacerProps {
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'
  axis?: 'horizontal' | 'vertical'
}

const spacerSizes = {
  xs: { horizontal: 'w-2', vertical: 'h-2' },
  sm: { horizontal: 'w-4', vertical: 'h-4' },
  md: { horizontal: 'w-6', vertical: 'h-6' },
  lg: { horizontal: 'w-8', vertical: 'h-8' },
  xl: { horizontal: 'w-12', vertical: 'h-12' },
  '2xl': { horizontal: 'w-16', vertical: 'h-16' },
}

export function Spacer({ size = 'md', axis = 'vertical' }: SpacerProps) {
  return <div className={cn('shrink-0', spacerSizes[size][axis])} aria-hidden="true" />
}

// Center - 居中容器
export const Center = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn('flex items-center justify-center', className)}
        {...props}
      >
        {children}
      </div>
    )
  }
)

Center.displayName = 'Center'

// AspectRatio - 固定宽高比容器
export interface AspectRatioProps extends HTMLAttributes<HTMLDivElement> {
  ratio?: number | '16/9' | '4/3' | '1/1' | '2/3' | '3/4'
}

export const AspectRatio = forwardRef<HTMLDivElement, AspectRatioProps>(
  ({ ratio = '16/9', className, children, ...props }, ref) => {
    const ratioValue = typeof ratio === 'number' ? ratio : eval(ratio.replace('/', '/'))
    const paddingBottom = `${(1 / ratioValue) * 100}%`

    return (
      <div
        ref={ref}
        className={cn('relative w-full', className)}
        style={{ paddingBottom }}
        {...props}
      >
        <div className="absolute inset-0">{children}</div>
      </div>
    )
  }
)

AspectRatio.displayName = 'AspectRatio'

// ScrollArea - 可滚动区域
export interface ScrollAreaProps extends HTMLAttributes<HTMLDivElement> {
  orientation?: 'horizontal' | 'vertical' | 'both'
  hideScrollbar?: boolean
}

export const ScrollArea = forwardRef<HTMLDivElement, ScrollAreaProps>(
  ({ orientation = 'vertical', hideScrollbar = false, className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          orientation === 'horizontal' && 'overflow-x-auto overflow-y-hidden',
          orientation === 'vertical' && 'overflow-y-auto overflow-x-hidden',
          orientation === 'both' && 'overflow-auto',
          hideScrollbar ? 'scrollbar-hide' : 'scrollbar-thin',
          className
        )}
        {...props}
      >
        {children}
      </div>
    )
  }
)

ScrollArea.displayName = 'ScrollArea'

// Responsive visibility components
export function ShowOnMobile({ children }: { children: ReactNode }) {
  return <div className="block md:hidden">{children}</div>
}

export function ShowOnDesktop({ children }: { children: ReactNode }) {
  return <div className="hidden md:block">{children}</div>
}

export function HideOnMobile({ children }: { children: ReactNode }) {
  return <div className="hidden md:block">{children}</div>
}

export function HideOnDesktop({ children }: { children: ReactNode }) {
  return <div className="block md:hidden">{children}</div>
}

export default Container
