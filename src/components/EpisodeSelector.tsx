/* eslint-disable @next/next/no-img-element */

import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'next/navigation';
import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import {
  SearchResult,
  SourceHealthStats,
  SourceTestResult,
} from '@/lib/types';
import { useProxyImage } from '@/hooks/useProxyImage';

// 定义视频信息类型
interface VideoInfo {
  quality: string;
  loadSpeed: string;
  pingTime: number;
  hasError?: boolean; // 添加错误状态标识
}

// 源封面组件 - 使用 Tauri proxy_image
const SourcePoster: React.FC<{ poster: string; title: string }> = ({ poster, title }) => {
  const { url: proxiedUrl } = useProxyImage(poster);

  return (
    <img
      src={proxiedUrl}
      alt={title}
      className='w-full h-full object-cover'
      onError={(e) => {
        const target = e.target as HTMLImageElement;
        target.style.display = 'none';
      }}
    />
  );
};

interface EpisodeSelectorProps {
  /** 总集数 */
  totalEpisodes: number;
  /** 剧集标题 */
  episodes_titles: string[];
  /** 每页显示多少集，默认 50 */
  episodesPerPage?: number;
  /** 当前选中的集数（1 开始） */
  value?: number;
  /** 用户点击选集后的回调 */
  onChange?: (episodeNumber: number) => void;
  /** 换源相关 */
  onSourceChange?: (source: string, id: string, title: string) => void;
  currentSource?: string;
  currentId?: string;
  videoTitle?: string;
  videoYear?: string;
  availableSources?: SearchResult[];
  sourceSearchLoading?: boolean;
  sourceSearchError?: string | null;
  /** 预计算的测速结果，避免重复测速 */
  precomputedVideoInfo?: Map<string, VideoInfo>;
  /** 是否启用优选和测速功能 */
  optimizationEnabled?: boolean;
  sourceHealthMap?: ReadonlyMap<string, SourceHealthStats>;
}

/**
 * 选集组件，支持分页、自动滚动聚焦当前分页标签，以及换源功能。
 */
