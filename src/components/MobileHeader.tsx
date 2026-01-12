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
      className='md:hidden fixed top-0 left-0 right-0 z-999 w-full'
      style={{
        // Android刘海屏/状态栏适配
        paddingTop: 'env(safe-area-inset-top)',
      }}
    >
      {/* 磨砂玻璃背景层 */}
      <div className='absolute inset-0 bg-white/60 dark:bg-gray-950/70 backdrop-blur-2xl border-b border-white/20 dark:border-white/5' />

      {/* 顶部微光效果 */}
      <div className='absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-purple-500/50 to-transparent' />

      <div className='relative h-14 flex items-center justify-between px-4'>
        {/* 左侧：搜索按钮、返回按钮 */}
        <div className='flex items-center gap-1'>
          <Link
            href='/search'
            className='w-10 h-10 rounded-xl flex items-center justify-center text-gray-600 dark:text-gray-300 hover:bg-black/5 dark:hover:bg-white/10 active:scale-95 transition-all duration-200'
          >
            <Search className='w-5 h-5' strokeWidth={2} />
          </Link>
          {showBackButton && <BackButton />}
        </div>

        {/* 右侧按钮 */}
        <div className='flex items-center gap-1'>
          <ThemeToggle />
          <UserMenu />
        </div>
      </div>

      {/* 中间：Logo（绝对居中） */}
      <div className='absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none' style={{ marginTop: 'calc(env(safe-area-inset-top) / 2)' }}>
        <Link
          href='/'
          className='pointer-events-auto flex items-center gap-1.5 group'
        >
          <Sparkles className='w-5 h-5 text-purple-500 dark:text-purple-400 group-hover:animate-spin transition-all duration-500' />
          <span className='text-xl font-black tracking-tight bg-gradient-to-r from-violet-600 via-purple-500 to-fuchsia-500 dark:from-violet-400 dark:via-purple-400 dark:to-fuchsia-400 bg-clip-text text-transparent'>
            {siteName}
          </span>
        </Link>
      </div>
    </header>
  );
};

export default MobileHeader;
