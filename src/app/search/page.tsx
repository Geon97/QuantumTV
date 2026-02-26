/* eslint-disable react-hooks/exhaustive-deps, @typescript-eslint/no-explicit-any,@typescript-eslint/no-non-null-assertion,no-empty */
'use client';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ChevronUp, Search, X } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import React, { Suspense, useEffect, useMemo, useRef, useState } from 'react';

import { AggregatedGroup, SearchFilter, SearchResult } from '@/lib/types';
import { subscribeToDataUpdates } from '@/lib/utils';
import { useImagePreload } from '@/hooks/useImagePreload';

import PageLayout from '@/components/PageLayout';
import SearchResultFilter, {
  SearchFilterCategory,
} from '@/components/SearchResultFilter';
import SearchSuggestions from '@/components/SearchSuggestions';
import VideoCard, { VideoCardHandle } from '@/components/VideoCard';
const globalSearchUIState = {
  query: '',
  viewMode: 'agg' as 'agg' | 'all',
  filterAgg: {
    source: 'all',
    title: 'all',
    year: 'all',
    yearOrder: 'none' as const,
  },
  filterAll: {
    source: 'all',
    title: 'all',
    year: 'all',
    yearOrder: 'none' as const,
  },
};
// 搜索缓存
const searchCache = new Map<string, SearchResult[]>();
function SearchPageClient() {
  // 搜索历史
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  // 返回顶部按钮显示状态
  const [showBackToTop, setShowBackToTop] = useState(false);

  const router = useRouter();
  const searchParams = useSearchParams();
  // 避免渲染时的“初次加载”闪烁
  const qParam = searchParams.get('q') || '';
  const isReturning =
    qParam && qParam === globalSearchUIState.query && searchCache.has(qParam);
  const currentQueryRef = useRef<string>('');
  const [searchQuery, setSearchQuery] = useState('');
  const [normalizedQuery, setNormalizedQuery] = useState('');

  const [isLoading, setIsLoading] = useState(!isReturning && !!qParam);
  const [showResults, setShowResults] = useState(!!qParam);
  const [searchResults, setSearchResults] = useState<SearchResult[]>(
    isReturning ? searchCache.get(qParam)! : [],
  );
  const [showSuggestions, setShowSuggestions] = useState(false);
  const eventSourceRef = useRef<EventSource | null>(null);
  const [totalSources, setTotalSources] = useState(0);
  const [completedSources, setCompletedSources] = useState(0);
  const pendingResultsRef = useRef<SearchResult[]>([]);
  const flushTimerRef = useRef<number | null>(null);
  const [useFluidSearch, setUseFluidSearch] = useState(true);
  // 聚合卡片 refs
  const groupRefs = useRef<
    Map<string, React.RefObject<VideoCardHandle | null>>
  >(new Map());

  const getGroupRef = (key: string) => {
    let ref = groupRefs.current.get(key);
    if (!ref) {
      ref = React.createRef<VideoCardHandle | null>();
      groupRefs.current.set(key, ref);
    }
    return ref;
  };

  // 聚合结果状态 (from Rust)
  const [aggregatedGroups, setAggregatedGroups] = useState<
    Map<string, AggregatedGroup>
  >(new Map());

  // 过滤器：非聚合与聚合
  const [filterAll, setFilterAll] = useState(
    isReturning
      ? globalSearchUIState.filterAll
      : {
          source: 'all',
          title: 'all',
          year: 'all',
          yearOrder: 'none' as const,
        },
  );
  const [filterAgg, setFilterAgg] = useState(
    isReturning
      ? globalSearchUIState.filterAgg
      : {
          source: 'all',
          title: 'all',
          year: 'all',
          yearOrder: 'none' as const,
        },
  );

  const [viewMode, setViewMode] = useState<'agg' | 'all'>(
    isReturning ? globalSearchUIState.viewMode : 'agg',
  );

  // 当搜索结果变化时，调用 Rust 进行聚合
  useEffect(() => {
    if (searchResults.length === 0) {
      setAggregatedGroups(new Map());
      return;
    }

    const query = currentQueryRef.current;
    const filter: SearchFilter = {
      source: filterAgg.source,
      title: filterAgg.title,
      year: filterAgg.year,
      year_order: filterAgg.yearOrder as 'none' | 'asc' | 'desc',
    };

    invoke<Array<[string, AggregatedGroup]>>(
      'aggregate_search_results_filtered_command',
      {
        results: searchResults,
        query,
        normalizedQuery: normalizedQuery || null,
        filter,
      },
    )
      .then((aggregatedEntries) => {
        setAggregatedGroups(new Map(aggregatedEntries));

        // 输出缓存统计信息
        invoke<Record<string, { entry_count: number; weighted_size: number }>>(
          'get_cache_stats',
        )
          .then((stats) => {
            console.log(
              '📊 缓存统计 | 视频缓存:',
              stats.video.entry_count,
              '条 | 搜索缓存:',
              stats.search.entry_count,
              '条',
            );
          })
          .catch(console.error);
      })
      .catch(console.error);
  }, [searchResults, normalizedQuery, filterAgg]);

  // 构建筛选选项
  const filterOptions = useMemo(() => {
    const sourcesSet = new Map<string, string>();
    const titlesSet = new Set<string>();
    const yearsSet = new Set<string>();

    searchResults.forEach((item) => {
      if (item.source && item.source_name) {
        sourcesSet.set(item.source, item.source_name);
      }
      if (item.title) titlesSet.add(item.title);
      if (item.year) yearsSet.add(item.year);
    });

    const sourceOptions: { label: string; value: string }[] = [
      { label: '全部来源', value: 'all' },
      ...Array.from(sourcesSet.entries())
        .sort((a, b) => a[1].localeCompare(b[1]))
        .map(([value, label]) => ({ label, value })),
    ];

    const titleOptions: { label: string; value: string }[] = [
      { label: '全部标题', value: 'all' },
      ...Array.from(titlesSet.values())
        .sort((a, b) => a.localeCompare(b))
        .map((t) => ({ label: t, value: t })),
    ];

    // 年份: 将 unknown 放末尾
    const years = Array.from(yearsSet.values());
    const knownYears = years
      .filter((y) => y !== 'unknown')
      .sort((a, b) => parseInt(b) - parseInt(a));
    const hasUnknown = years.includes('unknown');
    const yearOptions: { label: string; value: string }[] = [
      { label: '全部年份', value: 'all' },
      ...knownYears.map((y) => ({ label: y, value: y })),
      ...(hasUnknown ? [{ label: '未知', value: 'unknown' }] : []),
    ];

    const categoriesAll: SearchFilterCategory[] = [
      { key: 'source', label: '来源', options: sourceOptions },
      { key: 'title', label: '标题', options: titleOptions },
      { key: 'year', label: '年份', options: yearOptions },
    ];

    const categoriesAgg: SearchFilterCategory[] = [
      { key: 'source', label: '来源', options: sourceOptions },
      { key: 'title', label: '标题', options: titleOptions },
      { key: 'year', label: '年份', options: yearOptions },
    ];

    return { categoriesAll, categoriesAgg };
  }, [searchResults]);

  // 非聚合：应用筛选与排序 (使用 Rust)
  const [filteredAllResults, setFilteredAllResults] = useState<SearchResult[]>(
    [],
  );

  useEffect(() => {
    if (searchResults.length === 0) {
      setFilteredAllResults([]);
      return;
    }

    const filter: SearchFilter = {
      source: filterAll.source,
      title: filterAll.title,
      year: filterAll.year,
      year_order: filterAll.yearOrder as 'none' | 'asc' | 'desc',
    };

    invoke<SearchResult[]>('filter_and_sort_results', {
      results: searchResults,
      filter,
    })
      .then(setFilteredAllResults)
      .catch(console.error);
  }, [searchResults, filterAll]);

  useEffect(() => {
    // 无搜索参数时聚焦搜索框
    !searchParams.get('q') && document.getElementById('searchInput')?.focus();

    // 初始加载搜索历史
    invoke<string[]>('get_search_history')
      .then(setSearchHistory)
      .catch(console.error);

    // 读取流式搜索设置 - 从用户偏好配置读取
    const loadFluidSearchSetting = async () => {
      try {
        const prefs = await invoke<{ fluid_search: boolean }>(
          'get_user_preferences',
        );
        setUseFluidSearch(prefs.fluid_search);
      } catch (error) {
        console.error('读取流式搜索设置失败:', error);
        // 降级到默认值
        setUseFluidSearch(true);
      }
    };

    loadFluidSearchSetting();

    // 监听搜索历史更新事件
    const unsubscribe = subscribeToDataUpdates(
      'searchHistoryUpdated',
      async () => {
        // 重新获取搜索历史
        try {
          const history = await invoke<string[]>('get_search_history');
          setSearchHistory(history);
        } catch (err) {
          console.error('获取搜索历史失败:', err);
        }
      },
    );

    // 获取滚动位置的函数 - 专门针对 body 滚动
    const getScrollTop = () => {
      return document.body.scrollTop || 0;
    };

    // 使用 requestAnimationFrame 持续检测滚动位置
    let isRunning = false;
    const checkScrollPosition = () => {
      if (!isRunning) return;

      const scrollTop = getScrollTop();
      const shouldShow = scrollTop > 300;
      setShowBackToTop(shouldShow);

      requestAnimationFrame(checkScrollPosition);
    };

    // 启动持续检测
    isRunning = true;
    checkScrollPosition();

    // 监听 body 元素的滚动事件
    const handleScroll = () => {
      const scrollTop = getScrollTop();
      setShowBackToTop(scrollTop > 300);
    };

    document.body.addEventListener('scroll', handleScroll, { passive: true });

    return () => {
      unsubscribe();
      isRunning = false; // 停止 requestAnimationFrame 循环

      // 移除 body 滚动事件监听器
      document.body.removeEventListener('scroll', handleScroll);
    };
  }, []);
  // 图片预加载：提取前 30 张搜索结果的图片
  const searchImageUrls = useMemo(() => {
    return searchResults
      .slice(0, 30)
      .map((item) => item.poster)
      .filter(Boolean);
  }, [searchResults]);
  useImagePreload(searchImageUrls, !isLoading && searchResults.length > 0);
  useEffect(() => {
    if (!qParam) {
      setShowResults(false);
      return;
    }

    // ✅ 返回搜索页：直接恢复，不做任何事
    if (qParam === globalSearchUIState.query && searchCache.has(qParam)) {
      setSearchResults(searchCache.get(qParam)!);
      setIsLoading(false);
      setShowResults(true);
      return;
    }

    setIsLoading(true);
    setShowResults(true);

    // 清空之前的进度
    setTotalSources(0);
    setCompletedSources(0);
    pendingResultsRef.current = [];

    // 如果启用流式搜索，监听事件
    if (useFluidSearch) {
      let unlistenStream: (() => void) | null = null;
      let unlistenCompleted: (() => void) | null = null;

      const setupListeners = async () => {
        try {
          // 监听流式结果事件
          unlistenStream = await listen<any>(
            'search-stream-result',
            (event) => {
              const { results, total_sources, completed_sources } =
                event.payload;

              setTotalSources(total_sources);
              setCompletedSources(completed_sources);

              // 实时添加结果（去重）
              if (results && results.length > 0) {
                setSearchResults((prevResults) => {
                  const existingIds = new Set(
                    prevResults.map((r) => `${r.source}|${r.id}`),
                  );
                  const filteredNew = results.filter(
                    (r: SearchResult) =>
                      !existingIds.has(`${r.source}|${r.id}`),
                  );
                  return [...prevResults, ...filteredNew];
                });
              }
            },
          );

          // 监听搜索完成事件
          unlistenCompleted = await listen<any>(
            'search-stream-completed',
            (event) => {
              setIsLoading(false);
            },
          );
        } catch (err) {
          console.error('Failed to setup stream listeners:', err);
        }
      };

      // 设置监听器
      setupListeners();

      // 调用搜索命令
      invoke<SearchResult[]>('search', { query: qParam })
        .then((results) => {
          const safeResults = results || [];
          searchCache.set(qParam, safeResults);
          setSearchResults(safeResults);
        })
        .catch((err) => {
          console.error('Search error:', err);
          setIsLoading(false);
        })
        .finally(() => {
          // 清理监听器
          if (unlistenStream) unlistenStream();
          if (unlistenCompleted) unlistenCompleted();
        });
    } else {
      // 非流式搜索：原有逻辑
      invoke<SearchResult[]>('search', { query: qParam })
        .then((results) => {
          const safeResults = results || [];
          searchCache.set(qParam, safeResults);
          setSearchResults(safeResults);
        })
        .catch(console.error)
        .finally(() => {
          setIsLoading(false);
        });
    }
  }, [qParam, useFluidSearch]);
  useEffect(() => {
    globalSearchUIState.query = qParam;
    globalSearchUIState.viewMode = viewMode;
    globalSearchUIState.filterAgg = filterAgg;
    globalSearchUIState.filterAll = filterAll;
  }, [qParam, viewMode, filterAgg, filterAll]);
  // 组件卸载时，关闭可能存在的连接
  useEffect(() => {
    return () => {
      if (eventSourceRef.current) {
        try {
          eventSourceRef.current.close();
        } catch {}
        eventSourceRef.current = null;
      }
      if (flushTimerRef.current) {
        clearTimeout(flushTimerRef.current);
        flushTimerRef.current = null;
      }
      pendingResultsRef.current = [];
    };
  }, []);

  // 输入框内容变化时触发，显示搜索建议
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSearchQuery(value);

    if (value.trim()) {
      setShowSuggestions(true);
    } else {
      setShowSuggestions(false);
    }
  };

  // 搜索框聚焦时触发，显示搜索建议
  const handleInputFocus = () => {
    if (searchQuery.trim()) {
      setShowSuggestions(true);
    }
  };

  // 搜索表单提交时触发，处理搜索逻辑
  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = searchQuery.trim();
    if (!trimmed) return;

    // 保存搜索历史
    try {
      await invoke('add_search_history', { keyword: trimmed });
      window.dispatchEvent(
        new CustomEvent('searchHistoryUpdated', { detail: {} }),
      );
    } catch (err) {
      console.error('Failed to save search history:', err);
    }

    router.push(`/search?q=${encodeURIComponent(trimmed)}`);
  };

  const handleSuggestionSelect = async (suggestion: string) => {
    setSearchQuery(suggestion);
    setShowSuggestions(false);

    // 自动执行搜索
    setIsLoading(true);
    setShowResults(true);

    // 保存搜索历史
    try {
      await invoke('add_search_history', { keyword: suggestion });
      window.dispatchEvent(
        new CustomEvent('searchHistoryUpdated', { detail: {} }),
      );
    } catch (err) {
      console.error('Failed to save search history:', err);
    }

    router.push(`/search?q=${encodeURIComponent(suggestion)}`);
    // 其余由 searchParams 变化的 effect 处理
  };

  // 返回顶部功能
  const scrollToTop = () => {
    try {
      // 根据调试结果，真正的滚动容器是 document.body
      document.body.scrollTo({
        top: 0,
        behavior: 'smooth',
      });
    } catch {
      // 如果平滑滚动完全失败，使用立即滚动
      document.body.scrollTop = 0;
    }
  };

  return (
    <PageLayout activePath='/search'>
      <div className='px-4 sm:px-10 py-4 sm:py-8 overflow-visible mb-10'>
        {/* 搜索框 */}
        <div className='mb-8'>
          <form onSubmit={handleSearch} className='max-w-2xl mx-auto'>
            <div className='relative'>
              <Search className='absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400 dark:text-gray-500' />
              <input
                id='searchInput'
                type='text'
                value={searchQuery}
                onChange={handleInputChange}
                onFocus={handleInputFocus}
                placeholder='搜索电影、电视剧...'
                autoComplete='off'
                className='w-full h-12 rounded-lg bg-gray-50/80 py-3 pl-10 pr-12 text-sm text-gray-700 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-400 focus:bg-white border border-gray-200/50 shadow-sm dark:bg-gray-800 dark:text-gray-300 dark:placeholder-gray-500 dark:focus:bg-gray-700 dark:border-gray-700'
              />

              {/* 清除按钮 */}
              {searchQuery && (
                <button
                  type='button'
                  onClick={() => {
                    setSearchQuery('');
                    setShowSuggestions(false);
                    document.getElementById('searchInput')?.focus();
                  }}
                  className='absolute right-3 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors dark:text-gray-500 dark:hover:text-gray-300'
                  aria-label='清除搜索内容'
                >
                  <X className='h-5 w-5' />
                </button>
              )}

              {/* 搜索建议 */}
              <SearchSuggestions
                query={searchQuery}
                isVisible={showSuggestions}
                onSelect={handleSuggestionSelect}
                onClose={() => setShowSuggestions(false)}
                onEnterKey={async () => {
                  // 当用户按回车键时，使用搜索框的实际内容进行搜索
                  const trimmed = searchQuery.trim().replace(/\s+/g, ' ');
                  if (!trimmed) return;

                  // 回显搜索框
                  setSearchQuery(trimmed);
                  setIsLoading(true);
                  setShowResults(true);
                  setShowSuggestions(false);

                  // 保存搜索历史
                  try {
                    await invoke('add_search_history', { keyword: trimmed });
                    window.dispatchEvent(
                      new CustomEvent('searchHistoryUpdated', { detail: {} }),
                    );
                  } catch (err) {
                    console.error('Failed to save search history:', err);
                  }

                  router.push(`/search?q=${encodeURIComponent(trimmed)}`);
                }}
              />
            </div>
          </form>
        </div>

        {/* 搜索结果或搜索历史 */}
        <div className='max-w-[95%] mx-auto mt-12 overflow-visible'>
          {showResults ? (
            <section className='mb-12'>
              {/* 标题 */}
              <div className='mb-4'>
                <h2 className='text-xl font-bold text-gray-800 dark:text-gray-200'>
                  搜索结果
                  {totalSources > 0 && useFluidSearch && (
                    <span className='ml-2 text-sm font-normal text-gray-500 dark:text-gray-400'>
                      {completedSources}/{totalSources}
                    </span>
                  )}
                  {isLoading && useFluidSearch && (
                    <span className='ml-2 inline-block align-middle'>
                      <span className='inline-block h-3 w-3 border-2 border-gray-300 border-t-green-500 rounded-full animate-spin'></span>
                    </span>
                  )}
                </h2>
              </div>
              {/* 筛选器 + 聚合开关 同行 */}
              <div className='mb-8 flex items-center justify-between gap-3'>
                <div className='flex-1 min-w-0'>
                  {viewMode === 'agg' ? (
                    <SearchResultFilter
                      categories={filterOptions.categoriesAgg}
                      values={filterAgg}
                      onChange={(v) => setFilterAgg(v as any)}
                    />
                  ) : (
                    <SearchResultFilter
                      categories={filterOptions.categoriesAll}
                      values={filterAll}
                      onChange={(v) => setFilterAll(v as any)}
                    />
                  )}
                </div>
                {/* 聚合开关 */}
                <label className='flex items-center gap-2 cursor-pointer select-none shrink-0'>
                  <span className='text-xs sm:text-sm text-gray-700 dark:text-gray-300'>
                    聚合
                  </span>
                  <div className='relative'>
                    <input
                      type='checkbox'
                      className='sr-only peer'
                      checked={viewMode === 'agg'}
                      onChange={() =>
                        setViewMode(viewMode === 'agg' ? 'all' : 'agg')
                      }
                    />
                    <div className='w-9 h-5 bg-gray-300 rounded-full peer-checked:bg-green-500 transition-colors dark:bg-gray-600'></div>
                    <div className='absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform peer-checked:translate-x-4'></div>
                  </div>
                </label>
              </div>
              {searchResults.length === 0 ? (
                isLoading ? (
                  <div className='flex justify-center items-center h-40'>
                    <div className='animate-spin rounded-full h-8 w-8 border-b-2 border-green-500'></div>
                  </div>
                ) : (
                  <div className='text-center text-gray-500 py-8 dark:text-gray-400'>
                    未找到相关结果
                  </div>
                )
              ) : (
                <div
                  key={`search-results-${viewMode}`}
                  className='justify-start grid grid-cols-3 gap-x-2 gap-y-14 sm:gap-y-20 px-0 sm:px-2 sm:grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] sm:gap-x-8'
                >
                  {viewMode === 'agg'
                    ? Array.from(aggregatedGroups.entries()).map(
                        ([mapKey, group]) => {
                          const rep = group.representative;
                          const title = rep.title || '';
                          const poster = rep.poster || '';
                          const year = rep.year || 'unknown';
                          const episodes = group.episodes;
                          const source_names = group.source_names;
                          const douban_id = group.douban_id;
                          const type = episodes === 1 ? 'movie' : 'tv';

                          return (
                            <div key={`agg-${mapKey}`} className='w-full'>
                              <VideoCard
                                ref={getGroupRef(mapKey)}
                                from='search'
                                isAggregate={true}
                                title={title}
                                poster={poster}
                                year={year}
                                episodes={episodes}
                                source_names={source_names}
                                douban_id={douban_id}
                                query={
                                  searchQuery.trim() !== title
                                    ? searchQuery.trim()
                                    : ''
                                }
                                type={type}
                              />
                            </div>
                          );
                        },
                      )
                    : filteredAllResults.map((item) => (
                        <div
                          key={`all-${item.source}-${item.id}`}
                          className='w-full'
                        >
                          <VideoCard
                            id={item.id}
                            title={item.title}
                            poster={item.poster}
                            episodes={item.episodes.length}
                            source={item.source}
                            source_name={item.source_name}
                            douban_id={item.douban_id}
                            query={
                              searchQuery.trim() !== item.title
                                ? searchQuery.trim()
                                : ''
                            }
                            year={item.year}
                            from='search'
                            type={item.episodes.length > 1 ? 'tv' : 'movie'}
                          />
                        </div>
                      ))}
                </div>
              )}
            </section>
          ) : searchHistory.length > 0 ? (
            // 搜索历史
            <section className='mb-12'>
              <h2 className='mb-4 text-xl font-bold text-gray-800 text-left dark:text-gray-200'>
                搜索历史
                {searchHistory.length > 0 && (
                  <button
                    onClick={async () => {
                      await invoke('clear_search_history');
                      setSearchHistory([]);
                      window.dispatchEvent(
                        new CustomEvent('searchHistoryUpdated', { detail: {} }),
                      );
                    }}
                    className='ml-3 text-sm text-gray-500 hover:text-red-500 transition-colors dark:text-gray-400 dark:hover:text-red-500'
                  >
                    清空
                  </button>
                )}
              </h2>
              <div className='flex flex-wrap gap-2'>
                {searchHistory.map((item) => (
                  <div key={item} className='relative group'>
                    <button
                      onClick={async () => {
                        setSearchQuery(item);

                        // 更新搜索历史时间戳
                        try {
                          await invoke('add_search_history', {
                            keyword: item.trim(),
                          });
                          window.dispatchEvent(
                            new CustomEvent('searchHistoryUpdated', {
                              detail: {},
                            }),
                          );
                        } catch (err) {
                          console.error(
                            'Failed to update search history:',
                            err,
                          );
                        }

                        router.push(
                          `/search?q=${encodeURIComponent(item.trim())}`,
                        );
                      }}
                      className='px-4 py-2 bg-gray-500/10 hover:bg-gray-300 rounded-full text-sm text-gray-700 transition-colors duration-200 dark:bg-gray-700/50 dark:hover:bg-gray-600 dark:text-gray-300'
                    >
                      {item}
                    </button>
                    {/* 删除按钮 */}
                    <button
                      aria-label='删除搜索历史'
                      onClick={async (e) => {
                        e.stopPropagation();
                        e.preventDefault();
                        await invoke('delete_search_history', {
                          keyword: item,
                        });
                        setSearchHistory((prev) =>
                          prev.filter((h) => h !== item),
                        );
                        window.dispatchEvent(
                          new CustomEvent('searchHistoryUpdated', {
                            detail: {},
                          }),
                        );
                      }}
                      className='absolute -top-1 -right-1 w-4 h-4 opacity-0 group-hover:opacity-100 bg-gray-400 hover:bg-red-500 text-white rounded-full flex items-center justify-center text-[10px] transition-colors'
                    >
                      <X className='w-3 h-3' />
                    </button>
                  </div>
                ))}
              </div>
            </section>
          ) : null}
        </div>
      </div>

      {/* 返回顶部悬浮按钮 */}
      <button
        onClick={scrollToTop}
        className={`fixed bottom-20 md:bottom-6 right-6 z-500 w-12 h-12 bg-green-500/90 hover:bg-green-500 text-white rounded-full shadow-lg backdrop-blur-sm transition-all duration-300 ease-in-out flex items-center justify-center group ${
          showBackToTop
            ? 'opacity-100 translate-y-0 pointer-events-auto'
            : 'opacity-0 translate-y-4 pointer-events-none'
        }`}
        aria-label='返回顶部'
      >
        <ChevronUp className='w-6 h-6 transition-transform group-hover:scale-110' />
      </button>
    </PageLayout>
  );
}

export default function SearchPage() {
  return (
    <Suspense>
      <SearchPageClient />
    </Suspense>
  );
}
