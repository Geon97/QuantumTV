'use client';

import { useEffect, useState } from 'react';

// ============================================================================
// Types
// ============================================================================

interface CinematicLoaderProps {
  /** 加载提示文字 */
  message?: string;
  /** 是否显示 */
  visible?: boolean;
  /** 进度 (0-100)，可选 */
  progress?: number;
}

// ============================================================================
// 子组件
// ============================================================================

/** Logo 呼吸闪烁动画 */
const LogoPulse = () => (
  <div className='relative flex items-center justify-center'>
    {/* 外层光晕 - 多层渐变 */}
    <div className='absolute h-40 w-40 animate-pulse rounded-full bg-red-600/10 blur-3xl' />
    <div
      className='absolute h-32 w-32 rounded-full bg-red-500/20 blur-2xl'
      style={{ animation: 'breathe 3s ease-in-out infinite' }}
    />
    <div
      className='absolute h-24 w-24 rounded-full bg-red-400/30 blur-xl'
      style={{ animation: 'breathe 3s ease-in-out infinite 0.5s' }}
    />

    {/* Logo 容器 */}
    <div
      className='relative z-10 flex h-20 w-20 items-center justify-center rounded-2xl shadow-2xl'
      style={{
        background:
          'linear-gradient(135deg, #dc2626 0%, #991b1b 50%, #7f1d1d 100%)',
        boxShadow:
          '0 0 60px rgba(239, 68, 68, 0.4), 0 0 100px rgba(239, 68, 68, 0.2)',
        animation: 'glow 2s ease-in-out infinite',
      }}
    >
      {/* 播放三角形 */}
      <svg
        className='ml-1 h-10 w-10 text-white drop-shadow-lg'
        viewBox='0 0 24 24'
        fill='currentColor'
      >
        <path d='M8 5v14l11-7z' />
      </svg>
    </div>

    {/* 品牌名称 */}
    <div className='absolute -bottom-12 flex items-center gap-0.5'>
      <span className='text-2xl font-bold tracking-widest text-white/90'>
        Deco
      </span>
      <span className='text-2xl font-bold tracking-widest text-red-500'>
        TV
      </span>
    </div>
  </div>
);

/** 底部进度条 */
const ProgressBar = ({ progress }: { progress?: number }) => {
  const [width, setWidth] = useState(0);

  useEffect(() => {
    if (progress !== undefined) {
      setWidth(progress);
      return;
    }

    // 无限动画模式
    const interval = setInterval(() => {
      setWidth((prev) => {
        if (prev >= 100) return 0;
        return prev + 0.5;
      });
    }, 30);

    return () => clearInterval(interval);
  }, [progress]);

  return (
    <div className='relative h-0.5 w-64 overflow-hidden rounded-full bg-white/10'>
      {/* 进度条 */}
      <div
        className='absolute inset-y-0 left-0 rounded-full transition-all duration-100'
        style={{
          width: `${width}%`,
          background:
            'linear-gradient(90deg, #ef4444 0%, #f97316 50%, #ef4444 100%)',
          boxShadow: '0 0 10px rgba(239, 68, 68, 0.6)',
        }}
      />

      {/* 光点 */}
      <div
        className='absolute inset-y-0 h-full w-1 rounded-full bg-white'
        style={{
          left: `${width}%`,
          transform: 'translateX(-50%)',
          boxShadow: '0 0 8px rgba(255, 255, 255, 0.8)',
          opacity: width > 0 ? 1 : 0,
        }}
      />
    </div>
  );
};

