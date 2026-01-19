import type { Metadata, Viewport } from 'next';
import { DM_Sans, Space_Grotesk } from 'next/font/google';
import NextTopLoader from 'nextjs-toploader';
import React from 'react';

import './globals.css';

import { getConfig } from '@/lib/config';

import { GlobalErrorIndicator } from '../components/GlobalErrorIndicator';
import NavbarGate from '../components/NavbarGate';
import ParticleBackground from '../components/ParticleBackground';
import { SiteProvider } from '../components/SiteProvider';
import { ThemeProvider } from '../components/ThemeProvider';
import TopNavbar from '../components/TopNavbar';

// Font configuration - Tech Startup Pairing
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

// // // // // export const dynamic = 'force-dynamic';

// 动态生成 metadata，支持配置更新后的标题变化
export async function generateMetadata(): Promise<Metadata> {
  const isStatic = process.env.NEXT_PUBLIC_STATIC_EXPORT === 'true';
  const storageType = process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage';

  let siteName = process.env.NEXT_PUBLIC_SITE_NAME || 'QuantumTV';

  // 静态导出时不调用 getConfig，使用环境变量
  if (!isStatic && storageType !== 'localstorage') {
    try {
      const config = getConfig();
      siteName = config.SiteConfig.SiteName;
    } catch {
      // 静态构建时可能失败，使用默认值
    }
  }

  return {
    title: siteName,
    description: '影视聚合',
    manifest: '/manifest.json',
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
  const isStatic = process.env.NEXT_PUBLIC_STATIC_EXPORT === 'true';
  const storageType = process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage';

  let siteName = process.env.NEXT_PUBLIC_SITE_NAME || 'QuantumTV';
  let announcement =
    process.env.ANNOUNCEMENT ||
    '本网站仅提供影视信息搜索服务，所有内容均来自第三方网站。本站不存储任何视频资源，不对任何内容的准确性、合法性、完整性负责。';

  let doubanProxyType =
    process.env.NEXT_PUBLIC_DOUBAN_PROXY_TYPE || 'cmliussss-cdn-tencent';
  let doubanProxy = process.env.NEXT_PUBLIC_DOUBAN_PROXY || '';
  let doubanImageProxyType =
    process.env.NEXT_PUBLIC_DOUBAN_IMAGE_PROXY_TYPE || 'cmliussss-cdn-tencent';
  let doubanImageProxy = process.env.NEXT_PUBLIC_DOUBAN_IMAGE_PROXY || '';
  let disableYellowFilter =
    process.env.NEXT_PUBLIC_DISABLE_YELLOW_FILTER === 'true';
  let fluidSearch = process.env.NEXT_PUBLIC_FLUID_SEARCH !== 'false';
  let customCategories = [] as {
    name: string;
    type: 'movie' | 'tv';
    query: string;
  }[];

  // 静态导出时不调用 getConfig，使用环境变量
  if (!isStatic && storageType !== 'localstorage') {
    try {
      const config = getConfig();
      siteName = config.SiteConfig.SiteName;
      announcement = config.SiteConfig.Announcement;

      doubanProxyType = config.SiteConfig.DoubanProxyType;
      doubanProxy = config.SiteConfig.DoubanProxy;
      doubanImageProxyType = config.SiteConfig.DoubanImageProxyType;
      doubanImageProxy = config.SiteConfig.DoubanImageProxy;
      disableYellowFilter = config.SiteConfig.DisableYellowFilter;
      customCategories = config.CustomCategories.filter(
        (category) => !category.disabled,
      ).map((category) => ({
        name: category.name || '',
        type: category.type,
        query: category.query,
      }));
      fluidSearch = config.SiteConfig.FluidSearch;
    } catch {
      // 静态构建时可能失败，使用默认值
    }
  }

  // 将运行时配置注入到全局 window 对象，供客户端在运行时读取
  const runtimeConfig = {
    STORAGE_TYPE: process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage',
    DOUBAN_PROXY_TYPE: doubanProxyType,
    DOUBAN_PROXY: doubanProxy,
    DOUBAN_IMAGE_PROXY_TYPE: doubanImageProxyType,
    DOUBAN_IMAGE_PROXY: doubanImageProxy,
    DISABLE_YELLOW_FILTER: disableYellowFilter,
    CUSTOM_CATEGORIES: customCategories,
    FLUID_SEARCH: fluidSearch,
  };

  return (
    <html lang='zh-CN' suppressHydrationWarning>
      <head>
        <meta
          name='viewport'
          content='width=device-width, initial-scale=1.0, viewport-fit=cover'
        />
        <link rel='apple-touch-icon' href='/icons/icon-192x192.png' />
        {/* 将配置序列化后直接写入脚本，浏览器端可通过 window.RUNTIME_CONFIG 获取 */}
        <script
          dangerouslySetInnerHTML={{
            __html: `window.RUNTIME_CONFIG = ${JSON.stringify(runtimeConfig)};`,
          }}
        />
      </head>
      <body
        className={`${dmSans.variable} ${spaceGrotesk.variable} font-sans min-h-screen text-gray-900 dark:text-gray-100 bg-aurora`}
      >
        {/* 顶部进度条 - Aurora 紫色主题 */}
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
