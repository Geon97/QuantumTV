'use client';

/**
 * Root Loading Component
 * Shows during page transitions in Next.js App Router
 */
export default function Loading() {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-white/80 dark:bg-gray-950/80 backdrop-blur-sm">
      <div className="flex flex-col items-center gap-4">
        {/* Cinematic Loader */}
        <div className="relative w-12 h-12">
          <div className="absolute inset-0 rounded-full border-2 border-transparent border-t-purple-500 animate-spin" />
          <div className="absolute inset-1 rounded-full border-2 border-transparent border-b-fuchsia-500 animate-spin" style={{ animationDirection: 'reverse', animationDuration: '0.8s' }} />
        </div>
        <span className="text-sm text-gray-500 dark:text-gray-400 animate-pulse">
          加载中...
        </span>
      </div>
    </div>
  );
}
