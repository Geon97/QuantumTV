/* eslint-disable no-undef */

'use client';

/// <reference lib="dom" />

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  AnchorHTMLAttributes,
  forwardRef,
  MouseEvent,
  ReactNode,
  useCallback,
  useTransition,
} from 'react';

/**
 * FastLink - 高性能导航链接组件
 *
 * 解决问题：
 * - Next.js App Router 的客户端导航在复杂页面可能卡顿
 * - 用户反馈"改地址栏很快，点链接很慢"
 *
 * 工作原理：
 * 1. 始终使用 next/link 渲染（保持 SSR hydration 一致性）
 * 2. 点击时检测环境：
 *    - Tauri 桌面环境：使用 window.location 硬跳转（绕过 React 重渲染）
 *    - 浏览器环境：使用 next/link 的 SPA 导航或 startTransition
 * 3. forceRefresh 模式：强制使用浏览器硬跳转
 *
 * 使用场景：
 * - 导航栏、底部栏等频繁点击的核心路由
 * - 页面组件复杂、客户端导航卡顿时的兜底方案
 */

interface FastLinkProps extends Omit<
  AnchorHTMLAttributes<HTMLAnchorElement>,
  'href'
> {
  /** 目标路径 */
  href: string;
  /** 子元素 */
  children: ReactNode;
  /**
   * 强制刷新模式
   * - true: 使用 window.location 硬跳转（最快，但会丢失 SPA 状态）
   * - false (默认): 使用 next/link SPA 导航
   */
  forceRefresh?: boolean;
  /**
   * 使用 React Transition 包裹导航（仅在非 Tauri 环境生效）
   * - 将导航标记为低优先级，不阻塞当前 UI 交互
   * - 在 Tauri 环境下会被忽略，始终使用硬跳转
   */
  useTransitionNav?: boolean;
  /** 额外的点击处理 */
  onClick?: (e: MouseEvent<HTMLAnchorElement>) => void;
}

const FastLink = forwardRef<HTMLAnchorElement, FastLinkProps>(
  (
    {
      href,
      children,
      forceRefresh = false,
      useTransitionNav = false,
      onClick,
      className,
      ...rest
    },
    ref,
  ) => {
    const router = useRouter();
    const [, startTransition] = useTransition();

    /**
     * 处理点击事件
     * 在点击时才检测环境，避免 SSR/CSR hydration 不匹配
     */
    const handleClick = useCallback(
      (e: MouseEvent<HTMLAnchorElement>) => {
        // 先执行外部 onClick
        onClick?.(e);

        // 如果外部已阻止默认行为，直接返回
        if (e.defaultPrevented) return;

        // 检查是否按住修饰键（Cmd/Ctrl + 点击应该在新标签打开）
        if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;

        // 外部链接直接走浏览器默认行为
        if (href.startsWith('http://') || href.startsWith('https://')) return;

        // 在点击时检测 Tauri 环境（避免 hydration 不匹配）
        const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
        
        // Tauri 环境或 forceRefresh 模式：使用硬跳转
        if (forceRefresh || isTauri) {
          e.preventDefault();
          window.location.assign(href);
        } else if (useTransitionNav) {
          // 非 Tauri 环境的 Transition 模式
          e.preventDefault();
          startTransition(() => {
            router.push(href);
          });
        }
        // 默认情况：让 next/link 处理
      },
      [href, forceRefresh, useTransitionNav, onClick, router, startTransition],
    );

    // 始终使用 next/link 渲染，保持 SSR 一致性
    // 硬跳转逻辑在 handleClick 中处理
    return (
      <Link
        ref={ref}
        href={href}
        prefetch={false}
        onClick={handleClick}
        className={className}
        {...rest}
      >
        {children}
      </Link>
    );
  },
);

FastLink.displayName = 'FastLink';

export default FastLink;
