'use client';

import { Cat, Clover, Film, Home, Search, Sparkles, Tv } from 'lucide-react';
import { usePathname, useSearchParams } from 'next/navigation';
import { memo, Suspense, useCallback, useMemo } from 'react';

import FastLink from './FastLink';
import { useSite } from './SiteProvider';
import { ThemeToggle } from './ThemeToggle';
import { UserMenu } from './UserMenu';

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

function NavItems() {
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const currentType = useMemo(() => searchParams.get('type'), [searchParams]);

  const isActive = useCallback((href: string) => pathname === href, [pathname]);

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
            className={`tap-target inline-flex shrink-0 cursor-pointer items-center gap-2 rounded-full px-3 py-2 text-sm font-medium min-[1440px]:text-[0.95rem]
              transition-colors duration-200
              glass-chip chip-glow chip-theme ${item.chip}
              ${
                active
                  ? 'ring-2 ring-purple-500/40 dark:ring-purple-400/40 shadow-md shadow-purple-500/20'
                  : 'hover:shadow-sm'
              }`}
          >
            <Icon
              className={`h-4 w-4 ${active ? 'text-purple-600 dark:text-purple-300' : ''}`}
            />
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
      className='fixed left-0 right-0 top-0 z-[80] hidden lg:block'
      style={{ contain: 'layout paint' }}
    >
      <div className='mx-auto w-full max-w-[1720px] px-3 min-[834px]:px-7 lg:px-8 min-[1440px]:px-10'>
        <div className='relative mt-3 overflow-hidden rounded-2xl min-[1440px]:mt-4'>
          <div className='absolute inset-0 rounded-2xl bg-white/75 backdrop-blur-2xl dark:bg-gray-950/70' />
          <div className='absolute inset-0 rounded-2xl border border-white/30 shadow-xl shadow-purple-500/10 dark:border-white/10' />
          <div className='absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/40 to-transparent' />

          <nav className='relative grid h-16 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3 px-4 min-[1440px]:h-[4.25rem]'>
            <FastLink
              href='/'
              useTransitionNav
              className='flex min-w-0 justify-self-start select-none items-center gap-2'
            >
              <Sparkles className='h-6 w-6 text-purple-500 dark:text-purple-400' />
              <span className='truncate text-lg font-black tracking-tight deco-brand xl:text-xl'>
                {siteName || 'QuantumTV'}
              </span>
            </FastLink>

            <div className='justify-self-center min-w-0'>
              <div className='mx-auto flex max-w-[54vw] items-center justify-center gap-1 overflow-x-auto py-1 scrollbar-hide'>
                <Suspense fallback={null}>
                  <NavItems />
                </Suspense>
              </div>
            </div>

            <div className='flex min-w-0 justify-self-end items-center gap-2'>
              <ThemeToggle />
              <UserMenu />
            </div>
          </nav>
        </div>
      </div>
    </header>
  );
}

export default memo(TopNavbar);
