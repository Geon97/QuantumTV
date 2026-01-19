'use client';

/**
 * Search Page Loading Component
 */
export default function Loading() {
  return (
    <div className="w-full min-h-screen">
      {/* Mock header space */}
      <div className="h-14 md:h-24" />

      <div className="px-3 sm:px-10 py-6">
        {/* Search bar skeleton */}
        <div className="mb-6 h-12 w-full rounded-xl bg-gray-200 dark:bg-gray-800 animate-pulse" />

        {/* Results skeleton */}
        <div className="grid grid-cols-3 gap-x-2 gap-y-10 sm:grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] sm:gap-x-6">
          {Array.from({ length: 9 }).map((_, i) => (
            <div key={i} className="w-full">
              <div className="relative aspect-[2/3] w-full overflow-hidden rounded-xl bg-gray-200 dark:bg-gray-800 animate-pulse" />
              <div className="mt-2 h-4 w-3/4 rounded bg-gray-200 dark:bg-gray-800 animate-pulse" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
