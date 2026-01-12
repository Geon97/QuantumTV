/* eslint-disable @typescript-eslint/no-explicit-any */

'use client';

import { Cat, Clover, Film, Home, Search, Star, Tv } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useCallback, useEffect, useRef, useState } from 'react';

// 简单的 className 合并函数
function cn(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(' ');
}

interface NavItem {
  icon: typeof Home;
  label: string;
  href: string;
  // 激活状态的渐变色配置
  activeGradient: string;
  // 激活状态的发光色
  glowColor: string;
}

interface MobileBottomNavProps {
  /**
   * 主动指定当前激活的路径。当未提供时，自动使用 usePathname() 获取的路径。
   */
  activePath?: string;
}

/**
 * 移动端底部导航栏 - Aurora Design System
 * 悬浮胶囊风格 + 极光渐变
 */
const MobileBottomNav = ({ activePath }: MobileBottomNavProps) => {
  const pathname = usePathname();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLElement | null)[]>([]);

  // 当前激活路径：优先使用传入的 activePath，否则回退到浏览器地址
  const currentActive = activePath ?? pathname;

  // 导航项配置 - Aurora 风格渐变色
  const [navItems, setNavItems] = useState<NavItem[]>([
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
  ]);

  // 动态添加自定义分类
  useEffect(() => {
    const runtimeConfig = (window as any).RUNTIME_CONFIG;
    if (runtimeConfig?.CUSTOM_CATEGORIES?.length > 0) {
      setNavItems((prevItems) => {
        // 防止重复添加
        if (prevItems.some((item) => item.label === '自定义')) return prevItems;
        return [
          ...prevItems,
          {
            icon: Star,
            label: '自定义',
            href: '/douban?type=custom',
            activeGradient: 'from-yellow-400 to-amber-500',
            glowColor: 'shadow-yellow-500/40',
          },
        ];
      });
    }
  }, []);

  // 判断是否激活
  const isActive = useCallback(
    (href: string) => {
      const typeMatch = href.match(/type=([^&]+)/)?.[1];
      const decodedActive = decodeURIComponent(currentActive);
      const decodedItemHref = decodeURIComponent(href);

      // 精确匹配
      if (decodedActive === decodedItemHref) return true;

      // 首页特殊处理
      if (href === '/' && decodedActive === '/') return true;

      // 搜索页特殊处理
      if (href === '/search' && decodedActive.startsWith('/search'))
        return true;

      // 豆瓣分类匹配
      if (
        typeMatch &&
        decodedActive.startsWith('/douban') &&
        decodedActive.includes(`type=${typeMatch}`)
      ) {
        return true;
      }

      return false;
    },
    [currentActive],
  );

  // 滚动到激活项
  const scrollToActiveItem = useCallback(() => {
    const activeIndex = navItems.findIndex((item) => isActive(item.href));
    if (activeIndex === -1) return;

    const activeItem = itemRefs.current[activeIndex];
    if (activeItem) {
      activeItem.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
        inline: 'center',
      });
    }
  }, [navItems, isActive]);

  // 路径变化时滚动到激活项
  useEffect(() => {
    const timer = setTimeout(scrollToActiveItem, 100);
    return () => clearTimeout(timer);
  }, [currentActive, scrollToActiveItem]);

  return (
    <nav
      className={cn(
        'md:hidden fixed z-600',
        // 悬浮居中定位
        'left-1/2 -translate-x-1/2',
        // 尺寸限制
        'w-auto max-w-[94vw]',
      )}
      style={{
        // 距离底部安全区
        bottom: 'calc(1rem + env(safe-area-inset-bottom))',
      }}
    >
      {/* 背景层 - Aurora 磨砂玻璃 */}
      <div className='absolute inset-0 rounded-2xl bg-white/70 dark:bg-gray-950/70 backdrop-blur-2xl border border-white/30 dark:border-white/10 shadow-xl shadow-black/5 dark:shadow-black/30' />

      {/* 顶部微光线 */}
      <div className='absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/30 to-transparent' />

      {/* 横向滚动容器 */}
      <div
        ref={scrollContainerRef}
        className={cn(
          'relative flex items-center gap-1.5 px-3 py-2.5',
          'overflow-x-auto',
          'scroll-smooth scrollbar-hide',
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
            <Link
              key={item.href}
              href={item.href}
              prefetch={false}
              ref={(el) => {
                itemRefs.current[index] = el;
              }}
              className={cn(
                // 基础样式
                'shrink-0 inline-flex items-center gap-1.5',
                'rounded-xl px-4 py-2.5',
                'text-sm font-medium',
                'transition-all duration-300 ease-out',
                'focus:outline-none',
                // 点击反馈
                'active:scale-95',
                // 激活状态
                active && `bg-gradient-to-r ${item.activeGradient}`,
                active && 'text-white',
                active && `shadow-lg ${item.glowColor}`,
                // 非激活状态
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
            </Link>
          );
        })}
      </div>
    </nav>
  );
};

export default MobileBottomNav;
