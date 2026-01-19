'use client'

import { type ReactNode, type HTMLAttributes, forwardRef } from 'react'
import { cn } from '@/lib/utils'
import { ChevronRight } from 'lucide-react'

// Settings Section
export interface SettingsSectionProps extends HTMLAttributes<HTMLDivElement> {
  title?: string
  description?: string
}

export const SettingsSection = forwardRef<HTMLDivElement, SettingsSectionProps>(
  ({ title, description, className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn('mb-8', className)} {...props}>
        {(title || description) && (
          <div className="mb-4">
            {title && (
              <h3 className="text-lg font-semibold text-foreground font-heading">{title}</h3>
            )}
            {description && (
              <p className="text-sm text-foreground-muted mt-0.5">{description}</p>
            )}
          </div>
        )}
        <div className="glass-card p-0 overflow-hidden">{children}</div>
      </div>
    )
  }
)

SettingsSection.displayName = 'SettingsSection'

// Settings Item
export interface SettingsItemProps extends HTMLAttributes<HTMLDivElement> {
  icon?: ReactNode
  title: string
  description?: string
  value?: ReactNode
  action?: ReactNode
  href?: string
  showChevron?: boolean
  danger?: boolean
  disabled?: boolean
}

export const SettingsItem = forwardRef<HTMLDivElement, SettingsItemProps>(
  (
    {
      icon,
      title,
      description,
      value,
      action,
      href,
      showChevron = false,
      danger = false,
      disabled = false,
      className,
      onClick,
      ...props
    },
    ref
  ) => {
    const content = (
      <>
        {/* Icon */}
        {icon && (
          <div
            className={cn(
              'w-10 h-10 shrink-0 flex items-center justify-center rounded-xl',
              'bg-background-muted',
              danger ? 'text-error' : 'text-foreground-muted'
            )}
          >
            {icon}
          </div>
        )}

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div
            className={cn(
              'text-sm font-medium',
              danger ? 'text-error' : 'text-foreground'
            )}
          >
            {title}
          </div>
          {description && (
            <div className="text-xs text-foreground-muted mt-0.5 line-clamp-2">
              {description}
            </div>
          )}
        </div>

        {/* Value */}
        {value && (
          <div className="shrink-0 text-sm text-foreground-muted">{value}</div>
        )}

        {/* Action */}
        {action && <div className="shrink-0">{action}</div>}

        {/* Chevron */}
        {showChevron && !action && (
          <ChevronRight size={18} className="shrink-0 text-foreground-subtle" />
        )}
      </>
    )

    const baseClass = cn(
      'flex items-center gap-4 px-4 py-3',
      'border-b border-border last:border-b-0',
      'transition-colors',
      disabled && 'opacity-50 cursor-not-allowed',
      (onClick || href) && !disabled && 'cursor-pointer hover:bg-background-muted',
      className
    )

    if (href && !disabled) {
      return (
        <a ref={ref as any} href={href} className={baseClass} {...(props as any)}>
          {content}
        </a>
      )
    }

    return (
      <div
        ref={ref}
        className={baseClass}
        onClick={disabled ? undefined : onClick}
        role={onClick ? 'button' : undefined}
        tabIndex={onClick && !disabled ? 0 : undefined}
        {...props}
      >
        {content}
      </div>
    )
  }
)

SettingsItem.displayName = 'SettingsItem'

// Settings Toggle Item
export interface SettingsToggleItemProps extends Omit<SettingsItemProps, 'action' | 'value'> {
  checked: boolean
  onCheckedChange: (checked: boolean) => void
}

export function SettingsToggleItem({
  checked,
  onCheckedChange,
  disabled,
  ...props
}: SettingsToggleItemProps) {
  return (
    <SettingsItem
      {...props}
      disabled={disabled}
      onClick={() => !disabled && onCheckedChange(!checked)}
      action={
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            className="sr-only peer"
            checked={checked}
            onChange={(e) => onCheckedChange(e.target.checked)}
            disabled={disabled}
          />
          <div
            className={cn(
              'relative w-11 h-6 rounded-full transition-colors duration-200',
              'bg-border peer-checked:bg-primary-500',
              'after:content-[""] after:absolute after:top-0.5 after:left-0.5',
              'after:bg-white after:rounded-full after:h-5 after:w-5',
              'after:transition-transform after:duration-200',
              'peer-checked:after:translate-x-5',
              'after:shadow-sm',
              disabled && 'cursor-not-allowed'
            )}
          />
        </label>
      }
    />
  )
}

