/* eslint-disable no-console */
'use client';

import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';

import { ApiSite, RuntimeConfigResponse } from '@/lib/types';

export interface SourceCategory {
  type_id: string | number;
  type_name: string;
  type_pid?: string | number;
}

export interface UseSourceFilterReturn {
  sources: ApiSite[];
  currentSource: string;
  sourceCategories: SourceCategory[];
  isLoadingSources: boolean;
  isLoadingCategories: boolean;
  error: string | null;
  setCurrentSource: (sourceKey: string) => void;
  refreshSources: () => Promise<void>;
  getFilteredCategories: (
    contentType: 'movie' | 'tv' | 'anime' | 'show',
  ) => SourceCategory[];
}

/**
 * useSourceFilter v2 - 使用 Rust 统一的分类过滤规则
 *
 * 改进：
 * 1. 使用 Rust 的 get_filtered_source_categories 命令
 * 2. 消除 TS 和 Rust 的双重过滤规则
 * 3. 保证桌面端和其他客户端的一致性
 */
export function useSourceFilter(): UseSourceFilterReturn {
  const [sources, setSources] = useState<ApiSite[]>([]);
  const [currentSource, setCurrentSourceState] = useState<string>('auto');
  const [sourceCategories, setSourceCategories] = useState<SourceCategory[]>(
    [],
  );
  const [isLoadingSources, setIsLoadingSources] = useState(false);
  const [isLoadingCategories, setIsLoadingCategories] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSources = useCallback(async () => {
    setIsLoadingSources(true);
    setError(null);

    try {
      let useLocalSourceConfig =
        (process.env.NEXT_PUBLIC_STORAGE_TYPE || 'localstorage') ===
        'localstorage';
      try {
        const runtimeConfig = await invoke<RuntimeConfigResponse>(
          'get_runtime_config',
        );
        useLocalSourceConfig = runtimeConfig.use_local_source_config;
      } catch {
        // Fallback to env default when command is unavailable.
      }

      if (useLocalSourceConfig) {
        const config = await invoke<any>('get_config');
        const sourceConfig = config.SourceConfig || [];
        const enabledSources = sourceConfig
          .filter((s: any) => !s.disabled)
          .map((s: any) => ({
            key: s.key,
            api: s.api,
            name: s.name,
            detail: s.detail,
            is_adult: s.is_adult,
          }));
        setSources(enabledSources);
        return;
      }

      const response = await fetch('/api/search/resources', {
        credentials: 'include',
      });
      if (!response.ok) {
        throw new Error('获取数据源列表失败');
      }
      const data: ApiSite[] = await response.json();
      setSources(data);
    } catch (err) {
      console.error('获取数据源失败', err);
      setError(err instanceof Error ? err.message : '未知错误');
    } finally {
      setIsLoadingSources(false);
    }
  }, []);

  const fetchSourceCategories = useCallback(
    async (sourceKey: string) => {
      if (sourceKey === 'auto') {
        setSourceCategories([]);
        return;
      }

      setIsLoadingCategories(true);
      setError(null);

      try {
        const source = sources.find((s) => s.key === sourceKey);
        if (!source) {
          throw new Error('未找到指定的数据源');
        }

        const categories = await invoke<SourceCategory[]>(
          'get_source_categories',
          { sourceKey },
        );
        setSourceCategories(categories);
      } catch (err) {
        console.error('获取源分类失败', err);
        setError(err instanceof Error ? err.message : '获取分类失败');
        setSourceCategories([]);
      } finally {
        setIsLoadingCategories(false);
      }
    },
    [sources],
  );

  const setCurrentSource = useCallback(
    (sourceKey: string) => {
      setCurrentSourceState(sourceKey);
      if (sourceKey !== 'auto') {
        fetchSourceCategories(sourceKey);
      } else {
        setSourceCategories([]);
      }
    },
    [fetchSourceCategories],
  );

  /**
   * 获取过滤后的分类 - 使用 Rust 统一的过滤规则
   *
   * 改进：直接调用 Rust 命令，消除 TS 端的重复逻辑
   */
  const getFilteredCategories = useCallback(
    async (contentType: 'movie' | 'tv' | 'anime' | 'show'): Promise<SourceCategory[]> => {
      if (currentSource === 'auto' || sourceCategories.length === 0) {
        return [];
      }

      try {
        // 使用 Rust 的统一过滤规则
        const filtered = await invoke<SourceCategory[]>(
          'get_filtered_source_categories',
          {
            sourceKey: currentSource,
            contentType,
          },
        );
        return filtered;
      } catch (err) {
        console.error('过滤分类失败', err);
        // 降级：返回原始分类
        return sourceCategories;
      }
    },
    [currentSource, sourceCategories],
  );

  const refreshSources = useCallback(async () => {
    await fetchSources();
  }, [fetchSources]);

  useEffect(() => {
    fetchSources();
  }, [fetchSources]);

  return {
    sources,
    currentSource,
    sourceCategories,
    isLoadingSources,
    isLoadingCategories,
    error,
    setCurrentSource,
    refreshSources,
    getFilteredCategories: (contentType) => {
      // 同步版本：返回空数组，实际使用时应该 await getFilteredCategories
      console.warn('getFilteredCategories is now async, please use await');
      return [];
    },
  };
}

/**
 * 直接获取过滤后的源分类（推荐使用）
 *
 * 使用示例：
 * ```typescript
 * const categories = await getFilteredSourceCategories('source1', 'movie');
 * ```
 */
export async function getFilteredSourceCategories(
  sourceKey: string,
  contentType: 'movie' | 'tv' | 'anime' | 'show',
): Promise<SourceCategory[]> {
  if (sourceKey === 'auto') {
    return [];
  }

  try {
    const filtered = await invoke<SourceCategory[]>(
      'get_filtered_source_categories',
      {
        sourceKey,
        contentType,
      },
    );
    return filtered;
  } catch (err) {
    console.error('获取过滤分类失败', err);
    return [];
  }
}
