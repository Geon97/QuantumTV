'use client'

import {
  createContext,
  type HTMLAttributes,
  type ReactNode,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'

import { cn } from '@/lib/utils'

// Tabs Context
interface TabsContextValue {
  activeTab: string
  setActiveTab: (id: string) => void
}

const TabsContext = createContext<TabsContextValue | null>(null)

// Tabs Root
interface TabsProps extends Omit<React.HTMLAttributes<HTMLDivElement>, 'onChange'> {
  defaultTab?: string
  value?: string
  onChange?: (value: string) => void
  children: ReactNode
}

export function Tabs({
  defaultTab,
  value,
  onChange,
  className,
  children,
  ...props
}: TabsProps) {
  const [internalValue, setInternalValue] = useState(defaultTab || '')
  const activeTab = value ?? internalValue

  const setActiveTab = (id: string) => {
    if (value === undefined) {
      setInternalValue(id)
    }
    onChange?.(id)
  }

  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab }}>
      <div className={cn('w-full', className)} {...props}>
        {children}
      </div>
    </TabsContext.Provider>
  )
}

// Tab List
export interface TabListProps extends HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'pills' | 'underline'
}

export function TabList({ variant = 'default', className, children, ...props }: TabListProps) {
  const variantClasses = {
    default: 'tabs',
    pills: 'flex gap-2',
    underline: 'flex gap-4 border-b border-border',
  }

  return (
    <div role="tablist" className={cn(variantClasses[variant], className)} {...props}>
      {children}
    </div>
  )
}

// Tab Trigger
export interface TabTriggerProps extends HTMLAttributes<HTMLButtonElement> {
  value: string
  disabled?: boolean
  icon?: ReactNode
}

export function TabTrigger({
  value,
  disabled = false,
  icon,
  className,
  children,
  ...props
}: TabTriggerProps) {
  const context = useContext(TabsContext)
  if (!context) throw new Error('TabTrigger must be used within Tabs')

  const isActive = context.activeTab === value

  return (
    <button
      role="tab"
      aria-selected={isActive}
      aria-controls={`tabpanel-${value}`}
      id={`tab-${value}`}
      tabIndex={isActive ? 0 : -1}
      disabled={disabled}
      className={cn(
        'tab',
        isActive && 'active',
        disabled && 'opacity-50 cursor-not-allowed',
        className
      )}
      onClick={() => !disabled && context.setActiveTab(value)}
      {...props}
    >
      {icon && <span className="shrink-0 mr-2">{icon}</span>}
      {children}
    </button>
  )
}

// Tab Panel
export interface TabPanelProps extends HTMLAttributes<HTMLDivElement> {
  value: string
}

export function TabPanel({ value, className, children, ...props }: TabPanelProps) {
  const context = useContext(TabsContext)
  if (!context) throw new Error('TabPanel must be used within Tabs')

  if (context.activeTab !== value) return null

  return (
    <div
      role="tabpanel"
      id={`tabpanel-${value}`}
      aria-labelledby={`tab-${value}`}
      tabIndex={0}
      className={cn('mt-4 animate-fade-in', className)}
      {...props}
    >
      {children}
    </div>
  )
}

// Capsule Switch (Alternative Tab style)
export interface CapsuleSwitchProps<T extends string> {
  options: { value: T; label: string; icon?: ReactNode }[]
  value: T
  onChange: (value: T) => void
  className?: string
}

export function CapsuleSwitch<T extends string>({
  options,
  value,
  onChange,
  className,
}: CapsuleSwitchProps<T>) {
  const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 })
  const containerRef = useRef<HTMLDivElement>(null)
  const buttonRefs = useRef<(HTMLButtonElement | null)[]>([])

  useEffect(() => {
    const activeIndex = options.findIndex((opt) => opt.value === value)
    const activeButton = buttonRefs.current[activeIndex]
    if (activeButton && containerRef.current) {
      const containerRect = containerRef.current.getBoundingClientRect()
      const buttonRect = activeButton.getBoundingClientRect()
      setIndicatorStyle({
        left: buttonRect.left - containerRect.left,
        width: buttonRect.width,
      })
    }
  }, [value, options])

  return (
    <div ref={containerRef} className={cn('capsule-switch', className)}>
      {/* Indicator */}
      <div
        className="capsule-switch-indicator"
        style={{
          left: indicatorStyle.left,
          width: indicatorStyle.width,
        }}
      />
      {/* Options */}
      {options.map((option, index) => (
        <button
          key={option.value}
          ref={(el) => { buttonRefs.current[index] = el }}
          className={cn('capsule-switch-item', value === option.value && 'active')}
          onClick={() => onChange(option.value)}
        >
          {option.icon && <span className="mr-1.5">{option.icon}</span>}
          {option.label}
        </button>
      ))}
    </div>
  )
}

// Breadcrumb
export interface BreadcrumbItem {
  label: string
  href?: string
  onClick?: () => void
}

export interface BreadcrumbProps {
  items: BreadcrumbItem[]
  separator?: ReactNode
  className?: string
}

export function Breadcrumb({ items, separator = '/', className }: BreadcrumbProps) {
  return (
    <nav aria-label="Breadcrumb" className={cn('breadcrumb', className)}>
      {items.map((item, index) => (
        <span key={index} className="flex items-center gap-2">
          {index > 0 && (
            <span className="breadcrumb-separator" aria-hidden="true">
              {separator}
            </span>
          )}
          {item.href || item.onClick ? (
            <a
              href={item.href}
              onClick={item.onClick}
              className={cn(
                'breadcrumb-item',
                index === items.length - 1 && 'active'
              )}
              aria-current={index === items.length - 1 ? 'page' : undefined}
            >
              {item.label}
            </a>
          ) : (
            <span
              className={cn(
                'breadcrumb-item',
                index === items.length - 1 && 'active'
              )}
              aria-current={index === items.length - 1 ? 'page' : undefined}
            >
              {item.label}
            </span>
          )}
        </span>
      ))}
    </nav>
  )
}

export default Tabs
