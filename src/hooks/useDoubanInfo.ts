/* eslint-disable no-console */
'use client';

import { useCallback, useEffect, useState } from 'react';

// ============================================================================
// Types
// ============================================================================

/** 演员/导演信息 */
export interface DoubanCelebrity {
  id: string;
  name: string;
  alt?: string;
  avatars?: {
    small: string;
    medium: string;
    large: string;
  };
  roles?: string[];
}

/** 电影详情 */
export interface DoubanMovieDetail {
  id: string;
  title: string;
  original_title?: string;
  alt?: string;
  rating?: {
    max: number;
    average: number;
    stars: string;
    min: number;
  };
  ratings_count?: number;
  wish_count?: number;
  collect_count?: number;
  images?: {
    small: string;
    medium: string;
    large: string;
  };
  subtype?: string;
  directors?: DoubanCelebrity[];
  casts?: DoubanCelebrity[];
  writers?: DoubanCelebrity[];
  pubdates?: string[];
  year?: string;
  genres?: string[];
  countries?: string[];
  mainland_pubdate?: string;
  aka?: string[];
  summary?: string;
  tags?: Array<{ name: string; count: number }>;
  durations?: string[];
  seasons_count?: number;
  episodes_count?: number;
}

/** 用户评论 */
export interface DoubanComment {
  id: string;
  created_at: string;
  content: string;
  useful_count: number;
  rating?: {
    max: number;
    value: number;
    min: number;
  };
  author: {
    id: string;
    uid: string;
    name: string;
    avatar: string;
    alt?: string;
  };
}

/** 评论列表响应 */
export interface DoubanCommentsResponse {
  start: number;
  count: number;
  total: number;
  comments: DoubanComment[];
}

/** Hook 返回类型 */
export interface UseDoubanInfoResult {
  // 详情数据
  detail: DoubanMovieDetail | null;
  detailLoading: boolean;
  detailError: Error | null;

  // 评论数据
  comments: DoubanComment[];
  commentsLoading: boolean;
  commentsError: Error | null;
  commentsTotal: number;

  // 刷新函数
  refreshDetail: () => Promise<void>;
  refreshComments: () => Promise<void>;
}

// ============================================================================
// Fetch Helpers
// ============================================================================

/**
 * 检测是否在 Tauri 环境
 */
function isTauriEnvironment(): boolean {
  if (typeof window === 'undefined') return false;
  const tauri = (window as any).__TAURI__;
  const runtimeStorageType =
    (window as any).RUNTIME_CONFIG?.STORAGE_TYPE || 'localstorage';
  return runtimeStorageType === 'localstorage' && !!tauri?.core?.invoke;
}

/**
 * 通过代理接口获取豆瓣电影详情
 */
async function fetchDoubanDetail(
  doubanId: string | number,
): Promise<DoubanMovieDetail> {
  // Tauri 环境：使用 get_douban_data 命令直接获取
  if (isTauriEnvironment()) {
    const tauri = (window as any).__TAURI__;
    try {
      // 使用 get_douban_data 命令，参数为 subject_id 和 data_type
      const result = await tauri.core.invoke('get_douban_data', {
        subjectId: String(doubanId),
        dataType: 'full',
      });
      if (result && typeof result === 'object') {
        return result as DoubanMovieDetail;
      }
      throw new Error('返回数据格式错误');
    } catch (error) {
      console.error('[useDoubanInfo] Tauri get_douban_data failed:', error);
      throw new Error(`获取详情失败: ${(error as Error).message}`);
    }
  }

  // 云端模式：使用 API
  const response = await fetch(
    `/api/douban/proxy?path=movie/subject/${doubanId}`,
  );

  if (!response.ok) {
    throw new Error(`获取详情失败: ${response.status}`);
  }

  return response.json();
}

/**
 * 通过代理接口获取豆瓣评论
 */
