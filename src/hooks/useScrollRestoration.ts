import { usePathname, useSearchParams } from 'next/navigation';
import { useEffect, useRef } from 'react';

const STORAGE_PREFIX = 'scrollPos:';
const SAVE_THROTTLE_MS = 250;
// 异常向上跳变阈值：路由导航的 scroll-to-top 通常会让位置一次跳变上千像素
// 用户主动滚动不会产生这种突变，因此可以安全过滤
const PROGRAMMATIC_JUMP_THRESHOLD = 400;
const PROGRAMMATIC_JUMP_MIN_OLD = 200;

const readScrollTop = (): number => {
  if (typeof window === 'undefined') return 0;
  const value =
    document.body.scrollTop ||
    document.documentElement.scrollTop ||
    window.scrollY ||
    0;
  return value;
};

const writeScrollTop = (value: number) => {
  if (typeof window === 'undefined') return;
  try {
    window.scrollTo({ top: value, left: 0, behavior: 'auto' });
  } catch {
    // ignore
  }
  try {
    document.documentElement.scrollTop = value;
  } catch {
    // ignore
  }
  try {
    document.body.scrollTop = value;
  } catch {
    // ignore
  }
};

const buildKey = (pathname: string, search: string) =>
  `${STORAGE_PREFIX}${pathname}${search ? `?${search}` : ''}`;

const persist = (key: string, value: number) => {
  try {
    sessionStorage.setItem(key, String(value));
  } catch {
    // ignore
  }
};

interface UseScrollRestorationOptions {
  ready?: boolean;
}

export function useScrollRestoration(
  options: UseScrollRestorationOptions = {},
) {
  const { ready = true } = options;
  const pathname = usePathname() || '';
  const searchParams = useSearchParams();
  const search = searchParams?.toString() ?? '';
  const storageKey = buildKey(pathname, search);

  const positionRef = useRef(0);

  // 滚动时节流保存，避免依赖卸载清理（App Router 的 Router Cache 可能让组件不卸载）
  useEffect(() => {
    let throttleId: ReturnType<typeof setTimeout> | null = null;
    let lastSavedPos = -1;
    let suppressUntil = 0;

    const flushSave = () => {
      if (lastSavedPos !== positionRef.current) {
        persist(storageKey, positionRef.current);
        lastSavedPos = positionRef.current;
      }
    };

    const handleScroll = () => {
      const newPos = readScrollTop();
      const oldPos = positionRef.current;

      // 抑制窗口期内的滚动事件（点击后路由导航触发的 scroll-to-top）
      if (performance.now() < suppressUntil) {
        return;
      }

      // 异常向上跳变（通常由 Next.js 路由 scroll-to-top 或内容高度收缩引起）
      const upwardJump = oldPos - newPos;
      if (
        oldPos > PROGRAMMATIC_JUMP_MIN_OLD &&
        upwardJump > PROGRAMMATIC_JUMP_THRESHOLD
      ) {
        // 临时抑制后续 500ms 内的所有滚动事件（导航过程中可能有多次回弹）
        suppressUntil = performance.now() + 500;
        return;
      }

      positionRef.current = newPos;

      if (throttleId !== null) return;
      throttleId = setTimeout(() => {
        flushSave();
        throttleId = null;
      }, SAVE_THROTTLE_MS);
    };

    // 任意用户点击都立即保存当前位置 - 这是导航前最后的可靠时机
    // 同时短暂抑制后续的 scroll 事件，避免路由导航触发的 scroll-to-top 覆盖正确位置
    const handlePointerDown = () => {
      const current = readScrollTop();
      if (current > 0) {
        positionRef.current = current;
        if (throttleId !== null) {
          clearTimeout(throttleId);
          throttleId = null;
        }
        persist(storageKey, current);
        lastSavedPos = current;
      }
      // 点击后 600ms 内的所有 scroll 视为非用户主动行为
      suppressUntil = performance.now() + 600;
    };

    document.body.addEventListener('scroll', handleScroll, { passive: true });
    window.addEventListener('scroll', handleScroll, { passive: true });
    window.addEventListener('pointerdown', handlePointerDown, true);

    return () => {
      if (throttleId !== null) {
        clearTimeout(throttleId);
        throttleId = null;
      }
      // 兜底：组件卸载或 storageKey 变化时同步保存一次
      flushSave();
      document.body.removeEventListener('scroll', handleScroll);
      window.removeEventListener('scroll', handleScroll);
      window.removeEventListener('pointerdown', handlePointerDown, true);
    };
  }, [storageKey]);

  // 失去可见性时（窗口最小化、切换标签等）保存
  useEffect(() => {
    const handleVisibility = () => {
      if (document.visibilityState === 'hidden') {
        persist(storageKey, positionRef.current);
      }
    };
    const handleHide = () => persist(storageKey, positionRef.current);

    document.addEventListener('visibilitychange', handleVisibility);
    window.addEventListener('pagehide', handleHide);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibility);
      window.removeEventListener('pagehide', handleHide);
    };
  }, [storageKey]);

  // 内容就绪后，尝试恢复上次的位置
  useEffect(() => {
    if (!ready) return;

    let savedY = 0;
    try {
      const raw = sessionStorage.getItem(storageKey);
      if (raw) savedY = parseInt(raw, 10);
    } catch {
      // ignore
    }

    if (!savedY || Number.isNaN(savedY) || savedY <= 0) return;

    let cancelled = false;
    let attempts = 0;
    const maxAttempts = 30;
    let timerId: ReturnType<typeof setTimeout> | null = null;

    const tryRestore = () => {
      if (cancelled) return;
      attempts += 1;

      writeScrollTop(savedY);
      positionRef.current = savedY;

      const current = readScrollTop();
      const matched = Math.abs(current - savedY) < 4;

      if (matched || attempts >= maxAttempts) {
        return;
      }

      timerId = setTimeout(tryRestore, attempts < 8 ? 50 : 120);
    };

    const rafId = requestAnimationFrame(tryRestore);

    return () => {
      cancelled = true;
      cancelAnimationFrame(rafId);
      if (timerId !== null) {
        clearTimeout(timerId);
      }
    };
  }, [storageKey, ready]);
}
