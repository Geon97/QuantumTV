/* eslint-disable @typescript-eslint/no-explicit-any */

'use client';

import { Cat, Clover, Film, Home, Search, Star, Tv } from 'lucide-react';
import { useRouter } from 'next/navigation';
import {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useTransition,
} from 'react';

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

const MobileBottomNav = memo(function MobileBottomNav({
  activePath = '/',
}: MobileBottomNavProps) {
  const router = useRouter();
  const [isPending, startTransition] = useTransition();
  const itemRefs = useRef<(HTMLElement | null)[]>([]);

  const navItems = useMemo(() => {
    const runtimeConfig =
      typeof window !== 'undefined' ? (window as any).RUNTIME_CONFIG : null;

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

  const isActive = useCallback(
    (href: string) => {
      const typeMatch = href.match(/type=([^&]+)/)?.[1];
      const decodedActive = decodeURIComponent(activePath);
      const decodedItemHref = decodeURIComponent(href);

      if (decodedActive === decodedItemHref) return true;
      if (href === '/' && decodedActive === '/') return true;
      if (href === '/search' && decodedActive.startsWith('/search'))
        return true;

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

  const handleNavigation = useCallback(
    (e: React.MouseEvent, href: string) => {
      e.preventDefault();

      const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
      if (isTauri) {
        window.location.assign(href);
        return;
      }

      startTransition(() => {
        router.push(href);
      });
    },
    [router, startTransition],
  );

  useEffect(() => {
    const activeIndex = navItems.findIndex((item) => isActive(item.href));
    if (activeIndex === -1) return;

    const timer = setTimeout(() => {
      itemRefs.current[activeIndex]?.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
        inline: 'center',
      });
    }, 120);

    return () => clearTimeout(timer);
  }, [activePath, navItems, isActive]);

  return (
    <nav
      className={cn(
        'fixed left-1/2 z-[75] w-auto max-w-[96vw] -translate-x-1/2 lg:hidden min-[834px]:max-w-[90vw]',
        isPending && 'opacity-75',
      )}
      style={{
        bottom: 'calc(0.75rem + env(safe-area-inset-bottom))',
      }}
    >
      <div className='absolute inset-0 rounded-2xl border border-white/35 bg-white/75 shadow-xl shadow-black/10 backdrop-blur-2xl dark:border-white/10 dark:bg-gray-950/75 dark:shadow-black/35' />
      <div className='absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/35 to-transparent' />

      <div
        className='relative flex items-center gap-1 overflow-x-auto px-2 py-2 scrollbar-hide max-[375px]:px-1.5 min-[834px]:gap-1.5 min-[834px]:px-3.5'
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
                'tap-target inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-xl px-3 py-2 text-xs font-medium transition-all duration-200 max-[375px]:px-2 min-[834px]:px-4 min-[834px]:text-sm',
                'focus:outline-none active:scale-95',
                active &&
                  `bg-gradient-to-r ${item.activeGradient} text-white shadow-md ${item.glowColor}`,
                !active &&
                  'text-gray-700 hover:bg-black/5 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-white/8 dark:hover:text-white',
              )}
              aria-current={active ? 'page' : undefined}
            >
              <Icon className='h-4 w-4 shrink-0' />
              <span className='whitespace-nowrap'>{item.label}</span>
            </a>
          );
        })}
      </div>
    </nav>
  );
});

export default MobileBottomNav;
