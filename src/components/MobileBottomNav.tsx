/* eslint-disable @typescript-eslint/no-explicit-any */

'use client';

import { Cat, Clover, Film, Home, Search, Star, Tv } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { memo, useCallback, useEffect, useMemo, useRef, useTransition } from 'react';

// 简单的 className 合并函数
function cn(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}

interface NavItem {
  icon: typeof Home;
  label: string;
  href: string;
  activeGradient: string;
  glowColor: string;
}

// 基础导航项 - 静态配置
const BASE_NAV_ITEMS: NavItem[] = [
  {
    icon: Home,
    label: '首页',
    href: '/',
    activeGradient: 'from-violet-500 to-purple-600',
    glowColor: 'shadow-violet-500/40',
  },
  {
    icon: Search,
    label: '搜索',
    href: '/search',
    activeGradient: 'from-indigo-500 to-blue-500',
    glowColor: 'shadow-indigo-500/40',
  },
  {
    icon: Film,
    label: '电影',
    href: '/douban?type=movie',
    activeGradient: 'from-fuchsia-500 to-pink-500',
    glowColor: 'shadow-fuchsia-500/40',
  },
  {
    icon: Tv,
    label: '剧集',
    href: '/douban?type=tv',
    activeGradient: 'from-purple-500 to-violet-500',
    glowColor: 'shadow-purple-500/40',
  },
  {
    icon: Cat,
    label: '动漫',
    href: '/douban?type=anime',
    activeGradient: 'from-teal-400 to-emerald-500',
    glowColor: 'shadow-teal-500/40',
  },
  {
    icon: Clover,
    label: '综艺',
    href: '/douban?type=show',
    activeGradient: 'from-amber-400 to-orange-500',
    glowColor: 'shadow-amber-500/40',
  },
];

interface MobileBottomNavProps {
  activePath?: string;
}

/**
 * 移动端底部导航栏 - Aurora Design System
 * 使用 useTransition 优化导航性能
 */
const MobileBottomNav = memo(function MobileBottomNav({ activePath = '/' }: MobileBottomNavProps) {
  const router = useRouter();
  const [isPending, startTransition] = useTransition();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLElement | null)[]>([]);

  // 使用 useMemo 计算导航项，避免每次渲染重建
  const navItems = useMemo(() => {
    const runtimeConfig = typeof window !== 'undefined' ? (window as any).RUNTIME_CONFIG : null;
    if (runtimeConfig?.CUSTOM_CATEGORIES?.length > 0) {
      return [
        ...BASE_NAV_ITEMS,
        {
          icon: Star,
          label: '自定义',
          href: '/douban?type=custom',
          activeGradient: 'from-yellow-400 to-amber-500',
          glowColor: 'shadow-yellow-500/40',
        },
      ];
    }
    return BASE_NAV_ITEMS;
  }, []);

  // 判断是否激活
  const isActive = useCallback(
    (href: string) => {
      const typeMatch = href.match(/type=([^&]+)/)?.[1];
      const decodedActive = decodeURIComponent(activePath);
      const decodedItemHref = decodeURIComponent(href);

      if (decodedActive === decodedItemHref) return true;
      if (href === '/' && decodedActive === '/') return true;
      if (href === '/search' && decodedActive.startsWith('/search')) return true;
      if (
        typeMatch &&
        decodedActive.startsWith('/douban') &&
        decodedActive.includes(`type=${typeMatch}`)
      ) {
        return true;
      }
      return false;
    },
    [activePath],
  );

  // 使用硬跳转在 Tauri 环境下获得更好的性能
  const handleNavigation = useCallback(
    (e: React.MouseEvent, href: string) => {
      e.preventDefault();
      
      // 检测是否在 Tauri 桌面环境中
      const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
      
      if (isTauri) {
        // Tauri 环境：使用硬跳转绕过 React 客户端导航
        window.location.assign(href);
      } else {
        // 浏览器环境：使用 startTransition 包裹
        startTransition(() => {
          router.push(href);
        });
      }
    },
    [router, startTransition],
  );

  // 滚动到激活项
  useEffect(() => {
    const activeIndex = navItems.findIndex((item) => isActive(item.href));
    if (activeIndex === -1) return;

    const timer = setTimeout(() => {
      const activeItem = itemRefs.current[activeIndex];
      if (activeItem) {
        activeItem.scrollIntoView({
          behavior: 'smooth',
          block: 'nearest',
          inline: 'center',
        });
      }
    }, 100);

    return () => clearTimeout(timer);
  }, [activePath, navItems, isActive]);

  return (
    <nav
      className={cn(
        'md:hidden fixed z-600',
        'left-1/2 -translate-x-1/2',
        'w-auto max-w-[94vw]',
        isPending && 'opacity-70',
      )}
      style={{
        bottom: 'calc(1rem + env(safe-area-inset-bottom))',
      }}
    >
      {/* 背景层 */}
      <div className='absolute inset-0 rounded-2xl bg-white/70 dark:bg-gray-950/70 backdrop-blur-2xl border border-white/30 dark:border-white/10 shadow-xl shadow-black/5 dark:shadow-black/30' />

      {/* 顶部微光线 */}
      <div className='absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/30 to-transparent' />

      {/* 横向滚动容器 */}
      <div
        ref={scrollContainerRef}
        className={cn(
          'relative flex items-center gap-1.5 px-3 py-2.5',
          'overflow-x-auto scroll-smooth scrollbar-hide',
        )}
        style={{
          scrollbarWidth: 'none',
          msOverflowStyle: 'none',
          WebkitOverflowScrolling: 'touch',
        }}
      >
        {navItems.map((item, index) => {
          const active = isActive(item.href);
          const Icon = item.icon;

          return (
            <a
              key={item.href}
              href={item.href}
              ref={(el) => {
                itemRefs.current[index] = el;
              }}
              onClick={(e) => handleNavigation(e, item.href)}
              className={cn(
                'shrink-0 inline-flex items-center gap-1.5',
                'rounded-xl px-4 py-2.5',
                'text-sm font-medium',
                'transition-all duration-300 ease-out',
                'focus:outline-none',
                'active:scale-95',
                active && `bg-gradient-to-r ${item.activeGradient}`,
                active && 'text-white',
                active && `shadow-lg ${item.glowColor}`,
                !active && 'text-gray-600 dark:text-gray-400',
                !active && 'hover:bg-black/5 dark:hover:bg-white/5',
                !active && 'hover:text-gray-900 dark:hover:text-white',
              )}
            >
              <Icon
                className={cn(
                  'w-4 h-4 shrink-0',
                  'transition-all duration-300',
                  active && 'drop-shadow-sm',
                )}
              />
              <span className='whitespace-nowrap'>{item.label}</span>
            </a>
          );
        })}
      </div>
    </nav>
  );
});

export default MobileBottomNav;
