'use client'

import { type ReactNode, type HTMLAttributes, forwardRef, useState, useEffect } from 'react'
import { usePathname, useSearchParams } from 'next/navigation'
import Link from 'next/link'
import { cn } from '@/lib/utils'
import { type LucideIcon } from 'lucide-react'

// Navigation Item Type
export interface NavItem {
  href: string
  label: string
  icon?: LucideIcon | ReactNode
  badge?: string | number
  chip?: string // CSS class for category theming
  matchType?: 'exact' | 'prefix' | 'custom'
  matchFn?: (pathname: string, searchParams: URLSearchParams) => boolean
}

// Desktop Navigation
export interface DesktopNavProps extends HTMLAttributes<HTMLElement> {
  items: NavItem[]
  logo?: ReactNode
  actions?: ReactNode
}

export const DesktopNav = forwardRef<HTMLElement, DesktopNavProps>(
  ({ items, logo, actions, className, ...props }, ref) => {
    const pathname = usePathname()
    const searchParams = useSearchParams()

    const isActive = (item: NavItem) => {
      if (item.matchFn) {
        return item.matchFn(pathname, searchParams)
      }
      if (item.matchType === 'prefix') {
        return pathname.startsWith(item.href.split('?')[0])
      }
      return pathname === item.href.split('?')[0]
    }

    return (
      <header
        ref={ref}
        className={cn(
          'hidden md:block fixed top-0 left-0 right-0 z-50',
          className
        )}
        style={{ contain: 'layout paint' }}
        {...props}
      >
        <div className="mx-auto max-w-7xl px-4">
          <div className="mt-3 rounded-2xl relative overflow-hidden">
            {/* Background */}
            <div className="absolute inset-0 bg-white/70 dark:bg-gray-950/60 backdrop-blur-2xl rounded-2xl" />
            {/* Border */}
            <div className="absolute inset-0 rounded-2xl border border-white/30 dark:border-white/10 shadow-xl" />
            {/* Top glow line */}
            <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary-500/30 to-transparent" />

            {/* Content */}
            <nav className="relative flex items-center justify-between h-16 px-5">
              {/* Logo */}
              {logo && <div className="shrink-0">{logo}</div>}

              {/* Nav Items */}
              <div className="flex items-center justify-center gap-2 flex-wrap">
                {items.map((item) => {
                  const active = isActive(item)
                  const Icon = item.icon

                  return (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={cn(
                        'inline-flex items-center gap-2 rounded-full px-4 py-2',
                        'text-sm font-medium transition-all duration-300 ease-out',
                        'glass-chip chip-glow chip-theme',
                        item.chip,
                        'hover:scale-[1.02] active:scale-[0.98]',
                        active
                          ? 'ring-2 ring-primary-500/40 shadow-lg shadow-primary-500/20'
                          : 'hover:shadow-md'
                      )}
                    >
                      {Icon && (
                        typeof Icon === 'function' ? (
                          <Icon className={cn('h-4 w-4 transition-transform duration-300', active && 'scale-110')} />
                        ) : (
                          Icon
                        )
                      )}
                      <span>{item.label}</span>
                      {item.badge && (
                        <span className="ml-1 px-1.5 py-0.5 text-xs font-medium bg-primary-500 text-white rounded-full">
                          {item.badge}
                        </span>
                      )}
                    </Link>
                  )
                })}
              </div>

              {/* Actions */}
              {actions && <div className="flex items-center gap-3">{actions}</div>}
            </nav>
          </div>
        </div>
      </header>
    )
  }
)

DesktopNav.displayName = 'DesktopNav'

// Mobile Bottom Navigation
export interface MobileNavProps extends HTMLAttributes<HTMLElement> {
  items: NavItem[]
}

export const MobileNav = forwardRef<HTMLElement, MobileNavProps>(
  ({ items, className, ...props }, ref) => {
    const pathname = usePathname()
    const searchParams = useSearchParams()

    const isActive = (item: NavItem) => {
      if (item.matchFn) {
        return item.matchFn(pathname, searchParams)
      }
      if (item.matchType === 'prefix') {
        return pathname.startsWith(item.href.split('?')[0])
      }
      return pathname === item.href.split('?')[0]
    }

    return (
      <nav
        ref={ref}
        className={cn(
          'md:hidden fixed z-50',
          'left-4 right-4 bottom-4',
          'rounded-full overflow-hidden',
          'bg-white/80 dark:bg-gray-900/80 backdrop-blur-2xl',
          'border border-white/30 dark:border-white/10',
          'shadow-xl shadow-black/10 dark:shadow-black/30',
          'safe-bottom',
          className
        )}
        {...props}
      >
        <div className="flex items-center justify-around h-14 px-2 overflow-x-auto scrollbar-hide">
          {items.map((item) => {
            const active = isActive(item)
            const Icon = item.icon
            const colorClass = item.chip?.replace('chip-', 'mb-') || 'mb-home'

            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  'mbnav-item flex flex-col items-center justify-center',
                  'min-w-[3.5rem] py-1.5 px-2',
                  active && 'mbnav-item-active',
                  active ? colorClass : 'text-foreground-muted'
                )}
              >
                {Icon && (
                  typeof Icon === 'function' ? (
                    <Icon className={cn('h-5 w-5 transition-transform', active && 'scale-110')} />
                  ) : (
                    Icon
                  )
                )}
                <span className="text-[10px] font-medium mt-0.5 truncate max-w-full">
                  {item.label}
                </span>
              </Link>
            )
          })}
        </div>
      </nav>
    )
  }
)

