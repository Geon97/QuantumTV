'use client';

/**
 * Play Page Loading Component
 */
export default function Loading() {
  return (
    <div className="w-full min-h-screen bg-black">
      {/* Video player skeleton */}
      <div className="relative w-full aspect-video bg-gray-900 animate-pulse">
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="relative w-16 h-16">
            <div className="absolute inset-0 rounded-full border-2 border-transparent border-t-purple-500 animate-spin" />
            <div className="absolute inset-2 rounded-full border-2 border-transparent border-b-fuchsia-500 animate-spin" style={{ animationDirection: 'reverse', animationDuration: '0.8s' }} />
          </div>
        </div>
      </div>

      {/* Info skeleton */}
      <div className="p-4">
        <div className="h-6 w-2/3 bg-gray-800 rounded animate-pulse mb-3" />
        <div className="h-4 w-1/2 bg-gray-800 rounded animate-pulse" />
      </div>
    </div>
  );
}
