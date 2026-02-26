/* eslint-disable no-console,react-hooks/exhaustive-deps,@typescript-eslint/no-explicit-any */

'use client';
import { invoke } from '@tauri-apps/api/core';
import { useSearchParams } from 'next/navigation';
import { Suspense, useCallback, useEffect, useRef, useState } from 'react';

import type {
  DoubanDefaultsResponse,
  DoubanItem,
  DoubanPageResponse,
} from '@/lib/types';
import { useImagePreload } from '@/hooks/useImagePreload';
import { useCachedData } from '@/hooks/usePageCache';
import { useSourceFilter } from '@/hooks/useSourceFilter';

import DoubanCardSkeleton from '@/components/DoubanCardSkeleton';
import DoubanCustomSelector from '@/components/DoubanCustomSelector';
import DoubanSelector, { SourceCategory } from '@/components/DoubanSelector';
import PageLayout from '@/components/PageLayout';
import VideoCard from '@/components/VideoCard';

interface FetchUrlResult {
  status: number;
  body: string;
}

function DoubanPageClient() {
  const searchParams = useSearchParams();
  const [doubanData, setDoubanData] = useState<DoubanItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [selectorsReady, setSelectorsReady] = useState(false);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const loadingRef = useRef<HTMLDivElement>(null);
  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // 用于存储最新参数值的 refs
  const currentParamsRef = useRef({
    type: '',
    primarySelection: '',
    secondarySelection: '',
    multiLevelSelection: {} as Record<string, string>,
    selectedWeekday: '',
    currentPage: 0,
  });

  const type = searchParams.get('type') || 'movie';

  // 获取 runtimeConfig 中的自定义分类数据
  const [customCategories, setCustomCategories] = useState<
    Array<{ name: string; type: 'movie' | 'tv'; query: string }>
  >([]);

  // 选择器状态 - 完全独立，不依赖URL参数
  const [primarySelection, setPrimarySelection] = useState<string>(() => {
    if (type === 'movie') return '热门';
    if (type === 'tv' || type === 'show') return '最近热门';
    if (type === 'anime') return '每日放送';
    return '';
  });
  const [secondarySelection, setSecondarySelection] = useState<string>(() => {
    if (type === 'movie') return '全部';
    if (type === 'tv') return 'tv';
    if (type === 'show') return 'show';
    return '全部';
  });

  // MultiLevelSelector 状态
  const [multiLevelValues, setMultiLevelValues] = useState<
    Record<string, string>
  >({
    type: 'all',
    region: 'all',
    year: 'all',
    platform: 'all',
    label: 'all',
    sort: 'T',
  });

  // 星期选择器状态
  const [selectedWeekday, setSelectedWeekday] = useState<string>('');
  const [doubanDefaults, setDoubanDefaults] =
    useState<DoubanDefaultsResponse | null>(null);

  // 数据源筛选 Hook
  const {
    sources,
    currentSource,
    isLoadingSources,
    isLoadingCategories,
    setCurrentSource,
    getFilteredCategories,
  } = useSourceFilter();

  // 【核心修复】存储当前源的过滤后分类列表（用于渲染）
  const [filteredSourceCategories, setFilteredSourceCategories] = useState<
    SourceCategory[]
  >([]);

  // 图片预加载：提取前 30 张图片
  const imageUrls = doubanData
    .slice(0, 30)
    .map((item) => item.poster)
    .filter(Boolean);
  useImagePreload(imageUrls, !loading && doubanData.length > 0);

  // 判断是否为默认状态（可以使用缓存）
  const isDefaultState = useCallback(() => {
    if (!doubanDefaults || !doubanDefaults.cacheEnabled) {
      return false;
    }
    if (primarySelection !== doubanDefaults.primarySelection) {
      return false;
    }
    if (
      doubanDefaults.requireSecondary &&
      secondarySelection !== doubanDefaults.secondarySelection
    ) {
      return false;
    }
    return true;
  }, [doubanDefaults, primarySelection, secondarySelection]);

  // 缓存数据获取函数（仅用于默认状态）
  const buildDoubanRequest = useCallback(
    (page: number) => ({
      type,
      primarySelection,
      secondarySelection,
      multiLevelSelection: multiLevelValues,
      selectedWeekday,
      page,
      pageLimit: 25,
    }),
    [
      type,
      primarySelection,
      secondarySelection,
      multiLevelValues,
      selectedWeekday,
    ],
  );

  const fetchDefaultDoubanData =
    useCallback(async (): Promise<DoubanPageResponse> => {
      return await invoke<DoubanPageResponse>('get_douban_page_data', {
        request: buildDoubanRequest(0),
      });
    }, [buildDoubanRequest]);

  // 使用缓存（仅在默认状态时启用）
  const { fetchData: fetchCachedData } = useCachedData<DoubanPageResponse>(
    `douban_${type}`,
    fetchDefaultDoubanData,
    {
      enabled: isDefaultState(),
      staleWhileRevalidate: true,
      onUpdate: (freshData) => {
        // 后台更新完成后，如果仍在默认状态，静默更新数据
        if (isDefaultState()) {
          setDoubanData(freshData.list);
          setHasMore(freshData.has_more);
        }
      },
    },
  );

  // 选中的源分类
  const [selectedSourceCategory, setSelectedSourceCategory] =
    useState<SourceCategory | null>(null);

  // 源分类数据（用于直接查询源接口）
  const [sourceData, setSourceData] = useState<DoubanItem[]>([]);
  const [isLoadingSourceData, setIsLoadingSourceData] = useState(false);

  // 获取自定义分类数据
  useEffect(() => {
    const runtimeConfig = (window as any).RUNTIME_CONFIG;
    if (runtimeConfig?.CUSTOM_CATEGORIES?.length > 0) {
      setCustomCategories(runtimeConfig.CUSTOM_CATEGORIES);
    }
  }, []);

  // 同步最新参数值到 ref
  useEffect(() => {
    currentParamsRef.current = {
      type,
      primarySelection,
      secondarySelection,
      multiLevelSelection: multiLevelValues,
      selectedWeekday,
      currentPage,
    };
  }, [
    type,
    primarySelection,
    secondarySelection,
    multiLevelValues,
    selectedWeekday,
    currentPage,
  ]);

  // 初始化时标记选择器为准备好状态
  useEffect(() => {
    // 短暂延迟确保初始状态设置完成
    const timer = setTimeout(() => {
      setSelectorsReady(true);
    }, 50);

    return () => clearTimeout(timer);
  }, []); // 只在组件挂载时执行一次

  // type变化时立即重置
  useEffect(() => {
    let timer: NodeJS.Timeout | null = null;

    const loadDefaults = async () => {
      try {
        const defaults = await invoke<DoubanDefaultsResponse>(
          'get_douban_defaults',
          {
            request: {
              type,
              customCategories,
              fallbackSecondary: secondarySelection,
            },
          },
        );

        setDoubanDefaults(defaults);
        setPrimarySelection(defaults.primarySelection);
        setSecondarySelection(defaults.secondarySelection);
        setMultiLevelValues(defaults.multiLevelSelection);
      } catch (error) {
        console.error('获取默认值失败:', error);
      } finally {
        timer = setTimeout(() => {
          setSelectorsReady(true);
        }, 50);
      }
    };

    loadDefaults();

    return () => {
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [type, customCategories]);

  // 生成骨架屏数据
  const skeletonData = Array.from({ length: 25 }, (_, index) => index);

  // 参数快照比较函数
  const isSnapshotEqual = useCallback(
    (
      snapshot1: {
        type: string;
        primarySelection: string;
        secondarySelection: string;
        multiLevelSelection: Record<string, string>;
        selectedWeekday: string;
        currentPage: number;
      },
      snapshot2: {
        type: string;
        primarySelection: string;
        secondarySelection: string;
        multiLevelSelection: Record<string, string>;
        selectedWeekday: string;
        currentPage: number;
      },
    ) => {
      return (
        snapshot1.type === snapshot2.type &&
        snapshot1.primarySelection === snapshot2.primarySelection &&
        snapshot1.secondarySelection === snapshot2.secondarySelection &&
        snapshot1.selectedWeekday === snapshot2.selectedWeekday &&
        snapshot1.currentPage === snapshot2.currentPage &&
        JSON.stringify(snapshot1.multiLevelSelection) ===
          JSON.stringify(snapshot2.multiLevelSelection)
      );
    },
    [],
  );

  // 生成API请求参数的辅助函数

  // 防抖的数据加载函数
  const loadInitialData = useCallback(async () => {
    // 创建当前参数的快照
    const requestSnapshot = {
      type,
      primarySelection,
      secondarySelection,
      multiLevelSelection: multiLevelValues,
      selectedWeekday,
      currentPage: 0,
    };

    try {
      setLoading(true);
      // 确保在加载初始数据时重置页面状态
      setDoubanData([]);
      setCurrentPage(0);
      setHasMore(true);
      setIsLoadingMore(false);

      let data: DoubanPageResponse;

      // 如果是默认状态，尝试使用缓存
      if (isDefaultState() && type !== 'custom' && type !== 'anime') {
        data = await fetchCachedData();
      } else {
        data = await invoke<DoubanPageResponse>('get_douban_page_data', {
          request: buildDoubanRequest(0),
        });
      }

      const currentSnapshot = { ...currentParamsRef.current };
      if (isSnapshotEqual(requestSnapshot, currentSnapshot)) {
        setDoubanData(data.list);
        setHasMore(data.has_more);
        setLoading(false);
      } else {
        console.log('参数不一致，不执行任何操作，避免设置过期数据');
      }
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
  }, [
    type,
    primarySelection,
    secondarySelection,
    multiLevelValues,
    selectedWeekday,
    buildDoubanRequest,
    fetchCachedData,
    isDefaultState,
  ]);

  // 只在选择器准备好后才加载数据
  useEffect(() => {
    // 只有在选择器准备好时才开始加载
    if (!selectorsReady) {
      return;
    }

    // 如果当前是特定源模式，不加载豆瓣数据
    if (currentSource !== 'auto') {
      // 特定源模式下，等待用户选择分类后再加载
      setLoading(false);
      return;
    }

    // 清除之前的防抖定时器
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }

    // 使用防抖机制加载数据，避免连续状态更新触发多次请求
    debounceTimeoutRef.current = setTimeout(() => {
      loadInitialData();
    }, 100); // 100ms 防抖延迟

    // 清理函数
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, [
    selectorsReady,
    type,
    primarySelection,
    secondarySelection,
    multiLevelValues,
    selectedWeekday,
    loadInitialData,
    currentSource, // 添加 currentSource 依赖
  ]);

  // 单独处理 currentPage 变化（加载更多）
  useEffect(() => {
    if (currentPage > 0) {
      const fetchMoreData = async () => {
        const requestSnapshot = {
          type,
          primarySelection,
          secondarySelection,
          multiLevelSelection: multiLevelValues,
          selectedWeekday,
          currentPage,
        };

        try {
          setIsLoadingMore(true);

          const data = await invoke<DoubanPageResponse>(
            'get_douban_page_data',
            {
              request: buildDoubanRequest(currentPage),
            },
          );

          const currentSnapshot = { ...currentParamsRef.current };

          if (isSnapshotEqual(requestSnapshot, currentSnapshot)) {
            setDoubanData((prev) => {
              const existingIds = new Set(prev.map((item) => item.id));
              const newItems = data.list.filter(
                (item) => !existingIds.has(item.id),
              );
              return [...prev, ...newItems];
            });

            setHasMore(data.has_more);
          } else {
            console.log('参数不一致，不执行任何操作，避免设置过期数据');
          }
        } catch (err) {
          console.error(err);
        } finally {
          setIsLoadingMore(false);
        }
      };

      fetchMoreData();
    }
  }, [
    currentPage,
    type,
    primarySelection,
    secondarySelection,
    multiLevelValues,
    selectedWeekday,
    buildDoubanRequest,
  ]);

  // 设置滚动监听
  useEffect(() => {
    // 如果没有更多数据或正在加载，则不设置监听
    if (!hasMore || isLoadingMore || loading) {
      return;
    }

    // 确保 loadingRef 存在
    if (!loadingRef.current) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !isLoadingMore) {
          setCurrentPage((prev) => prev + 1);
        }
      },
      { threshold: 0.1 },
    );

    observer.observe(loadingRef.current);
    observerRef.current = observer;

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect();
      }
    };
  }, [hasMore, isLoadingMore, loading]);

  // 处理选择器变化
  const handlePrimaryChange = useCallback(
    (value: string) => {
      // 只有当值真正改变时才设置loading状态
      if (value !== primarySelection) {
        setLoading(true);
        // 立即重置页面状态，防止基于旧状态的请求
        setCurrentPage(0);
        setDoubanData([]);
        setHasMore(true);
        setIsLoadingMore(false);

        // 清空 MultiLevelSelector 状态
        setMultiLevelValues({
          type: 'all',
          region: 'all',
          year: 'all',
          platform: 'all',
          label: 'all',
          sort: 'T',
        });

        // 如果是自定义分类模式，同时更新一级和二级选择器
        if (type === 'custom' && customCategories.length > 0) {
          const firstCategory = customCategories.find(
            (cat) => cat.type === value,
          );
          if (firstCategory) {
            // 批量更新状态，避免多次触发数据加载
            setPrimarySelection(value);
            setSecondarySelection(firstCategory.query);
          } else {
            setPrimarySelection(value);
          }
        } else {
          // 电视剧和综艺切换到"最近热门"时，重置二级分类为第一个选项
          if ((type === 'tv' || type === 'show') && value === '最近热门') {
            setPrimarySelection(value);
            if (type === 'tv') {
              setSecondarySelection('tv');
            } else if (type === 'show') {
              setSecondarySelection('show');
            }
          } else {
            setPrimarySelection(value);
          }
        }
      }
    },
    [primarySelection, type, customCategories],
  );

  const handleSecondaryChange = useCallback(
    (value: string) => {
      // 只有当值真正改变时才设置loading状态
      if (value !== secondarySelection) {
        setLoading(true);
        // 立即重置页面状态，防止基于旧状态的请求
        setCurrentPage(0);
        setDoubanData([]);
        setHasMore(true);
        setIsLoadingMore(false);
        setSecondarySelection(value);
      }
    },
    [secondarySelection],
  );

  const handleMultiLevelChange = useCallback(
    (values: Record<string, string>) => {
      // 比较两个对象是否相同，忽略顺序
      const isEqual = (
        obj1: Record<string, string>,
        obj2: Record<string, string>,
      ) => {
        const keys1 = Object.keys(obj1).sort();
        const keys2 = Object.keys(obj2).sort();

        if (keys1.length !== keys2.length) return false;

        return keys1.every((key) => obj1[key] === obj2[key]);
      };

      // 如果相同，则不设置loading状态
      if (isEqual(values, multiLevelValues)) {
        return;
      }

      setLoading(true);
      // 立即重置页面状态，防止基于旧状态的请求
      setCurrentPage(0);
      setDoubanData([]);
      setHasMore(true);
      setIsLoadingMore(false);
      setMultiLevelValues(values);
    },
    [multiLevelValues],
  );

  const handleWeekdayChange = useCallback((weekday: string) => {
    setSelectedWeekday(weekday);
  }, []);

  // 从源接口获取分类数据（必须在 handleSourceChange 之前定义）
  const fetchSourceCategoryData = useCallback(
    async (category: SourceCategory) => {
      if (currentSource === 'auto') return;

      const source = sources.find((s) => s.key === currentSource);
      if (!source) {
        setLoading(false);
        return;
      }

      setIsLoadingSourceData(true);
      try {
        // 构建视频列表 API URL
        const originalApiUrl = source.api.endsWith('/')
          ? `${source.api}?ac=videolist&t=${category.type_id}&pg=1`
          : `${source.api}/?ac=videolist&t=${category.type_id}&pg=1`;

        let data: any;

        // Tauri 环境：使用 fetch_url 命令绕过 CORS
        const result = await invoke<FetchUrlResult>('fetch_url', {
          url: originalApiUrl,
          method: 'GET',
        });
        if (result.status !== 200) {
          throw new Error('获取分类数据失败');
        }
        data = JSON.parse(result.body);
        const items = data.list || [];

        // 转换为 DoubanItem 格式
        const convertedItems: DoubanItem[] = items.map((item: any) => ({
          id: item.vod_id?.toString() || '',
          title: item.vod_name || '',
          poster: item.vod_pic || '',
          rating: 0,
          year: item.vod_year || '',
          subtitle: item.vod_remarks || '',
        }));

        setSourceData(convertedItems);
        setHasMore(items.length >= 20); // 假设每页20条
      } catch (error) {
        console.error('获取源分类数据失败:', error);
        setSourceData([]);
      } finally {
        setIsLoadingSourceData(false);
        setLoading(false);
      }
    },
    [currentSource, sources],
  );

  // 处理数据源切换 - 实现链式自动选中逻辑
  const handleSourceChange = useCallback(
    async (sourceKey: string) => {
      if (sourceKey === currentSource) return;

      // === Step 1: 立即重置所有状态，防止状态污染 ===
      setLoading(true);
      setCurrentPage(0);
      setDoubanData([]); // 清空豆瓣数据
      setSourceData([]); // 清空源数据
      setHasMore(true);
      setIsLoadingMore(false);
      setSelectedSourceCategory(null); // 清除旧分类ID，防止污染
      setFilteredSourceCategories([]); // 清空过滤后分类列表
      setIsLoadingSourceData(false);

      // === Step 2: 切换源状态 ===
      setCurrentSource(sourceKey);

      // === Step 3: 根据源类型执行不同逻辑 ===
      if (sourceKey === 'auto') {
        // 【切回聚合模式】重置为默认的豆瓣分类选择
        if (type === 'movie') {
          setPrimarySelection('热门');
          setSecondarySelection('全部');
        } else if (type === 'tv') {
          setPrimarySelection('最近热门');
          setSecondarySelection('tv');
        } else if (type === 'show') {
          setPrimarySelection('最近热门');
          setSecondarySelection('show');
        } else if (type === 'anime') {
          setPrimarySelection('每日放送');
          setSecondarySelection('全部');
        }
        // 重置多级筛选器
        setMultiLevelValues({
          type: 'all',
          region: 'all',
          year: 'all',
          platform: 'all',
          label: 'all',
          sort: 'T',
        });
        // 聚合模式下 useEffect 会自动触发 loadInitialData
      } else {
        // === 【特定源模式】获取分类并自动选中第一个 ===
        // Step 4: 等待分类列表加载完成
        const source = sources.find((s) => s.key === sourceKey);
        if (!source) {
          console.error('🔥 [Debug] Source not found:', sourceKey);
          setLoading(false);
          return;
        }

        try {
          // 构建分类 API URL
          const originalApiUrl = source.api.endsWith('/')
            ? `${source.api}?ac=class`
            : `${source.api}/?ac=class`;

          let data: any;

          // Tauri 环境：使用 fetch_url 命令绕过 CORS
          const result = await invoke<FetchUrlResult>('fetch_url', {
            url: originalApiUrl,
            method: 'GET',
          });
          if (result.status !== 200) {
            throw new Error(`获取分类列表失败: ${result.status}`);
          }
          data = JSON.parse(result.body);

          const allCategories: SourceCategory[] = data.class || [];

          // ========================================
          // 🚀 绝对直通模式 - 移除所有过滤逻辑
          // 直接使用 API 返回的原始分类，不做任何过滤
          // ========================================

          if (allCategories.length === 0) {
            console.warn('🔥 [Debug] API returned empty categories!');
            // 提示用户：源没有返回分类数据
            setFilteredSourceCategories([]);
            setLoading(false);
            return;
          }

          // 【绝对直通】直接使用原始分类，不过滤
          setFilteredSourceCategories(allCategories);

          // 【强制自动选中】立即选中第一个分类
          const firstCategory = allCategories[0];
          setSelectedSourceCategory(firstCategory);

          // 立即触发数据加载（不等待用户点击）
          fetchSourceCategoryData(firstCategory);
        } catch (err) {
          console.error('🔥 [Debug] Fetch error:', err);
          setFilteredSourceCategories([]); // 出错时清空
          setLoading(false);
        }
      }
    },
    [currentSource, setCurrentSource, type, sources, fetchSourceCategoryData],
  );

  // 处理源分类切换
  const handleSourceCategoryChange = useCallback(
    (category: SourceCategory) => {
      if (selectedSourceCategory?.type_id !== category.type_id) {
        setLoading(true);
        setCurrentPage(0);
        setSourceData([]);
        setHasMore(true);
        setIsLoadingMore(false);
        setSelectedSourceCategory(category);
        // 触发源分类数据加载
        fetchSourceCategoryData(category);
      }
    },
    [selectedSourceCategory, fetchSourceCategoryData],
  );

  const getPageTitle = () => {
    // 根据 type 生成标题
    return type === 'movie'
      ? '电影'
      : type === 'tv'
        ? '电视剧'
        : type === 'anime'
          ? '动漫'
          : type === 'show'
            ? '综艺'
            : '自定义';
  };

  const getPageDescription = () => {
    if (type === 'anime' && primarySelection === '每日放送') {
      return '来自 Bangumi 番组计划的精选内容';
    }
    return '来自豆瓣的精选内容';
  };

  const getActivePath = () => {
    const params = new URLSearchParams();
    if (type) params.set('type', type);

    const queryString = params.toString();
    const activePath = `/douban${queryString ? `?${queryString}` : ''}`;
    return activePath;
  };

  return (
    <PageLayout activePath={getActivePath()}>
      <div className='px-4 sm:px-10 py-4 sm:py-8 overflow-visible'>
        {/* 页面标题和选择器 */}
        <div className='mb-6 sm:mb-8 space-y-4 sm:space-y-6'>
          {/* 页面标题 */}
          <div>
            <h1 className='text-2xl sm:text-3xl font-bold text-gray-800 mb-1 sm:mb-2 dark:text-gray-200'>
              {getPageTitle()}
            </h1>
            <p className='text-sm sm:text-base text-gray-600 dark:text-gray-400'>
              {getPageDescription()}
            </p>
          </div>

          {/* 选择器组件 */}
          {type !== 'custom' ? (
            <div className='bg-white/60 dark:bg-gray-800/40 rounded-2xl p-4 sm:p-6 border border-gray-200/30 dark:border-gray-700/30'>
              <DoubanSelector
                type={type as 'movie' | 'tv' | 'show' | 'anime'}
                primarySelection={primarySelection}
                secondarySelection={secondarySelection}
                onPrimaryChange={handlePrimaryChange}
                onSecondaryChange={handleSecondaryChange}
                onMultiLevelChange={handleMultiLevelChange}
                onWeekdayChange={handleWeekdayChange}
                // 数据源相关 props
                sources={sources}
                currentSource={currentSource}
                // 【核心修复】使用 filteredSourceCategories state 而非 getFilteredCategories
                // 这样确保渲染的分类与 handleSourceChange 处理的分类一致
                sourceCategories={
                  currentSource !== 'auto'
                    ? filteredSourceCategories
                    : getFilteredCategories(
                        type as 'movie' | 'tv' | 'anime' | 'show',
                      )
                }
                isLoadingSources={isLoadingSources}
                isLoadingCategories={isLoadingCategories}
                onSourceChange={handleSourceChange}
                onSourceCategoryChange={handleSourceCategoryChange}
                selectedSourceCategory={selectedSourceCategory}
              />
            </div>
          ) : (
            <div className='bg-white/60 dark:bg-gray-800/40 rounded-2xl p-4 sm:p-6 border border-gray-200/30 dark:border-gray-700/30'>
              <DoubanCustomSelector
                customCategories={customCategories}
                primarySelection={primarySelection}
                secondarySelection={secondarySelection}
                onPrimaryChange={handlePrimaryChange}
                onSecondaryChange={handleSecondaryChange}
              />
            </div>
          )}
        </div>

        {/* 内容展示区域 */}
        <div className='max-w-[95%] mx-auto mt-8 overflow-visible'>
          {/* 内容网格 */}
          <div className='justify-start grid grid-cols-3 gap-x-2 gap-y-12 px-0 sm:px-2 sm:grid-cols-[repeat(auto-fill,minmax(160px,1fr))] sm:gap-x-8 sm:gap-y-20'>
            {loading || isLoadingSourceData || !selectorsReady ? (
              // 显示骨架屏
              skeletonData.map((index) => <DoubanCardSkeleton key={index} />)
            ) : currentSource !== 'auto' && sourceData.length > 0 ? (
              // 显示源分类数据
              sourceData.map((item, index) => (
                <div key={`source-${item.id}-${index}`} className='w-full'>
                  <VideoCard
                    from='douban'
                    title={item.title}
                    poster={item.poster}
                    year={item.year}
                    type={type === 'movie' ? 'movie' : ''}
                  />
                </div>
              ))
            ) : currentSource !== 'auto' && selectedSourceCategory ? (
              // 选择了源分类但没有数据
              <div className='col-span-full text-center py-12 text-gray-500 dark:text-gray-400'>
                <p>该分类暂无数据</p>
                <p className='text-sm mt-2'>请尝试选择其他分类</p>
              </div>
            ) : currentSource !== 'auto' && !selectedSourceCategory ? (
              // 选择了源但未选择分类
              <div className='col-span-full text-center py-12 text-gray-500 dark:text-gray-400'>
                <p>请选择一个分类</p>
                <p className='text-sm mt-2'>从上方分类列表中选择</p>
              </div>
            ) : (
              // 显示豆瓣数据
              doubanData.map((item, index) => (
                <div key={`${item.title}-${index}`} className='w-full'>
                  <VideoCard
                    from='douban'
                    title={item.title}
                    poster={item.poster}
                    douban_id={Number(item.id)}
                    rate={item.rate}
                    year={item.year}
                    type={type === 'movie' ? 'movie' : ''}
                    isBangumi={
                      type === 'anime' && primarySelection === '每日放送'
                    }
                  />
                </div>
              ))
            )}
          </div>

          {/* 加载更多指示器 */}
          {hasMore && !loading && (
            <div
              ref={(el) => {
                if (el && el.offsetParent !== null) {
                  (
                    loadingRef as React.MutableRefObject<HTMLDivElement | null>
                  ).current = el;
                }
              }}
              className='flex justify-center mt-12 py-8'
            >
              {isLoadingMore && (
                <div className='flex items-center gap-2'>
                  <div className='animate-spin rounded-full h-6 w-6 border-b-2 border-green-500'></div>
                  <span className='text-gray-600'>加载中...</span>
                </div>
              )}
            </div>
          )}

          {/* 没有更多数据提示 */}
          {!hasMore && doubanData.length > 0 && (
            <div className='text-center text-gray-500 py-8'>已加载全部内容</div>
          )}

          {/* 空状态 */}
          {!loading && doubanData.length === 0 && (
            <div className='text-center text-gray-500 py-8'>暂无相关内容</div>
          )}
        </div>
      </div>
    </PageLayout>
  );
}

export default function DoubanPage() {
  return (
    <Suspense>
      <DoubanPageClient />
    </Suspense>
  );
}
