'use client'

import { forwardRef, type InputHTMLAttributes } from 'react'
import { cn } from '@/lib/utils'

// Toggle Switch
export interface ToggleProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'size'> {
  label?: string
  description?: string
  size?: 'sm' | 'md' | 'lg'
}

const toggleSizes = {
  sm: 'w-9 h-5 after:w-4 after:h-4',
  md: 'w-11 h-6 after:w-5 after:h-5',
  lg: 'w-14 h-7 after:w-6 after:h-6',
}

const toggleTranslate = {
  sm: 'peer-checked:after:translate-x-4',
  md: 'peer-checked:after:translate-x-5',
  lg: 'peer-checked:after:translate-x-7',
}

export const Toggle = forwardRef<HTMLInputElement, ToggleProps>(
  ({ className, label, description, size = 'md', id, disabled, ...props }, ref) => {
    const toggleId = id || `toggle-${Math.random().toString(36).substr(2, 9)}`

    return (
      <div className={cn('flex items-start gap-3', disabled && 'opacity-50')}>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            ref={ref}
            type="checkbox"
            id={toggleId}
            className="sr-only peer"
            disabled={disabled}
            {...props}
          />
          <div
            className={cn(
              'relative rounded-full transition-colors duration-200',
              'bg-border peer-checked:bg-primary-500',
              'after:content-[""] after:absolute after:top-0.5 after:left-0.5',
              'after:bg-white after:rounded-full after:transition-transform after:duration-200',
              'after:shadow-sm',
              toggleSizes[size],
              toggleTranslate[size],
              disabled && 'cursor-not-allowed',
              className
            )}
          />
        </label>
        {(label || description) && (
          <div className="flex flex-col">
            {label && (
              <label htmlFor={toggleId} className="text-sm font-medium text-foreground cursor-pointer">
                {label}
              </label>
            )}
            {description && (
              <span className="text-xs text-foreground-muted mt-0.5">{description}</span>
            )}
          </div>
        )}
      </div>
    )
  }
)

Toggle.displayName = 'Toggle'

// Slider
export interface SliderProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label?: string
  showValue?: boolean
  formatValue?: (value: number) => string
}

export const Slider = forwardRef<HTMLInputElement, SliderProps>(
  ({ className, label, showValue = false, formatValue, id, value, ...props }, ref) => {
    const sliderId = id || `slider-${Math.random().toString(36).substr(2, 9)}`
    const displayValue = formatValue
      ? formatValue(Number(value))
      : String(value)

    return (
      <div className="w-full">
        {(label || showValue) && (
          <div className="flex items-center justify-between mb-2">
            {label && (
              <label htmlFor={sliderId} className="text-sm font-medium text-foreground">
                {label}
              </label>
            )}
            {showValue && (
              <span className="text-sm text-foreground-muted font-mono">{displayValue}</span>
            )}
          </div>
        )}
        <input
          ref={ref}
          type="range"
          id={sliderId}
          value={value}
          className={cn('slider', className)}
          {...props}
        />
      </div>
    )
  }
)

Slider.displayName = 'Slider'

// Progress Bar
export interface ProgressBarProps {
  value: number
  max?: number
  showLabel?: boolean
  label?: string
  size?: 'sm' | 'md' | 'lg'
  variant?: 'default' | 'success' | 'warning' | 'error'
  className?: string
}

const progressSizes = {
  sm: 'h-1',
  md: 'h-2',
  lg: 'h-3',
}

const progressVariants = {
  default: 'bg-gradient-to-r from-primary-500 to-aurora-fuchsia',
  success: 'bg-success',
  warning: 'bg-warning',
  error: 'bg-error',
}

export function ProgressBar({
  value,
  max = 100,
  showLabel = false,
  label,
  size = 'md',
  variant = 'default',
  className,
}: ProgressBarProps) {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100)

  return (
    <div className={cn('w-full', className)}>
      {(label || showLabel) && (
        <div className="flex items-center justify-between mb-1.5">
          {label && <span className="text-sm text-foreground">{label}</span>}
          {showLabel && (
            <span className="text-sm text-foreground-muted">{Math.round(percentage)}%</span>
          )}
        </div>
      )}
      <div className={cn('w-full bg-border rounded-full overflow-hidden', progressSizes[size])}>
        <div
          className={cn('h-full rounded-full transition-all duration-300', progressVariants[variant])}
          style={{ width: `${percentage}%` }}
          role="progressbar"
          aria-valuenow={value}
          aria-valuemin={0}
          aria-valuemax={max}
        />
      </div>
    </div>
  )
}

export default Toggle