const EpisodeSelector: React.FC<EpisodeSelectorProps> = ({
  totalEpisodes,
  episodes_titles,
  episodesPerPage = 50,
  value = 1,
  onChange,
  onSourceChange,
  currentSource,
  currentId,
  videoTitle,
  availableSources = [],
  sourceSearchLoading = false,
  sourceSearchError = null,
  precomputedVideoInfo,
  optimizationEnabled = true,
  sourceHealthMap,
}) => {
  const router = useRouter();
  const pageCount = Math.ceil(totalEpisodes / episodesPerPage);

  // 存储每个源的视频信息
  const [videoInfoMap, setVideoInfoMap] = useState<Map<string, VideoInfo>>(
    new Map(),
  );
  const [attemptedSources, setAttemptedSources] = useState<Set<string>>(
    new Set(),
  );

  // 使用 ref 来避免闭包问题
  const attemptedSourcesRef = useRef<Set<string>>(new Set());
  const videoInfoMapRef = useRef<Map<string, VideoInfo>>(new Map());

  // 同步状态到 ref
  useEffect(() => {
    attemptedSourcesRef.current = attemptedSources;
  }, [attemptedSources]);

  useEffect(() => {
    videoInfoMapRef.current = videoInfoMap;
  }, [videoInfoMap]);

  // 主要的 tab 状态：'episodes' 或 'sources'
  // 当只有一集时默认展示 "换源"，并隐藏 "选集" 标签
  const [activeTab, setActiveTab] = useState<'episodes' | 'sources'>(
    totalEpisodes > 1 ? 'episodes' : 'sources',
  );

  // 当前分页索引（0 开始）
  const initialPage = Math.floor((value - 1) / episodesPerPage);
  const [currentPage, setCurrentPage] = useState<number>(initialPage);

  // 是否倒序显示
  const [descending, setDescending] = useState<boolean>(false);

  // 根据 descending 状态计算实际显示的分页索引
  const displayPage = useMemo(() => {
    if (descending) {
      return pageCount - 1 - currentPage;
    }
    return currentPage;
  }, [currentPage, descending, pageCount]);

  // 获取视频信息的函数 - 移除 attemptedSources 依赖避免不必要的重新创建
  const getVideoInfo = useCallback(async (source: SearchResult) => {
    const sourceKey = `${source.source}-${source.id}`;

    // 使用 ref 获取最新的状态，避免闭包问题
    if (attemptedSourcesRef.current.has(sourceKey)) {
      return;
    }

    // 获取第一集的URL
    if (!source.episodes || source.episodes.length === 0) {
      return;
    }
    const episodeUrl =
      source.episodes.length > 1 ? source.episodes[1] : source.episodes[0];

    // 标记为已尝试
    setAttemptedSources((prev) => new Set(prev).add(sourceKey));

    try {
      const result = await invoke<SourceTestResult>('test_video_source_command', {
        m3u8Url: episodeUrl,
        sourceKey: source.source,
      });

      // 转换字段名：Rust 使用 snake_case，前端使用 camelCase
      setVideoInfoMap((prev) =>
        new Map(prev).set(sourceKey, {
          quality: result.quality,
          loadSpeed: result.load_speed,
          pingTime: result.ping_time,
          hasError: result.has_error,
        }),
      );
    } catch {
      // 失败时保存错误状态
      setVideoInfoMap((prev) =>
        new Map(prev).set(sourceKey, {
          quality: '错误',
          loadSpeed: '未知',
          pingTime: 0,
          hasError: true,
        }),
      );
    }
  }, []);

  // 当有预计算结果时，先合并到videoInfoMap中
  useEffect(() => {
    if (precomputedVideoInfo && precomputedVideoInfo.size > 0) {
      // 原子性地更新两个状态，避免时序问题
      setVideoInfoMap((prev) => {
        const newMap = new Map(prev);
        precomputedVideoInfo.forEach((value, key) => {
          newMap.set(key, value);
        });
        return newMap;
      });

      setAttemptedSources((prev) => {
        const newSet = new Set(prev);
        precomputedVideoInfo.forEach((info, key) => {
          if (!info.hasError) {
            newSet.add(key);
          }
        });
        return newSet;
      });

      // 同步更新 ref，确保 getVideoInfo 能立即看到更新
      precomputedVideoInfo.forEach((info, key) => {
        if (!info.hasError) {
          attemptedSourcesRef.current.add(key);
        }
      });
    }
  }, [precomputedVideoInfo]);

  // 当切换到换源tab并且有源数据时，异步获取视频信息 - 移除 attemptedSources 依赖避免循环触发
  useEffect(() => {
    const fetchVideoInfosInBatches = async () => {
      if (
        !optimizationEnabled || // 若关闭测速则直接退出
        activeTab !== 'sources' ||
        availableSources.length === 0
      )
        return;

      // 筛选出尚未测速的播放源
      const pendingSources = availableSources.filter((source) => {
        const sourceKey = `${source.source}-${source.id}`;
        return !attemptedSourcesRef.current.has(sourceKey);
      });

      if (pendingSources.length === 0) return;

      const batchSize = Math.ceil(pendingSources.length / 2);

      for (let start = 0; start < pendingSources.length; start += batchSize) {
        const batch = pendingSources.slice(start, start + batchSize);
        await Promise.all(batch.map(getVideoInfo));
      }
    };

    fetchVideoInfosInBatches();
    // 依赖项保持与之前一致
  }, [activeTab, availableSources, getVideoInfo, optimizationEnabled]);

  // 升序分页标签
  const categoriesAsc = useMemo(() => {
    return Array.from({ length: pageCount }, (_, i) => {
      const start = i * episodesPerPage + 1;
      const end = Math.min(start + episodesPerPage - 1, totalEpisodes);
      return { start, end };
    });
  }, [pageCount, episodesPerPage, totalEpisodes]);

  // 根据 descending 状态决定分页标签的排序和内容
  const categories = useMemo(() => {
    if (descending) {
      // 倒序时，label 也倒序显示
      return [...categoriesAsc]
        .reverse()
        .map(({ start, end }) => `${end}-${start}`);
    }
    return categoriesAsc.map(({ start, end }) => `${start}-${end}`);
  }, [categoriesAsc, descending]);

  const categoryContainerRef = useRef<HTMLDivElement>(null);
  const buttonRefs = useRef<(HTMLButtonElement | null)[]>([]);

  // 添加鼠标悬停状态管理
  const [isCategoryHovered, setIsCategoryHovered] = useState(false);

  // 阻止页面竖向滚动
  const preventPageScroll = useCallback(
    (e: WheelEvent) => {
      if (isCategoryHovered) {
        e.preventDefault();
      }
    },
    [isCategoryHovered],
  );

  // 处理滚轮事件，实现横向滚动
  const handleWheel = useCallback(
    (e: WheelEvent) => {
      if (isCategoryHovered && categoryContainerRef.current) {
        e.preventDefault(); // 阻止默认的竖向滚动

        const container = categoryContainerRef.current;
        const scrollAmount = e.deltaY * 2; // 调整滚动速度

        // 根据滚轮方向进行横向滚动
        container.scrollBy({
          left: scrollAmount,
          behavior: 'smooth',
        });
      }
    },
    [isCategoryHovered],
  );

  // 添加全局wheel事件监听器
  useEffect(() => {
    if (isCategoryHovered) {
      // 鼠标悬停时阻止页面滚动
      document.addEventListener('wheel', preventPageScroll, { passive: false });
      document.addEventListener('wheel', handleWheel, { passive: false });
    } else {
      // 鼠标离开时恢复页面滚动
      document.removeEventListener('wheel', preventPageScroll);
      document.removeEventListener('wheel', handleWheel);
    }

    return () => {
      document.removeEventListener('wheel', preventPageScroll);
      document.removeEventListener('wheel', handleWheel);
    };
  }, [isCategoryHovered, preventPageScroll, handleWheel]);

  // 当分页切换时，将激活的分页标签滚动到视口中间
  useEffect(() => {
    const btn = buttonRefs.current[displayPage];
    const container = categoryContainerRef.current;
    if (btn && container) {
      // 手动计算滚动位置，只滚动分页标签容器
      const containerRect = container.getBoundingClientRect();
      const btnRect = btn.getBoundingClientRect();
      const scrollLeft = container.scrollLeft;

      // 计算按钮相对于容器的位置
      const btnLeft = btnRect.left - containerRect.left + scrollLeft;
      const btnWidth = btnRect.width;
      const containerWidth = containerRect.width;

      // 计算目标滚动位置，使按钮居中
      const targetScrollLeft = btnLeft - (containerWidth - btnWidth) / 2;

      // 平滑滚动到目标位置
      container.scrollTo({
        left: targetScrollLeft,
        behavior: 'smooth',
      });
    }
  }, [displayPage, pageCount]);

  // 处理换源tab点击，只在点击时才搜索
  const handleSourceTabClick = () => {
    setActiveTab('sources');
  };

  const handleCategoryClick = useCallback(
    (index: number) => {
      if (descending) {
        // 在倒序时，需要将显示索引转换为实际索引
        setCurrentPage(pageCount - 1 - index);
      } else {
        setCurrentPage(index);
      }
    },
    [descending, pageCount],
  );

  const handleEpisodeClick = useCallback(
    (episodeNumber: number) => {
      onChange?.(episodeNumber);
    },
    [onChange],
  );

  const handleSourceClick = useCallback(
    (source: SearchResult) => {
      onSourceChange?.(source.source, source.id, source.title);
    },
    [onSourceChange],
  );

  const currentStart = currentPage * episodesPerPage + 1;
  const currentEnd = Math.min(
    currentStart + episodesPerPage - 1,
    totalEpisodes,
  );

  return (
    <div className='flex h-full min-h-0 flex-col overflow-hidden rounded-[1.4rem] border border-black/5 bg-black/10 px-3.5 py-0 shadow-[0_16px_40px_-28px_rgba(15,23,42,0.45)] backdrop-blur-sm sm:px-4 min-[834px]:rounded-[1.55rem] min-[834px]:px-5 xl:ml-2 xl:border-white/10 dark:border-white/15 dark:bg-white/5'>
      {/* 主要的 Tab 切换 - 无缝融入设计 */}
      <div className='-mx-3.5 mb-1.5 flex shrink-0 sm:-mx-4 min-[834px]:-mx-5'>
        {totalEpisodes > 1 && (
          <div
            onClick={() => setActiveTab('episodes')}
            className={`flex-1 cursor-pointer px-4 py-3.5 text-center text-[0.95rem] font-semibold transition-all duration-200 sm:text-[0.95rem] min-[834px]:py-3.5
              ${
                activeTab === 'episodes'
                  ? 'text-green-600 dark:text-green-400'
                  : 'text-gray-700 hover:text-green-600 bg-black/5 dark:bg-white/5 dark:text-gray-300 dark:hover:text-green-400 hover:bg-black/3 dark:hover:bg-white/3'
              }
            `.trim()}
          >
            选集
          </div>
        )}
        <div
          onClick={handleSourceTabClick}
          className={`flex-1 cursor-pointer px-4 py-3.5 text-center text-[0.95rem] font-semibold transition-all duration-200 sm:text-[0.95rem] min-[834px]:py-3.5
            ${
              activeTab === 'sources'
                ? 'text-green-600 dark:text-green-400'
                : 'text-gray-700 hover:text-green-600 bg-black/5 dark:bg-white/5 dark:text-gray-300 dark:hover:text-green-400 hover:bg-black/3 dark:hover:bg-white/3'
            }
          `.trim()}
        >
          换源
        </div>
      </div>

      {/* 选集 Tab 内容 */}
      {activeTab === 'episodes' && (
        <>
          {/* 分类标签 */}
          <div className='-mx-3 mb-4 flex shrink-0 items-center gap-4 border-b border-gray-300 px-3 sm:-mx-4 sm:px-4 min-[834px]:-mx-5 min-[834px]:px-5 dark:border-gray-700'>
            <div
              className='flex-1 overflow-x-auto'
              ref={categoryContainerRef}
              onMouseEnter={() => setIsCategoryHovered(true)}
              onMouseLeave={() => setIsCategoryHovered(false)}
            >
              <div className='flex gap-2 min-w-max'>
                {categories.map((label, idx) => {
                  const isActive = idx === displayPage;
                  return (
                    <button
                      key={label}
                      ref={(el) => {
                        buttonRefs.current[idx] = el;
                      }}
                      onClick={() => handleCategoryClick(idx)}
                      className={`w-20 relative py-2 text-sm font-medium transition-colors whitespace-nowrap shrink-0 text-center 
                        ${
                          isActive
                            ? 'text-green-500 dark:text-green-400'
                            : 'text-gray-700 hover:text-green-600 dark:text-gray-300 dark:hover:text-green-400'
                        }
                      `.trim()}
                    >
                      {label}
                      {isActive && (
                        <div className='absolute bottom-0 left-0 right-0 h-0.5 bg-green-500 dark:bg-green-400' />
                      )}
                    </button>
                  );
                })}
              </div>
            </div>
            {/* 向上/向下按钮 */}
            <button
              className='shrink-0 w-8 h-8 rounded-md flex items-center justify-center text-gray-700 hover:text-green-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:text-green-400 dark:hover:bg-white/20 transition-colors transform -translate-y-1'
              onClick={() => {
                // 切换集数排序（正序/倒序）
                setDescending((prev) => !prev);
              }}
            >
              <svg
                className='w-4 h-4'
                fill='none'
                stroke='currentColor'
                viewBox='0 0 24 24'
              >
                <path
                  strokeLinecap='round'
                  strokeLinejoin='round'
                  strokeWidth='2'
                  d='M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4'
                />
              </svg>
            </button>
          </div>

          {/* 集数网格 */}
          <div className='flex flex-wrap gap-3 overflow-y-auto flex-1 content-start pb-4'>
            {(() => {
              const len = currentEnd - currentStart + 1;
              const episodes = Array.from({ length: len }, (_, i) =>
                descending ? currentEnd - i : currentStart + i,
              );
              return episodes;
            })().map((episodeNumber) => {
              const isActive = episodeNumber === value;
              return (
                <button
                  key={episodeNumber}
                  onClick={() => handleEpisodeClick(episodeNumber - 1)}
                  className={`h-10 min-w-10 px-3 py-2 flex items-center justify-center text-sm font-medium rounded-md transition-all duration-200 whitespace-nowrap font-mono
                    ${
                      isActive
                        ? 'bg-green-500 text-white shadow-lg shadow-green-500/25 dark:bg-green-600'
                        : 'bg-gray-200 text-gray-700 hover:bg-gray-300 hover:scale-105 dark:bg-white/10 dark:text-gray-300 dark:hover:bg-white/20'
                    }`.trim()}
                >
                  {(() => {
                    const title = episodes_titles?.[episodeNumber - 1];
                    if (!title) {
                      return episodeNumber;
                    }
                    // 如果匹配"第X集"、"第X话"、"X集"、"X话"格式，提取中间的数字
                    const match = title.match(/(?:第)?(\d+)(?:集|话)/);
                    if (match) {
                      return match[1];
                    }
                    return title;
                  })()}
                </button>
              );
            })}
          </div>
        </>
      )}

      {/* 换源 Tab 内容 */}
      {activeTab === 'sources' && (
        <div className='mt-3 flex min-h-0 flex-1 flex-col overflow-hidden min-[834px]:mt-4'>
          {sourceSearchLoading && (
            <div className='flex items-center justify-center py-8'>
              <div className='animate-spin rounded-full h-8 w-8 border-b-2 border-green-500'></div>
              <span className='ml-2 text-sm text-gray-600 dark:text-gray-300'>
                搜索中...
              </span>
            </div>
          )}

          {sourceSearchError && (
            <div className='flex items-center justify-center py-8'>
              <div className='text-center'>
                <div className='text-red-500 text-2xl mb-2'>⚠️</div>
                <p className='text-sm text-red-600 dark:text-red-400'>
                  {sourceSearchError}
                </p>
              </div>
            </div>
          )}

          {!sourceSearchLoading &&
            !sourceSearchError &&
            availableSources.length === 0 && (
              <div className='flex items-center justify-center py-8'>
                <div className='text-center'>
                  <div className='text-gray-400 text-2xl mb-2'>📺</div>
                  <p className='text-sm text-gray-600 dark:text-gray-300'>
                    暂无可用的换源
                  </p>
                </div>
              </div>
            )}

          {!sourceSearchLoading &&
            !sourceSearchError &&
            availableSources.length > 0 && (
              <div className='min-h-0 flex-1 overflow-hidden rounded-2xl border border-black/5 bg-white/20 p-1.5 dark:border-white/10 dark:bg-black/10'>
                <div className='source-scrollbar h-full min-h-0 overflow-y-auto space-y-3.5 pr-1.5 pb-20 xl:space-y-2 xl:pb-5'>
                {availableSources
                  .sort((a, b) => {
                    const aIsCurrent =
                      a.source?.toString() === currentSource?.toString() &&
                      a.id?.toString() === currentId?.toString();
                    const bIsCurrent =
                      b.source?.toString() === currentSource?.toString() &&
                      b.id?.toString() === currentId?.toString();
                    if (aIsCurrent && !bIsCurrent) return -1;
                    if (!aIsCurrent && bIsCurrent) return 1;
                    return 0;
                  })
                  .map((source, index) => {
                    const isCurrentSource =
                      source.source?.toString() === currentSource?.toString() &&
                      source.id?.toString() === currentId?.toString();
                    const sourceHealth = sourceHealthMap?.get(source.source);
                    return (
                      <div
                        key={`${source.source}-${source.id}`}
                        onClick={() =>
                          !isCurrentSource && handleSourceClick(source)
                        }
                        className={`relative flex items-start gap-3.5 rounded-2xl border px-3.5 py-4 transition-all duration-200 select-none sm:gap-3.5 sm:px-3.5 min-[834px]:px-4 min-[834px]:py-4 xl:gap-3 xl:px-3 xl:py-3
                      ${
                        isCurrentSource
                          ? 'border-green-500/30 bg-green-500/10 shadow-[0_12px_24px_-18px_rgba(34,197,94,0.6)] dark:bg-green-500/20'
                          : 'border-transparent hover:bg-gray-200/50 dark:hover:bg-white/10 hover:scale-[1.01] cursor-pointer'
                      }`.trim()}
                      >
                        {/* 封面 */}
                        <div className='shrink-0 h-[5.85rem] w-[4.15rem] overflow-hidden rounded-xl bg-gray-300 sm:h-[5.5rem] sm:w-[3.9rem] min-[834px]:h-[6rem] min-[834px]:w-[4.2rem] xl:h-[4.9rem] xl:w-[3.45rem] dark:bg-gray-600'>
                          {source.episodes && source.episodes.length > 0 && (
                            <SourcePoster poster={source.poster} title={source.title} />
                          )}
                        </div>

                        {/* 信息区域 */}
                        <div className='flex min-w-0 flex-1 flex-col justify-between gap-2.5 min-[834px]:gap-2.5 xl:gap-1.5'>
                          {/* 标题和分辨率 - 顶部 */}
                          <div className='flex items-start justify-between gap-2.5'>
                            <div className='flex-1 min-w-0 relative group/title'>
                              <h3
                                className='text-[1.22rem] font-semibold leading-[1.12] tracking-[-0.02em] text-gray-900 sm:text-[1.12rem] min-[834px]:text-[1.2rem] xl:text-[1rem] dark:text-gray-100'
                                style={{
                                  display: '-webkit-box',
                                  WebkitLineClamp: 2,
                                  WebkitBoxOrient: 'vertical',
                                  overflow: 'hidden',
                                }}
                              >
                                {source.title}
                              </h3>
                              {/* 标题级别的 tooltip - 第一个元素不显示 */}
                              {index !== 0 && (
                                <div className='absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-1 bg-gray-800 text-white text-xs rounded-md shadow-lg opacity-0 invisible group-hover/title:opacity-100 group-hover/title:visible transition-all duration-200 ease-out delay-100 whitespace-nowrap z-500 pointer-events-none'>
                                  {source.title}
                                  <div className='absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-800'></div>
                                </div>
                              )}
                            </div>
                            {(() => {
                              const sourceKey = `${source.source}-${source.id}`;
                              const videoInfo = videoInfoMap.get(sourceKey);

                              if (videoInfo && videoInfo.quality !== '未知') {
                                if (videoInfo.hasError) {
                                  return (
                                    <div className='bg-gray-500/10 dark:bg-gray-400/20 text-red-600 dark:text-red-400 px-1.5 py-0 rounded text-xs shrink-0 min-w-12.5 text-center'>
                                      检测失败
                                    </div>
                                  );
                                } else {
                                  // 根据分辨率设置不同颜色：2K、4K为紫色，1080p、720p为绿色，其他为黄色
                                  const isUltraHigh = ['4K', '2K'].includes(
                                    videoInfo.quality,
                                  );
                                  const isHigh = ['1080p', '720p'].includes(
                                    videoInfo.quality,
                                  );
                                  const textColorClasses = isUltraHigh
                                    ? 'text-purple-600 dark:text-purple-400'
                                    : isHigh
                                      ? 'text-green-600 dark:text-green-400'
                                      : 'text-yellow-600 dark:text-yellow-400';

                                  return (
                                    <div
                                      className={`shrink-0 rounded-full bg-gray-500/10 px-2.5 py-0.5 text-xs font-semibold text-center dark:bg-gray-400/20 ${textColorClasses}`}
                                    >
                                      {videoInfo.quality}
                                    </div>
                                  );
                                }
                              }

                              return null;
                            })()}
                          </div>

                          {/* 源名称和集数信息 - 垂直居中 */}
                          <div className='flex flex-wrap items-center gap-2.5 xl:gap-1.5'>
                            <span className='rounded-full border border-gray-400/60 px-2.5 py-1 text-xs font-medium text-gray-700 dark:border-gray-500 dark:text-gray-300'>
                              {source.source_name}
                            </span>
                            {sourceHealth && (
                              <>
                                <span
                                  className={`rounded-full px-2.5 py-1 text-xs font-medium ${
                                    sourceHealth.auto_degraded
                                      ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300'
                                      : 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300'
                                  }`}
                                >
                                  {sourceHealth.auto_degraded ? '已降级' : '健康'}
                                </span>
                              </>
                            )}
                            {source.episodes.length > 1 && (
                              <span className='text-xs font-medium text-gray-500 dark:text-gray-400'>
                                {source.episodes.length} 集
                              </span>
                            )}
                          </div>

                          {sourceHealth && (
                            <div className='flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs text-gray-500 dark:text-gray-400 xl:gap-x-2.5'>
                              <span>成功率 {sourceHealth.success_rate.toFixed(0)}%</span>
                              <span>
                                均响 {sourceHealth.avg_response_time_ms || '--'}ms
                              </span>
                            </div>
                          )}

                          {/* 网络信息 - 底部 */}
                          <div className='flex min-h-5 items-end'>
                            {(() => {
                              const sourceKey = `${source.source}-${source.id}`;
                              const videoInfo = videoInfoMap.get(sourceKey);
                              if (videoInfo) {
                                if (!videoInfo.hasError) {
                                  return (
                                    <div className='flex flex-wrap items-center gap-x-3.5 gap-y-1 text-xs xl:gap-x-2.5'>
                                      <div className='text-green-600 dark:text-green-400 font-semibold'>
                                        {videoInfo.loadSpeed}
                                      </div>
                                      <div className='text-orange-600 dark:text-orange-400 font-semibold'>
                                        {videoInfo.pingTime}ms
                                      </div>
                                    </div>
                                  );
                                } else {
                                  return (
                                    <div className='text-red-500/90 dark:text-red-400 font-medium text-xs'>
                                      无测速数据
                                    </div>
                                  ); // 占位div
                                }
                              }
                            })()}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                <div className='shrink-0 mt-auto pt-2 border-t border-gray-400 dark:border-gray-700'>
                  <button
                    onClick={() => {
                      if (videoTitle) {
                        router.push(
                          `/search?q=${encodeURIComponent(videoTitle)}`,
                        );
                      }
                    }}
                    className='w-full text-center text-xs text-gray-500 dark:text-gray-400 hover:text-green-500 dark:hover:text-green-400 transition-colors py-2'
                  >
                    影片匹配有误？点击去搜索
                  </button>
                </div>
                </div>
              </div>
            )}
        </div>
      )}
    </div>
  );
};

export default EpisodeSelector;
