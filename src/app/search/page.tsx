/* eslint-disable react-hooks/exhaustive-deps, @typescript-eslint/no-explicit-any,@typescript-eslint/no-non-null-assertion,no-empty */
'use client';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ChevronUp, Search, X } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import React, { Suspense, useEffect, useMemo, useRef, useState } from 'react';

import {
  AggregatedGroup,
  SearchFilter,
  SearchPageBootstrap,
  SearchResult,
} from '@/lib/types';
import { appLayoutClasses, getGridColumnsClass } from '@/lib/ui-layout';
import { subscribeToDataUpdates } from '@/lib/utils';
import { useImagePreload } from '@/hooks/useImagePreload';

import PageLayout from '@/components/PageLayout';
import SearchResultFilter, {
  SearchFilterCategory,
} from '@/components/SearchResultFilter';
import SearchSuggestions from '@/components/SearchSuggestions';
import VideoCard, { VideoCardHandle } from '@/components/VideoCard';

type SearchPageQueryResponse = {
  results: SearchResult[];
  cacheHit: boolean;
  filterCategoriesAll: SearchFilterCategory[];
  filterCategoriesAgg: SearchFilterCategory[];
};

function SearchPageClient() {
  // 搜索历史
  const [searchHistory, setSearchHistory] = useState<string[]>([]);
  // 返回顶部按钮显示状态
  const [showBackToTop, setShowBackToTop] = useState(false);

  const router = useRouter();
  const searchParams = useSearchParams();
  // 避免渲染时的“初次加载”闪烁
  const qParam = searchParams.get('q') || '';
  const currentQueryRef = useRef<string>('');
  const [searchQuery, setSearchQuery] = useState('');

  const [isLoading, setIsLoading] = useState(!!qParam);
  const [showResults, setShowResults] = useState(!!qParam);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [totalSources, setTotalSources] = useState(0);
  const [completedSources, setCompletedSources] = useState(0);
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
  const [filterAll, setFilterAll] = useState({
    source: 'all',
    title: 'all',
    year: 'all',
    yearOrder: 'none' as const,
  });
  const [filterAgg, setFilterAgg] = useState({
    source: 'all',
    title: 'all',
    year: 'all',
    yearOrder: 'none' as const,
  });

  const [viewMode, setViewMode] = useState<'agg' | 'all'>('agg');

  const [filterOptions, setFilterOptions] = useState<{
    categoriesAll: SearchFilterCategory[];
    categoriesAgg: SearchFilterCategory[];
  }>({
    categoriesAll: [],
    categoriesAgg: [],
  });

  // 聚合与过滤由 Rust 统一处理
  const [filteredAllResults, setFilteredAllResults] = useState<SearchResult[]>(
    [],
  );

  useEffect(() => {
    const query = currentQueryRef.current;
    const filterAggPayload: SearchFilter = {
      source: filterAgg.source,
      title: filterAgg.title,
      year: filterAgg.year,
      year_order: filterAgg.yearOrder as 'none' | 'asc' | 'desc',
    };
    const filterAllPayload: SearchFilter = {
      source: filterAll.source,
      title: filterAll.title,
      year: filterAll.year,
      year_order: filterAll.yearOrder as 'none' | 'asc' | 'desc',
    };

    invoke<{
      aggregatedEntries: Array<[string, AggregatedGroup]>;
      filteredResults: SearchResult[];
      filterCategoriesAll: SearchFilterCategory[];
      filterCategoriesAgg: SearchFilterCategory[];
    }>('build_search_page_state', {
      results: searchResults,
      query,
      normalizedQuery: null,
      filterAgg: filterAggPayload,
      filterAll: filterAllPayload,
    })
      .then((response) => {
        setAggregatedGroups(new Map(response.aggregatedEntries));
        setFilteredAllResults(response.filteredResults);
        setFilterOptions({
          categoriesAll: response.filterCategoriesAll,
          categoriesAgg: response.filterCategoriesAgg,
        });
      })
      .catch(console.error);
  }, [searchResults, filterAgg, filterAll]);

  useEffect(() => {
    // 无搜索参数时聚焦搜索框
    !searchParams.get('q') && document.getElementById('searchInput')?.focus();

    // 初始加载搜索历史 + 流式搜索设置
    invoke<SearchPageBootstrap>('get_search_page_bootstrap')
      .then((bootstrap) => {
        setSearchHistory(bootstrap.search_history);
        setUseFluidSearch(bootstrap.fluid_search);
      })
      .catch((error) => {
        console.error('加载搜索初始化数据失败:', error);
        setUseFluidSearch(true);
      });

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

    setIsLoading(true);
    setShowResults(true);

    // 清空之前的进度
    setTotalSources(0);
    setCompletedSources(0);
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
      invoke<SearchPageQueryResponse>('search_page_query', { query: qParam })
        .then((response) => {
          const safeResults = response?.results || [];
          const cacheHit = response?.cacheHit ?? false;
          setSearchResults(safeResults);
          setFilterOptions({
            categoriesAll: response?.filterCategoriesAll || [],
            categoriesAgg: response?.filterCategoriesAgg || [],
          });
          if (cacheHit) {
            setIsLoading(false);
          }
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
      invoke<SearchPageQueryResponse>('search_page_query', { query: qParam })
        .then((response) => {
          const safeResults = response?.results || [];
          setSearchResults(safeResults);
          setFilterOptions({
            categoriesAll: response?.filterCategoriesAll || [],
            categoriesAgg: response?.filterCategoriesAgg || [],
          });
        })
        .catch(console.error)
        .finally(() => {
          setIsLoading(false);
        });
    }
  }, [qParam, useFluidSearch]);
  // 组件卸载时，关闭可能存在的连接
  
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
      <div
        className={`${appLayoutClasses.pageShell} py-4 max-[375px]:py-3.5 min-[834px]:py-7 min-[1440px]:py-9 overflow-visible mb-8 max-[375px]:mb-7 min-[834px]:mb-10`}
      >
        {/* 搜索框 */}
        <div className='mb-8 max-[375px]:mb-6 min-[834px]:mb-9'>
          <form
            onSubmit={handleSearch}
            className='mx-auto max-w-2xl min-[834px]:max-w-3xl min-[1440px]:max-w-4xl'
          >
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
                className='w-full h-12 max-[375px]:h-11 min-[834px]:h-13 min-[1440px]:h-14 rounded-xl bg-gray-50/80 py-3 pl-10 pr-12 text-sm max-[375px]:text-[0.82rem] min-[834px]:text-base text-gray-700 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-green-400 focus:bg-white border border-gray-200/50 shadow-sm dark:bg-gray-800 dark:text-gray-300 dark:placeholder-gray-500 dark:focus:bg-gray-700 dark:border-gray-700'
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
        <div className={`${appLayoutClasses.pageContent} mt-10 sm:mt-12 min-[834px]:mt-14 overflow-visible`}>
          {showResults ? (
            <section className='mb-12'>
              {/* 标题 */}
              <div className='mb-4'>
                <h2 className='text-lg font-bold text-gray-800 max-[375px]:text-base min-[834px]:text-[1.35rem] min-[1440px]:text-[1.5rem] dark:text-gray-200'>
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
              <div className='mb-8 flex items-center justify-between gap-3 max-[375px]:flex-col max-[375px]:items-start min-[834px]:mb-9'>
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
                  <span className='text-xs min-[834px]:text-sm text-gray-700 dark:text-gray-300'>
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
                <div key={`search-results-${viewMode}`} className={getGridColumnsClass('dense')}>
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
              <h2 className='mb-4 text-lg font-bold text-gray-800 text-left max-[375px]:text-base min-[834px]:text-[1.35rem] min-[1440px]:text-[1.5rem] dark:text-gray-200'>
                搜索历史
                {searchHistory.length > 0 && (
                  <button
                    onClick={async () => {
                      await invoke('clear_search_history');
                      setSearchHistory([]);
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
                      className='tap-target px-4 py-2 bg-gray-500/10 hover:bg-gray-300 rounded-full text-sm text-gray-700 transition-colors duration-200 dark:bg-gray-700/50 dark:hover:bg-gray-600 dark:text-gray-300'
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
        className={`tap-target fixed bottom-20 right-4 z-[70] h-12 w-12 max-[375px]:h-11 max-[375px]:w-11 min-[834px]:h-13 min-[834px]:w-13 rounded-full bg-green-500/90 text-white shadow-lg backdrop-blur-sm transition-all duration-300 ease-in-out hover:bg-green-500 sm:right-6 lg:bottom-6 flex items-center justify-center group ${
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