/** 打字机文字效果 */
const TypewriterText = ({ text }: { text: string }) => {
  const [displayText, setDisplayText] = useState('');
  const [showCursor, setShowCursor] = useState(true);
  const [phase, setPhase] = useState<'typing' | 'waiting' | 'deleting'>(
    'typing',
  );

  useEffect(() => {
    let index = 0;
    let timeout: NodeJS.Timeout;

    const animate = () => {
      if (phase === 'typing') {
        if (index < text.length) {
          setDisplayText(text.slice(0, index + 1));
          index++;
          timeout = setTimeout(animate, 80 + Math.random() * 40);
        } else {
          setPhase('waiting');
          timeout = setTimeout(() => setPhase('deleting'), 2000);
        }
      } else if (phase === 'deleting') {
        if (index > 0) {
          index--;
          setDisplayText(text.slice(0, index));
          timeout = setTimeout(animate, 40);
        } else {
          setPhase('typing');
          timeout = setTimeout(animate, 500);
        }
      }
    };

    if (phase === 'typing' || phase === 'deleting') {
      timeout = setTimeout(animate, 100);
    }

    return () => clearTimeout(timeout);
  }, [text, phase]);

  // 光标闪烁
  useEffect(() => {
    const interval = setInterval(() => {
      setShowCursor((prev) => !prev);
    }, 530);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className='flex h-6 items-center justify-center text-sm tracking-wider'>
      <span className='text-white/50'>{displayText}</span>
      <span
        className='ml-px inline-block h-4 w-0.5 bg-red-500'
        style={{ opacity: showCursor ? 1 : 0, transition: 'opacity 0.1s' }}
      />
    </div>
  );
};

// ============================================================================
// 主组件
// ============================================================================

export default function CinematicLoader({
  message = 'Loading Resources...',
  visible = true,
  progress,
}: CinematicLoaderProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    if (visible) {
      // 延迟显示，避免闪烁
      const timeout = setTimeout(() => setMounted(true), 50);
      return () => clearTimeout(timeout);
    } else {
      setMounted(false);
    }
  }, [visible]);

  if (!visible) return null;

  return (
    <div
      className='fixed inset-0 z-50 flex flex-col items-center justify-center overflow-hidden'
      style={{
        background: `
          radial-gradient(ellipse 80% 60% at 50% 40%, rgba(127, 29, 29, 0.08) 0%, transparent 60%),
          radial-gradient(ellipse 60% 40% at 30% 70%, rgba(185, 28, 28, 0.05) 0%, transparent 50%),
          radial-gradient(ellipse 50% 30% at 70% 30%, rgba(220, 38, 38, 0.04) 0%, transparent 40%),
          linear-gradient(180deg, #0a0a0a 0%, #111111 40%, #0d0d0d 100%)
        `,
      }}
    >
      {/* 磨砂纹理 */}
      <div
        className='pointer-events-none absolute inset-0'
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 512 512' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.7' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E")`,
          opacity: 0.03,
        }}
      />

      {/* 顶部暗角 */}
      <div className='pointer-events-none absolute inset-x-0 top-0 h-40 bg-linear-to-b from-black/60 to-transparent' />

      {/* 主内容 */}
      <div
        className='relative flex flex-col items-center gap-20'
        style={{
          opacity: mounted ? 1 : 0,
          transform: mounted
            ? 'translateY(0) scale(1)'
            : 'translateY(30px) scale(0.95)',
          transition: 'all 0.8s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        {/* Logo */}
        <LogoPulse />

        {/* 底部区域 */}
        <div className='flex flex-col items-center gap-6'>
          {/* 进度条 */}
          <ProgressBar progress={progress} />

          {/* 打字机文字 */}
          <TypewriterText text={message} />
        </div>
      </div>

      {/* 底部暗角 */}
      <div className='pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-linear-to-t from-black/60 to-transparent' />

      {/* 四角装饰线 */}
      <div className='pointer-events-none absolute left-8 top-8 h-16 w-16 border-l-2 border-t-2 border-white/5' />
      <div className='pointer-events-none absolute right-8 top-8 h-16 w-16 border-r-2 border-t-2 border-white/5' />
      <div className='pointer-events-none absolute bottom-8 left-8 h-16 w-16 border-b-2 border-l-2 border-white/5' />
      <div className='pointer-events-none absolute bottom-8 right-8 h-16 w-16 border-b-2 border-r-2 border-white/5' />

      {/* 全局样式 */}
      <style jsx>{`
        @keyframes breathe {
          0%,
          100% {
            opacity: 0.3;
            transform: scale(1);
          }
          50% {
            opacity: 0.6;
            transform: scale(1.1);
          }
        }

        @keyframes glow {
          0%,
          100% {
            box-shadow:
              0 0 40px rgba(239, 68, 68, 0.3),
              0 0 80px rgba(239, 68, 68, 0.15);
          }
          50% {
            box-shadow:
              0 0 60px rgba(239, 68, 68, 0.5),
              0 0 120px rgba(239, 68, 68, 0.25);
          }
        }
      `}</style>
    </div>
  );
}

// 命名导出
export { CinematicLoader };
