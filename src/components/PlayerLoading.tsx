'use client';

import { useEffect, useState } from 'react';

// ============================================================================
// Types
// ============================================================================

interface PlayerLoadingProps {
  /** 加载提示文字 */
  message?: string;
  /** 是否显示 */
  visible?: boolean;
}

// ============================================================================
// 子组件
// ============================================================================

/** 呼吸灯 Logo */
const PulseLogo = () => (
  <div className='relative flex items-center justify-center'>
    {/* 外层光晕 */}
    <div className='absolute h-32 w-32 animate-pulse rounded-full bg-red-500/20 blur-xl' />
    <div
      className='absolute h-24 w-24 rounded-full bg-red-500/30 blur-lg'
      style={{ animation: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite' }}
    />

    {/* Logo 容器 */}
    <div className='relative z-10 flex h-20 w-20 items-center justify-center rounded-2xl bg-linear-to-br from-red-500 to-red-700 shadow-2xl shadow-red-500/30'>
      {/* QuantumTV Logo - 播放三角形 */}
      <svg
        className='ml-1 h-10 w-10 text-white drop-shadow-lg'
        viewBox='0 0 24 24'
        fill='currentColor'
      >
        <path d='M8 5v14l11-7z' />
      </svg>
    </div>

    {/* 品牌名称 */}
    <span className='absolute -bottom-10 text-xl font-bold tracking-wider text-white/90'>
      Deco<span className='text-red-500'>TV</span>
    </span>
  </div>
);

/** 环形进度条 */
const RingLoader = () => (
  <div className='absolute inset-0 flex items-center justify-center'>
    {/* 外环 - 静态轨道 */}
    <div className='absolute h-36 w-36 rounded-full border-2 border-white/5' />

    {/* 进度环 - 旋转动画 */}
    <svg
      className='absolute h-36 w-36 -rotate-90'
      viewBox='0 0 100 100'
      style={{ animation: 'spin 2s linear infinite' }}
    >
      <circle
        cx='50'
        cy='50'
        r='48'
        fill='none'
        stroke='url(#loaderGradient)'
        strokeWidth='1.5'
        strokeLinecap='round'
        strokeDasharray='75 226'
        className='origin-center'
      />
      <defs>
        <linearGradient id='loaderGradient' x1='0%' y1='0%' x2='100%' y2='0%'>
          <stop offset='0%' stopColor='#ef4444' stopOpacity='1' />
          <stop offset='50%' stopColor='#f97316' stopOpacity='0.8' />
          <stop offset='100%' stopColor='#ef4444' stopOpacity='0' />
        </linearGradient>
      </defs>
    </svg>

    {/* 内环光点 */}
    <div
      className='absolute h-36 w-36'
      style={{ animation: 'spin 3s linear infinite reverse' }}
    >
      <div className='absolute left-1/2 top-0 h-2 w-2 -translate-x-1/2 -translate-y-1 rounded-full bg-red-400 shadow-lg shadow-red-500/50' />
    </div>
  </div>
);

/** 打字机文字效果 */
const TypewriterText = ({ text }: { text: string }) => {
  const [displayText, setDisplayText] = useState('');
  const [showCursor, setShowCursor] = useState(true);

  useEffect(() => {
    let index = 0;
    const interval = setInterval(() => {
      if (index <= text.length) {
        setDisplayText(text.slice(0, index));
        index++;
      } else {
        // 打完后暂停，然后重新开始
        setTimeout(() => {
          index = 0;
        }, 2000);
      }
    }, 100);

    return () => clearInterval(interval);
  }, [text]);

  // 光标闪烁
  useEffect(() => {
    const cursorInterval = setInterval(() => {
      setShowCursor((prev) => !prev);
    }, 530);

    return () => clearInterval(cursorInterval);
  }, []);

  return (
    <div className='flex items-center justify-center text-sm text-white/60'>
      <span className='bg-linear-to-r from-white/50 via-white/80 to-white/50 bg-clip-text text-transparent'>
        {displayText}
      </span>
      <span
        className='ml-0.5 inline-block h-4 w-0.5 bg-red-500'
        style={{ opacity: showCursor ? 1 : 0, transition: 'opacity 0.1s' }}
      />
    </div>
  );
};

// ============================================================================
// 主组件
// ============================================================================

export default function PlayerLoading({
  message = '正在获取视频详情...',
  visible = true,
}: PlayerLoadingProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!visible) return null;

  return (
    <div
      className='fixed inset-0 z-50 flex flex-col items-center justify-center overflow-hidden'
      style={{
        background: `
          radial-gradient(ellipse 80% 50% at 50% 50%, rgba(127, 29, 29, 0.15) 0%, transparent 50%),
          radial-gradient(ellipse 60% 40% at 70% 60%, rgba(185, 28, 28, 0.1) 0%, transparent 40%),
          linear-gradient(to bottom, #0a0a0a 0%, #111111 50%, #0a0a0a 100%)
        `,
      }}
    >
      {/* 背景噪点纹理 */}
      <div
        className='pointer-events-none absolute inset-0 opacity-20'
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E")`,
        }}
      />

      {/* 顶部渐变遮罩 */}
      <div className='pointer-events-none absolute inset-x-0 top-0 h-32 bg-linear-to-b from-black/50 to-transparent' />

      {/* 主内容区域 */}
      <div
        className='relative flex flex-col items-center gap-16'
        style={{
          opacity: mounted ? 1 : 0,
          transform: mounted ? 'translateY(0)' : 'translateY(20px)',
          transition: 'all 0.8s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        {/* Logo + 环形进度条 */}
        <div className='relative h-40 w-40'>
          <RingLoader />
          <div className='flex h-full w-full items-center justify-center'>
            <PulseLogo />
          </div>
        </div>

        {/* 加载文字 */}
        <div className='flex flex-col items-center gap-3'>
          <TypewriterText text={message} />

          {/* 底部装饰线 */}
          <div className='mt-4 flex items-center gap-2'>
            <div className='h-px w-16 bg-linear-to-r from-transparent via-red-500/50 to-transparent' />
            <div className='h-1.5 w-1.5 animate-pulse rounded-full bg-red-500' />
            <div className='h-px w-16 bg-linear-to-r from-transparent via-red-500/50 to-transparent' />
          </div>
        </div>
      </div>

      {/* 底部渐变遮罩 */}
      <div className='pointer-events-none absolute inset-x-0 bottom-0 h-32 bg-linear-to-t from-black/50 to-transparent' />

      {/* 全局样式 */}
      <style jsx>{`
        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }
        @keyframes pulse {
          0%,
          100% {
            opacity: 0.4;
            transform: scale(1);
          }
          50% {
            opacity: 0.8;
            transform: scale(1.05);
          }
        }
      `}</style>
    </div>
  );
}

// 命名导出便于按需导入
export { PlayerLoading };
