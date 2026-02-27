import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

interface ScrollableRowProps {
  children: React.ReactNode;
  scrollDistance?: number;
}

export default function ScrollableRow({
  children,
  scrollDistance = 1000,
}: ScrollableRowProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [showLeftScroll, setShowLeftScroll] = useState(false);
  const [showRightScroll, setShowRightScroll] = useState(false);
  const [isHovered, setIsHovered] = useState(false);

  const checkScroll = () => {
    if (!containerRef.current) return;

    const { scrollWidth, clientWidth, scrollLeft } = containerRef.current;
    const threshold = 1;

    setShowRightScroll(scrollWidth - (scrollLeft + clientWidth) > threshold);
    setShowLeftScroll(scrollLeft > threshold);
  };

  useEffect(() => {
    checkScroll();

    window.addEventListener('resize', checkScroll);

    const resizeObserver = new ResizeObserver(() => {
      checkScroll();
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => {
      window.removeEventListener('resize', checkScroll);
      resizeObserver.disconnect();
    };
  }, [children]);

  useEffect(() => {
    if (!containerRef.current) return;

    const observer = new MutationObserver(() => {
      setTimeout(checkScroll, 80);
    });

    observer.observe(containerRef.current, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['style', 'class'],
    });

    return () => observer.disconnect();
  }, []);

  const scrollByDistance = (distance: number) => {
    containerRef.current?.scrollBy({
      left: distance,
      behavior: 'smooth',
    });
  };

  return (
    <div
      className='relative'
      onMouseEnter={() => {
        setIsHovered(true);
        checkScroll();
      }}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div
        ref={containerRef}
        className='flex gap-3 overflow-x-auto px-1 py-1 pb-8 scrollbar-hide sm:gap-4 sm:px-2 sm:pb-9 md:gap-5 lg:pb-10'
        onScroll={checkScroll}
      >
        {children}
      </div>

      {showLeftScroll && (
        <div
          className={`absolute inset-y-0 left-0 z-[60] hidden w-14 items-center bg-gradient-to-r from-white/65 to-transparent pl-1 transition-opacity duration-200 dark:from-gray-950/60 lg:flex ${
            isHovered ? 'opacity-100' : 'opacity-0'
          }`}
        >
          <button
            onClick={() => scrollByDistance(-scrollDistance)}
            className='flex h-10 w-10 items-center justify-center rounded-full border border-gray-200 bg-white/95 text-gray-700 shadow-md transition-colors hover:bg-white dark:border-gray-700 dark:bg-gray-800/90 dark:text-gray-200 dark:hover:bg-gray-700'
            aria-label='向左滚动'
          >
            <ChevronLeft className='h-5 w-5' />
          </button>
        </div>
      )}

      {showRightScroll && (
        <div
          className={`absolute inset-y-0 right-0 z-[60] hidden w-14 items-center justify-end bg-gradient-to-l from-white/65 to-transparent pr-1 transition-opacity duration-200 dark:from-gray-950/60 lg:flex ${
            isHovered ? 'opacity-100' : 'opacity-0'
          }`}
        >
          <button
            onClick={() => scrollByDistance(scrollDistance)}
            className='flex h-10 w-10 items-center justify-center rounded-full border border-gray-200 bg-white/95 text-gray-700 shadow-md transition-colors hover:bg-white dark:border-gray-700 dark:bg-gray-800/90 dark:text-gray-200 dark:hover:bg-gray-700'
            aria-label='向右滚动'
          >
            <ChevronRight className='h-5 w-5' />
          </button>
        </div>
      )}
    </div>
  );
}
