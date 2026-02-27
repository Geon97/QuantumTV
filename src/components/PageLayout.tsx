import React from 'react';

import { BackButton } from './BackButton';
import MobileBottomNav from './MobileBottomNav';
import MobileHeader from './MobileHeader';

interface PageLayoutProps {
  children: React.ReactNode;
  activePath?: string;
}

const PageLayout = ({ children, activePath = '/' }: PageLayoutProps) => {
  const showBackButton = ['/play', '/live'].includes(activePath);

  return (
    <div className='w-full min-h-dvh'>
      <MobileHeader showBackButton={showBackButton} />

      <div className='relative w-full min-h-dvh'>
        {showBackButton && (
          <div className='absolute left-4 top-4 z-[70] hidden lg:flex'>
            <BackButton />
          </div>
        )}

        <main
          className='min-h-dvh lg:min-h-0'
          style={{
            marginTop: 'var(--app-top-offset)',
            paddingBottom: 'var(--app-bottom-offset)',
          }}
        >
          {children}
        </main>
      </div>

      <div className='lg:hidden'>
        <MobileBottomNav activePath={activePath} />
      </div>
    </div>
  );
};

export default PageLayout;
