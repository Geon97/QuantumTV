'use client';

import { Search, Sparkles } from 'lucide-react';
import Link from 'next/link';

import { BackButton } from './BackButton';
import { useSite } from './SiteProvider';
import { ThemeToggle } from './ThemeToggle';
import { UserMenu } from './UserMenu';

interface MobileHeaderProps {
  showBackButton?: boolean;
}

const MobileHeader = ({ showBackButton = false }: MobileHeaderProps) => {
  const { siteName } = useSite();

  return (
    <header
      className='fixed left-0 right-0 top-0 z-[85] w-full lg:hidden'
      style={{ paddingTop: 'env(safe-area-inset-top)' }}
    >
      <div className='absolute inset-0 border-b border-white/20 bg-white/65 backdrop-blur-2xl dark:border-white/10 dark:bg-gray-950/75' />
      <div className='absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/50 to-transparent' />

      <div
        className='relative flex items-center justify-between px-3 max-[375px]:px-2.5 min-[834px]:px-6'
        style={{ height: 'var(--app-mobile-header-height)' }}
      >
        <div className='flex items-center gap-1'>
          <Link
            href='/search'
            aria-label='进入搜索'
            className='tap-target flex items-center justify-center rounded-xl text-gray-700 transition-colors hover:bg-black/5 active:scale-95 dark:text-gray-200 dark:hover:bg-white/10'
          >
            <Search className='h-5 w-5' strokeWidth={2} />
          </Link>
          {showBackButton && <BackButton />}
        </div>

        <div className='flex items-center gap-1'>
          <ThemeToggle />
          <UserMenu />
        </div>
      </div>

      <div className='pointer-events-none absolute inset-0 flex items-center justify-center'>
        <Link
          href='/'
          className='pointer-events-auto flex max-w-[54vw] items-center gap-1.5 truncate'
        >
          <Sparkles className='h-5 w-5 shrink-0 text-purple-500 dark:text-purple-400' />
          <span className='truncate text-sm font-black tracking-tight text-gradient-primary max-[375px]:text-[0.78rem] min-[834px]:text-base'>
            {siteName}
          </span>
        </Link>
      </div>
    </header>
  );
};

export default MobileHeader;
