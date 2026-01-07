/**
 * TopNavbar - PC 端顶部导航栏
 *
 * 性能优化策略：
 * 1. FastLink + useTransitionNav: 路由切换标记为"过渡更新"，不阻塞主线程
 * 2. React.memo: 防止父组件重绘导致不必要的渲染
 * 3. contain: layout paint: CSS 渲染隔离，防止 Aurora 背景触发导航栏重排
 * 4. 高 z-index: 确保导航栏始终在极光背景之上
 */

'use client';

import { Cat, Clover, Film, Home, Radio, Search, Tv } from 'lucide-react';
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
  {
    href: '/live',
    icon: Radio,
    label: '直播',
    chip: 'chip-live',
    type: 'exact',
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
            className={`inline-flex items-center gap-2 rounded-full px-4 py-1.5 text-sm hover:opacity-90 transition-all glass-chip chip-glow chip-theme ${item.chip} ${
              active ? 'ring-2 ring-purple-400/60' : ''
            }`}
          >
            <Icon className='h-4 w-4' />
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
        {/* PC 端保留 backdrop-blur 磨砂玻璃效果 */}
        <div className='mt-2 rounded-2xl border border-white/10 bg-white/80 dark:bg-gray-900/80 md:bg-white/30 md:dark:bg-gray-900/40 shadow-[0_0_1px_0_rgba(255,255,255,0.5),0_0_40px_-10px_rgba(99,102,241,0.5)] backdrop-blur-none md:backdrop-blur-xl'>
          <nav className='flex items-center justify-between h-14 px-3'>
            {/* Left: Logo */}
            <div className='flex items-center gap-2 min-w-0'>
              <FastLink
                href='/'
                useTransitionNav
                className='shrink-0 select-none hover:opacity-90 transition-opacity'
              >
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
            <div className='flex items-center gap-2'>
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