MobileNav.displayName = 'MobileNav'

// Mobile Header
export interface MobileHeaderProps extends Omit<HTMLAttributes<HTMLElement>, 'title'> {
  title?: ReactNode
  leftAction?: ReactNode
  rightAction?: ReactNode
  showBackButton?: boolean
  transparent?: boolean
}

export const MobileHeader = forwardRef<HTMLElement, MobileHeaderProps>(
  (
    {
      title,
      leftAction,
      rightAction,
      showBackButton = false,
      transparent = false,
      className,
      ...props
    },
    ref
  ) => {
    const [scrolled, setScrolled] = useState(false)

    useEffect(() => {
      const handleScroll = () => {
        setScrolled(window.scrollY > 10)
      }
      window.addEventListener('scroll', handleScroll)
      return () => window.removeEventListener('scroll', handleScroll)
    }, [])

    return (
      <header
        ref={ref}
        className={cn(
          'md:hidden fixed top-0 left-0 right-0 z-50',
          'h-14 flex items-center justify-between px-4',
          'transition-all duration-300',
          'safe-top',
          transparent && !scrolled
            ? 'bg-transparent'
            : 'bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl border-b border-border/50',
          className
        )}
        {...props}
      >
        {/* Left */}
        <div className="flex items-center gap-2 min-w-[4rem]">
          {leftAction}
        </div>

        {/* Center */}
        <div className="flex-1 flex items-center justify-center">
          {typeof title === 'string' ? (
            <h1 className="text-lg font-semibold truncate">{title}</h1>
          ) : (
            title
          )}
        </div>

        {/* Right */}
        <div className="flex items-center gap-2 min-w-[4rem] justify-end">
          {rightAction}
        </div>
      </header>
    )
  }
)

MobileHeader.displayName = 'MobileHeader'

// Sidebar Navigation
export interface SidebarProps extends HTMLAttributes<HTMLElement> {
  items: NavItem[]
  header?: ReactNode
  footer?: ReactNode
  collapsed?: boolean
  onCollapsedChange?: (collapsed: boolean) => void
}

export const Sidebar = forwardRef<HTMLElement, SidebarProps>(
  (
    {
      items,
      header,
      footer,
      collapsed = false,
      onCollapsedChange,
      className,
      ...props
    },
    ref
  ) => {
    const pathname = usePathname()

    return (
      <aside
        ref={ref}
        className={cn(
          'hidden lg:flex flex-col h-screen',
          'bg-background border-r border-border',
          'transition-all duration-300',
          collapsed ? 'w-16' : 'w-64',
          className
        )}
        {...props}
      >
        {/* Header */}
        {header && (
          <div className="h-16 flex items-center px-4 border-b border-border shrink-0">
            {header}
          </div>
        )}

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-4 px-2">
          <ul className="space-y-1">
            {items.map((item) => {
              const active = pathname === item.href.split('?')[0]
              const Icon = item.icon

              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    className={cn(
                      'flex items-center gap-3 px-3 py-2.5 rounded-lg',
                      'text-sm font-medium transition-colors',
                      active
                        ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400'
                        : 'text-foreground-muted hover:bg-background-muted hover:text-foreground'
                    )}
                    title={collapsed ? item.label : undefined}
                  >
                    {Icon && (
                      typeof Icon === 'function' ? (
                        <Icon className="h-5 w-5 shrink-0" />
                      ) : (
                        <span className="shrink-0">{Icon}</span>
                      )
                    )}
                    {!collapsed && <span className="truncate">{item.label}</span>}
                    {!collapsed && item.badge && (
                      <span className="ml-auto px-2 py-0.5 text-xs font-medium bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 rounded-full">
                        {item.badge}
                      </span>
                    )}
                  </Link>
                </li>
              )
            })}
          </ul>
        </nav>

        {/* Footer */}
        {footer && (
          <div className="p-4 border-t border-border shrink-0">{footer}</div>
        )}
      </aside>
    )
  }
)

Sidebar.displayName = 'Sidebar'

export default DesktopNav