// Settings Select Item
export interface SettingsSelectItemProps extends Omit<SettingsItemProps, 'action' | 'value'> {
  options: { value: string; label: string }[]
  value: string
  onValueChange: (value: string) => void
}

export function SettingsSelectItem({
  options,
  value,
  onValueChange,
  disabled,
  ...props
}: SettingsSelectItemProps) {
  const selectedOption = options.find((opt) => opt.value === value)

  return (
    <SettingsItem
      {...props}
      disabled={disabled}
      value={selectedOption?.label}
      action={
        <select
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          disabled={disabled}
          className={cn(
            'input input-sm w-auto min-w-[8rem] text-right',
            'bg-transparent border-0 focus:ring-0',
            disabled && 'cursor-not-allowed'
          )}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      }
    />
  )
}

// Settings Input Item
export interface SettingsInputItemProps extends Omit<SettingsItemProps, 'action' | 'value'> {
  value: string
  onValueChange: (value: string) => void
  placeholder?: string
  type?: 'text' | 'number' | 'password' | 'email'
}

export function SettingsInputItem({
  value,
  onValueChange,
  placeholder,
  type = 'text',
  disabled,
  ...props
}: SettingsInputItemProps) {
  return (
    <SettingsItem
      {...props}
      disabled={disabled}
      action={
        <input
          type={type}
          value={value}
          onChange={(e) => onValueChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          className={cn(
            'input input-sm w-auto min-w-[10rem] text-right',
            disabled && 'cursor-not-allowed'
          )}
        />
      }
    />
  )
}

// Settings Slider Item
export interface SettingsSliderItemProps extends Omit<SettingsItemProps, 'action' | 'value'> {
  value: number
  min?: number
  max?: number
  step?: number
  onValueChange: (value: number) => void
  formatValue?: (value: number) => string
}

export function SettingsSliderItem({
  value,
  min = 0,
  max = 100,
  step = 1,
  onValueChange,
  formatValue,
  disabled,
  ...props
}: SettingsSliderItemProps) {
  return (
    <SettingsItem
      {...props}
      disabled={disabled}
      value={formatValue ? formatValue(value) : String(value)}
      action={
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(e) => onValueChange(Number(e.target.value))}
          disabled={disabled}
          className={cn('slider w-24', disabled && 'cursor-not-allowed')}
        />
      }
    />
  )
}

// Settings Group (for related items)
export interface SettingsGroupProps extends HTMLAttributes<HTMLDivElement> {
  label?: string
}

export const SettingsGroup = forwardRef<HTMLDivElement, SettingsGroupProps>(
  ({ label, className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn('', className)} {...props}>
        {label && (
          <div className="px-4 py-2 text-xs font-medium text-foreground-muted uppercase tracking-wider bg-background-muted/50">
            {label}
          </div>
        )}
        {children}
      </div>
    )
  }
)

SettingsGroup.displayName = 'SettingsGroup'

// Settings Page Layout
export interface SettingsPageProps extends HTMLAttributes<HTMLDivElement> {
  title?: string
  subtitle?: string
}

export const SettingsPage = forwardRef<HTMLDivElement, SettingsPageProps>(
  ({ title, subtitle, className, children, ...props }, ref) => {
    return (
      <div ref={ref} className={cn('max-w-2xl mx-auto py-6 container-px', className)} {...props}>
        {(title || subtitle) && (
          <div className="mb-8">
            {title && (
              <h1 className="text-2xl font-bold text-foreground font-heading">{title}</h1>
            )}
            {subtitle && (
              <p className="text-foreground-muted mt-1">{subtitle}</p>
            )}
          </div>
        )}
        {children}
      </div>
    )
  }
)

SettingsPage.displayName = 'SettingsPage'

export default SettingsSection
