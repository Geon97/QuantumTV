/**
 * TopNavbar - PC 端顶部导航栏
 *
 * Aurora Design System - 磨砂玻璃 + 极光效果
 *
 * 性能优化策略：
 * 1. FastLink + useTransitionNav: 路由切换标记为"过渡更新"，不阻塞主线程
 * 2. React.memo: 防止父组件重绘导致不必要的渲染
 * 3. contain: layout paint: CSS 渲染隔离，防止 Aurora 背景触发导航栏重排
 * 4. 高 z-index: 确保导航栏始终在极光背景之上
 */

'use client';

import { Cat, Clover, Film, Home, Search, Sparkles, Tv } from 'lucide-react';
import { usePathname, useSearchParams } from 'next/navigation';
import { memo, Suspense, useCallback, useMemo } from 'react';

import FastLink from './FastLink';
import { useSite } from './SiteProvider';
import { ThemeToggle } from './ThemeToggle';
import { UserMenu } from './UserMenu';

/**
 * 导航项配置
 * 使用静态配置避免每次渲染重建数组
 */
const NAV_ITEMS = [
  { href: '/', icon: Home, label: '首页', chip: 'chip-home', type: 'exact' },
  {
    href: '/search',
    icon: Search,
    label: '搜索',
    chip: 'chip-search',
    type: 'exact',
  },
  {
    href: '/douban?type=movie',
    icon: Film,
    label: '电影',
    chip: 'chip-movie',
    type: 'douban',
    doubanType: 'movie',
  },
  {
    href: '/douban?type=tv',
    icon: Tv,
    label: '剧集',
    chip: 'chip-tv',
    type: 'douban',
    doubanType: 'tv',
  },
  {
    href: '/douban?type=anime',
    icon: Cat,
    label: '动漫',
    chip: 'chip-anime',
    type: 'douban',
    doubanType: 'anime',
  },
  {
    href: '/douban?type=show',
    icon: Clover,
    label: '综艺',
    chip: 'chip-show',
    type: 'douban',
    doubanType: 'show',
  },
] as const;

// 内部导航组件，使用 useSearchParams
function NavItems() {
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // 缓存当前 type 参数，避免重复获取
  const currentType = useMemo(() => searchParams.get('type'), [searchParams]);

  // 精确路径匹配
  const isActive = useCallback((href: string) => pathname === href, [pathname]);

  // 豆瓣分类匹配
  const isDoubanActive = useCallback(
    (type: string) => pathname.startsWith('/douban') && currentType === type,
    [pathname, currentType],
  );

  return (
    <>
      {NAV_ITEMS.map((item) => {
        const Icon = item.icon;
        const active =
          item.type === 'douban' && item.doubanType
            ? isDoubanActive(item.doubanType)
            : isActive(item.href);

        return (
          <FastLink
            key={item.href}
            href={item.href}
            useTransitionNav
            className={`inline-flex items-center gap-2 rounded-full px-4 py-2 text-sm font-medium
              transition-all duration-300 ease-out
              glass-chip chip-glow chip-theme ${item.chip}
              hover:scale-[1.02] active:scale-[0.98]
              ${active
                ? 'ring-2 ring-purple-500/40 dark:ring-purple-400/40 shadow-lg shadow-purple-500/20'
                : 'hover:shadow-md'
              }`}
          >
            <Icon className={`h-4 w-4 transition-transform duration-300 ${active ? 'scale-110' : ''}`} />
            <span>{item.label}</span>
          </FastLink>
        );
      })}
    </>
  );
}

function TopNavbar() {
  const { siteName } = useSite();

  return (
    <header
      className='hidden md:block fixed top-0 left-0 right-0 z-900'
      style={{
        // CSS 渲染隔离：防止背景层变化触发导航栏重排
        contain: 'layout paint',
      }}
    >
      <div className='mx-auto max-w-7xl px-4'>
        {/* Aurora 玻璃导航栏 */}
        <div className='mt-3 rounded-2xl relative overflow-hidden'>
          {/* 背景层 */}
          <div className='absolute inset-0 bg-white/70 dark:bg-gray-950/60 backdrop-blur-2xl rounded-2xl' />

          {/* 边框和阴影 */}
          <div className='absolute inset-0 rounded-2xl border border-white/30 dark:border-white/10 shadow-xl shadow-purple-500/5 dark:shadow-purple-500/10' />

          {/* 顶部微光线 */}
          <div className='absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/30 to-transparent' />

          {/* 内容 */}
          <nav className='relative flex items-center justify-between h-16 px-5'>
            {/* Left: Logo */}
            <div className='flex items-center gap-2 min-w-0'>
              <FastLink
                href='/'
                useTransitionNav
                className='shrink-0 select-none flex items-center gap-2 group'
              >
                <Sparkles className='w-6 h-6 text-purple-500 dark:text-purple-400 group-hover:animate-spin transition-all duration-700' />
                <span className='text-xl font-black tracking-tight deco-brand'>
                  {siteName || 'QuantumTV'}
                </span>
              </FastLink>
            </div>

            {/* Center: Navigation Items */}
            <div className='flex items-center justify-center gap-2 flex-wrap'>
              <Suspense fallback={null}>
                <NavItems />
              </Suspense>
            </div>

            {/* Right: Theme + User */}
            <div className='flex items-center gap-3'>
              <ThemeToggle />
              <UserMenu />
            </div>
          </nav>
        </div>
      </div>
    </header>
  );
}

// React.memo: 防止父组件重绘导致不必要的渲染
export default memo(TopNavbar);