async function fetchDoubanComments(
  doubanId: string | number,
  start = 0,
  count = 6,
): Promise<DoubanCommentsResponse> {
  // Tauri 环境：使用 get_douban_data 命令获取评论
  if (isTauriEnvironment()) {
    const tauri = (window as any).__TAURI__;
    try {
      const result = await tauri.core.invoke('get_douban_data', {
        subjectId: String(doubanId),
        dataType: 'comments',
        start: start,
        count: count,
      });
      if (result && typeof result === 'object') {
        return result as DoubanCommentsResponse;
      }
      throw new Error('返回数据格式错误');
    } catch (error) {
      console.error('[useDoubanInfo] Tauri get_douban_data comments failed:', error);
      throw new Error(`获取评论失败: ${(error as Error).message}`);
    }
  }

  // 云端模式：使用 API
  const response = await fetch(
    `/api/douban/proxy?path=movie/subject/${doubanId}/comments&start=${start}&count=${count}`,
  );

  if (!response.ok) {
    throw new Error(`获取评论失败: ${response.status}`);
  }

  return response.json();
}

// ============================================================================
// Hook
// ============================================================================

/**
 * 豆瓣信息 Hook
 * 用于并行获取电影详情和评论数据
 *
 * @param doubanId - 豆瓣电影 ID
 * @param options - 配置选项
 */
export function useDoubanInfo(
  doubanId: string | number | null | undefined,
  options: {
    /** 是否自动获取详情，默认 true */
    fetchDetail?: boolean;
    /** 是否自动获取评论，默认 true */
    fetchComments?: boolean;
    /** 评论数量，默认 6 */
    commentsCount?: number;
  } = {},
): UseDoubanInfoResult {
  const {
    fetchDetail: shouldFetchDetail = true,
    fetchComments: shouldFetchComments = true,
    commentsCount = 6,
  } = options;

  // 详情状态
  const [detail, setDetail] = useState<DoubanMovieDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<Error | null>(null);

  // 评论状态
  const [comments, setComments] = useState<DoubanComment[]>([]);
  const [commentsLoading, setCommentsLoading] = useState(false);
  const [commentsError, setCommentsError] = useState<Error | null>(null);
  const [commentsTotal, setCommentsTotal] = useState(0);

  // 获取详情
  const refreshDetail = useCallback(async () => {
    if (!doubanId) return;

    setDetailLoading(true);
    setDetailError(null);

    try {
      const data = await fetchDoubanDetail(doubanId);
      setDetail(data);
    } catch (error) {
      console.error('[useDoubanInfo] Failed to fetch detail:', error);
      setDetailError(error instanceof Error ? error : new Error('未知错误'));
    } finally {
      setDetailLoading(false);
    }
  }, [doubanId]);

  // 获取评论
  const refreshComments = useCallback(async () => {
    if (!doubanId) return;

    setCommentsLoading(true);
    setCommentsError(null);

    try {
      const data = await fetchDoubanComments(doubanId, 0, commentsCount);
      setComments(data.comments || []);
      setCommentsTotal(data.total || 0);
    } catch (error) {
      console.error('[useDoubanInfo] Failed to fetch comments:', error);
      setCommentsError(error instanceof Error ? error : new Error('未知错误'));
    } finally {
      setCommentsLoading(false);
    }
  }, [doubanId, commentsCount]);

  // 初始化加载
  useEffect(() => {
    if (!doubanId) {
      // 重置状态
      setDetail(null);
      setComments([]);
      setCommentsTotal(0);
      return;
    }

    // 并行请求
    const promises: Promise<void>[] = [];

    if (shouldFetchDetail) {
      promises.push(refreshDetail());
    }

    if (shouldFetchComments) {
      promises.push(refreshComments());
    }

    Promise.allSettled(promises);
  }, [
    doubanId,
    shouldFetchDetail,
    shouldFetchComments,
    refreshDetail,
    refreshComments,
  ]);

  return {
    detail,
    detailLoading,
    detailError,
    comments,
    commentsLoading,
    commentsError,
    commentsTotal,
    refreshDetail,
    refreshComments,
  };
}

export default useDoubanInfo;
