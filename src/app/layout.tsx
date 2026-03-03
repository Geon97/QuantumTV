import { invoke } from '@tauri-apps/api/core';
import type { Metadata, Viewport } from 'next';
import { DM_Sans, Space_Grotesk } from 'next/font/google';
import NextTopLoader from 'nextjs-toploader';
import React from 'react';

import './globals.css';

import { RuntimeConfigResponse } from '@/lib/types';

import { GlobalErrorIndicator } from '../components/GlobalErrorIndicator';
import NavbarGate from '../components/NavbarGate';
import ParticleBackground from '../components/ParticleBackground';
import { SiteProvider } from '../components/SiteProvider';
import { ThemeProvider } from '../components/ThemeProvider';
import TopNavbar from '../components/TopNavbar';

const dmSans = DM_Sans({
  subsets: ['latin'],
  variable: '--font-body',
  display: 'swap',
});

const spaceGrotesk = Space_Grotesk({
  subsets: ['latin'],
  variable: '--font-heading',
  display: 'swap',
});

const getRuntimeConfigFallback = (): RuntimeConfigResponse => ({
  storage_type: process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage',
  use_local_source_config:
    (process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage') ===
    'localstorage',
  site_name: process.env.NEXT_PUBLIC_SITE_NAME || 'QuantumTV',
  announcement:
    process.env.ANNOUNCEMENT ||
    '本网站仅提供影视信息搜索服务，所有内容均来自第三方网站。本站不存储任何视频资源，不对任何内容的准确性、合法性、完整性负责。',
  douban_proxy_type:
    process.env.NEXT_PUBLIC_DOUBAN_PROXY_TYPE || 'cmliussss-cdn-tencent',
  douban_proxy: process.env.NEXT_PUBLIC_DOUBAN_PROXY || '',
  douban_image_proxy_type:
    process.env.NEXT_PUBLIC_DOUBAN_IMAGE_PROXY_TYPE ||
    'cmliussss-cdn-tencent',
  douban_image_proxy: process.env.NEXT_PUBLIC_DOUBAN_IMAGE_PROXY || '',
  disable_yellow_filter: process.env.NEXT_PUBLIC_DISABLE_YELLOW_FILTER === 'true',
  fluid_search: process.env.NEXT_PUBLIC_FLUID_SEARCH !== 'false',
  custom_categories: [],
});

const getRuntimeConfig = async (): Promise<RuntimeConfigResponse> => {
  try {
    return await invoke<RuntimeConfigResponse>('get_runtime_config');
  } catch {
    return getRuntimeConfigFallback();
  }
};

export async function generateMetadata(): Promise<Metadata> {
  const runtimeConfig = await getRuntimeConfig();
  return {
    title: runtimeConfig.site_name,
    description: '影视聚合',
  };
}

export const viewport: Viewport = {
  viewportFit: 'cover',
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const runtimeConfig = await getRuntimeConfig();
  const siteName = runtimeConfig.site_name;
  const announcement = runtimeConfig.announcement;

  const injectedRuntimeConfig = {
    STORAGE_TYPE: runtimeConfig.storage_type,
    USE_LOCAL_SOURCE_CONFIG: runtimeConfig.use_local_source_config,
    DOUBAN_PROXY_TYPE: runtimeConfig.douban_proxy_type,
    DOUBAN_PROXY: runtimeConfig.douban_proxy,
    DOUBAN_IMAGE_PROXY_TYPE: runtimeConfig.douban_image_proxy_type,
    DOUBAN_IMAGE_PROXY: runtimeConfig.douban_image_proxy,
    DISABLE_YELLOW_FILTER: runtimeConfig.disable_yellow_filter,
    CUSTOM_CATEGORIES: runtimeConfig.custom_categories,
    FLUID_SEARCH: runtimeConfig.fluid_search,
  };

  return (
    <html lang='zh-CN' suppressHydrationWarning>
      <head>
        <meta
          name='viewport'
          content='width=device-width, initial-scale=1.0, viewport-fit=cover'
        />
        <link rel='apple-touch-icon' href='/icons/icon-192x192.png' />
        <script
          dangerouslySetInnerHTML={{
            __html: `window.RUNTIME_CONFIG = ${JSON.stringify(injectedRuntimeConfig)};`,
          }}
        />
      </head>
      <body
        className={`${dmSans.variable} ${spaceGrotesk.variable} font-sans min-h-screen text-gray-900 dark:text-gray-100 bg-aurora`}
      >
        <NextTopLoader
          color='#a855f7'
          initialPosition={0.08}
          crawlSpeed={200}
          height={3}
          crawl={true}
          showSpinner={false}
          easing='ease'
          speed={200}
          shadow='0 0 10px #a855f7,0 0 5px #a855f7'
        />
        <ThemeProvider
          attribute='class'
          defaultTheme='system'
          enableSystem
          disableTransitionOnChange
        >
          <SiteProvider siteName={siteName} announcement={announcement}>
            <ParticleBackground />
            <NavbarGate>
              <TopNavbar />
            </NavbarGate>
            {children}
            <GlobalErrorIndicator />
          </SiteProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
