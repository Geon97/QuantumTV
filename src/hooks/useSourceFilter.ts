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

const CONTENT_TYPE_KEYWORDS: Record<string, string[]> = {
  movie: ['电影', '影片', '院线', '4k', '蓝光'],
  tv: ['电视剧', '剧集', '美剧', '韩剧', '日剧', '港剧'],
  anime: ['动漫', '动画', '番剧', '漫画'],
  show: ['综艺', '真人秀', '脱口秀', '晚会', '纪录片'],
};

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

  const getFilteredCategories = useCallback(
    (contentType: 'movie' | 'tv' | 'anime' | 'show'): SourceCategory[] => {
      if (sourceCategories.length === 0) {
        return [];
      }

      const keywords = CONTENT_TYPE_KEYWORDS[contentType] || [];
      let filtered = sourceCategories.filter((cat) => {
        const name = cat.type_name.toLowerCase();
        return keywords.some((keyword) => name.includes(keyword.toLowerCase()));
      });

      if (filtered.length === 0) {
        filtered = sourceCategories.filter((cat) => {
          const name = cat.type_name;
          return (
            name.includes('影') || name.includes('剧') || name.includes('漫')
          );
        });
      }

      if (filtered.length === 0) {
        return sourceCategories.slice(0, 15);
      }

      return filtered;
    },
    [sourceCategories],
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
    getFilteredCategories,
  };
}

export default useSourceFilter;
