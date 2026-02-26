/* eslint-disable react-hooks/exhaustive-deps, no-console */

'use client';

import { invoke } from '@tauri-apps/api/core';
import { ChevronRight, Sparkles, X } from 'lucide-react';
import Link from 'next/link';
import { Suspense, useEffect, useState } from 'react';

import {
  BangumiItem,
  DoubanItem,
  FavoriteCard,
  HomePageData,
} from '@/lib/types';
import { subscribeToDataUpdates } from '@/lib/utils';
import { useImagePreload } from '@/hooks/useImagePreload';
import { useCachedData } from '@/hooks/usePageCache';

import CapsuleSwitch from '@/components/CapsuleSwitch';
import ContinueWatching from '@/components/ContinueWatching';
import PageLayout from '@/components/PageLayout';
import ScrollableRow from '@/components/ScrollableRow';
import { useSite } from '@/components/SiteProvider';
import VideoCard from '@/components/VideoCard';

function HomeClient() {
  const [activeTab, setActiveTab] = useState<'home' | 'favorites'>('home');
  const [hotMovies, setHotMovies] = useState<DoubanItem[]>([]);
  const [hotTvShows, setHotTvShows] = useState<DoubanItem[]>([]);
  const [hotVarietyShows, setHotVarietyShows] = useState<DoubanItem[]>([]);
  const [todayBangumi, setTodayBangumi] = useState<BangumiItem[]>([]);
  const [loading, setLoading] = useState(true);
  const { announcement } = useSite();

  const [showAnnouncement, setShowAnnouncement] = useState(false);

  // 检查公告弹窗状态
  useEffect(() => {
    const checkAnnouncement = async () => {
      if (typeof window !== 'undefined' && announcement) {
        try {
          const prefs = await invoke<{ has_seen_announcement: string }>(
            'get_user_preferences',
          );
          const hasSeenAnnouncement = prefs.has_seen_announcement;
          if (hasSeenAnnouncement !== announcement) {
            setShowAnnouncement(true);
          } else {
            setShowAnnouncement(Boolean(!hasSeenAnnouncement && announcement));
          }
        } catch (error) {
          console.error('读取公告状态失败:', error);
          // 出错时默认显示公告
          setShowAnnouncement(true);
        }
      }
    };
    checkAnnouncement();
  }, [announcement]);

  // 收藏夹数据

  const [favoriteItems, setFavoriteItems] = useState<FavoriteCard[]>([]);

  // 图片预加载：提取所有图片 URL
  const allImageUrls = [
    ...hotMovies.map((m) => m.poster),
    ...hotTvShows.map((m) => m.poster),
    ...hotVarietyShows.map((m) => m.poster),
    ...todayBangumi.map((item) => item.images?.large || item.images?.common),
    ...favoriteItems.map((f) => f.poster),
  ].filter((url): url is string => Boolean(url)); // 过滤空值并确保类型

  // 自动预加载（延迟 500ms，避免阻塞首屏渲染）
  useImagePreload(allImageUrls, !loading);

  // 定义首页数据类型

  // 数据获取函数
  const fetchHomeData = async (): Promise<HomePageData> => {
    const weekdays = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    const weekday = weekdays[new Date().getDay()];
    return await invoke<HomePageData>('get_home_data', { weekday });
  };

  // 使用缓存（启用 stale-while-revalidate）
  const { fetchData } = useCachedData<HomePageData>('home', fetchHomeData, {
    staleWhileRevalidate: true,
    onUpdate: (freshData) => {
      // 后台更新完成后，静默更新状态
      setHotMovies(freshData.hotMovies);
      setHotTvShows(freshData.hotTvShows);
      setHotVarietyShows(freshData.hotVarietyShows);
      setTodayBangumi(freshData.todayBangumi);
    },
  });

  useEffect(() => {
    const loadHomeData = async () => {
      try {
        setLoading(true);
        const data = await fetchData();
        setHotMovies(data.hotMovies);
        setHotTvShows(data.hotTvShows);
        setHotVarietyShows(data.hotVarietyShows);
        setTodayBangumi(data.todayBangumi);
      } catch (error) {
        console.error('获取推荐数据失败:', error);
      } finally {
        setLoading(false);
      }
    };

    loadHomeData();
  }, []);

  // 处理收藏数据更新的函数
  const updateFavoriteItems = async () => {
    try {
      const items = await invoke<FavoriteCard[]>('get_favorite_cards');
      setFavoriteItems(items);
    } catch (err) {
      console.error('获取收藏夹数据失败:', err);
    }
  };

  // 当切换到收藏夹时加载收藏数据
  useEffect(() => {
    if (activeTab !== 'favorites') return;

    updateFavoriteItems();

    // 监听收藏更新事件
    const unsubscribe = subscribeToDataUpdates('favoritesUpdated', () => {
      updateFavoriteItems();
    });

    return unsubscribe;
  }, [activeTab]);

  const handleCloseAnnouncement = async (announcement: string) => {
    setShowAnnouncement(false);
    try {
      // 更新已查看的公告
      await invoke('update_user_preferences', {
        preferences: {
          has_seen_announcement: announcement,
        },
      });
    } catch (error) {
      console.error('保存公告状态失败:', error);
    }
  };

  // 骨架屏组件
  const SkeletonCard = () => (
    <div className='min-w-24 w-24 sm:min-w-45 sm:w-44'>
      <div className='relative aspect-2/3 w-full overflow-hidden rounded-xl bg-gradient-to-br from-gray-200 to-gray-300 dark:from-gray-800 dark:to-gray-700 animate-pulse'>
        <div className='absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer' />
      </div>
      <div className='mt-2.5 h-4 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse w-3/4 mx-auto' />
    </div>
  );

  return (
    <PageLayout>
      <div className='px-3 sm:px-10 py-6 sm:py-8 overflow-visible'>
        {/* 顶部 Tab 切换 - Aurora 风格 */}
        <div className='mb-10 flex justify-center'>
          <CapsuleSwitch
            options={[
              { label: '首页', value: 'home' },
              { label: '收藏夹', value: 'favorites' },
            ]}
            active={activeTab}
            onChange={(value) => setActiveTab(value as 'home' | 'favorites')}
          />
        </div>

        <div className='max-w-[95%] mx-auto'>
          {activeTab === 'favorites' ? (
            // 收藏夹视图
            <section className='mb-8'>
              <div className='mb-6 flex items-center justify-between'>
                <h2 className='text-xl font-bold text-gray-800 dark:text-gray-100 flex items-center gap-2'>
                  <Sparkles className='w-5 h-5 text-purple-500' />
                  我的收藏
                </h2>
                {favoriteItems.length > 0 && (
                  <button
                    className='text-sm text-gray-500 hover:text-red-500 dark:text-gray-400 dark:hover:text-red-400 transition-colors'
                    onClick={async () => {
                      await invoke('clear_all_favorites');
                      setFavoriteItems([]);
                      window.dispatchEvent(
                        new CustomEvent('favoritesUpdated', { detail: {} }),
                      );
                    }}
                  >
                    清空
                  </button>
                )}
              </div>
              <div className='justify-start grid grid-cols-3 gap-x-2 gap-y-14 sm:gap-y-20 px-0 sm:px-2 sm:grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] sm:gap-x-8'>
                {favoriteItems.map((item) => (
                  <div key={item.id + item.source} className='w-full'>
                    <VideoCard
                      query={item.search_title}
                      {...item}
                      from='favorite'
                      type={item.episodes > 1 ? 'tv' : ''}
                    />
                  </div>
                ))}
                {favoriteItems.length === 0 && (
                  <div className='col-span-full text-center text-gray-500 py-16 dark:text-gray-400'>
                    <div className='w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center'>
                      <Sparkles className='w-8 h-8 text-gray-300 dark:text-gray-600' />
                    </div>
                    暂无收藏内容
                  </div>
                )}
              </div>
            </section>
          ) : (
            // 首页视图
            <>
              {/* 继续观看 */}
              <ContinueWatching />

              {/* 热门电影 */}
              <section className='mb-10'>
                <div className='mb-5 flex items-center justify-between'>
                  <h2 className='text-xl font-bold text-gray-800 dark:text-gray-100'>
                    热门电影
                  </h2>
                  <Link
                    href='/douban?type=movie'
                    className='flex items-center text-sm text-purple-600 hover:text-purple-700 dark:text-purple-400 dark:hover:text-purple-300 transition-colors group'
                  >
                    查看更多
                    <ChevronRight className='w-4 h-4 ml-0.5 group-hover:translate-x-0.5 transition-transform' />
                  </Link>
                </div>
                <ScrollableRow>
                  {loading
                    ? Array.from({ length: 8 }).map((_, index) => (
                        <SkeletonCard key={index} />
                      ))
                    : hotMovies.map((movie, index) => (
                        <div
                          key={index}
                          className='min-w-24 w-24 sm:min-w-45 sm:w-44'
                        >
                          <VideoCard
                            from='douban'
                            title={movie.title}
                            poster={movie.poster}
                            douban_id={Number(movie.id)}
                            rate={movie.rate}
                            year={movie.year}
                            type='movie'
                          />
                        </div>
                      ))}
                </ScrollableRow>
              </section>

              {/* 热门剧集 */}
              <section className='mb-10'>
                <div className='mb-5 flex items-center justify-between'>
                  <h2 className='text-xl font-bold text-gray-800 dark:text-gray-100'>
                    热门剧集
                  </h2>
                  <Link
                    href='/douban?type=tv'
                    className='flex items-center text-sm text-purple-600 hover:text-purple-700 dark:text-purple-400 dark:hover:text-purple-300 transition-colors group'
                  >
                    查看更多
                    <ChevronRight className='w-4 h-4 ml-0.5 group-hover:translate-x-0.5 transition-transform' />
                  </Link>
                </div>
                <ScrollableRow>
                  {loading
                    ? Array.from({ length: 8 }).map((_, index) => (
                        <SkeletonCard key={index} />
                      ))
                    : hotTvShows.map((show, index) => (
                        <div
                          key={index}
                          className='min-w-24 w-24 sm:min-w-45 sm:w-44'
                        >
                          <VideoCard
                            from='douban'
                            title={show.title}
                            poster={show.poster}
                            douban_id={Number(show.id)}
                            rate={show.rate}
                            year={show.year}
                          />
                        </div>
                      ))}
                </ScrollableRow>
              </section>

              {/* 每日新番放送 */}
              <section className='mb-10'>
                <div className='mb-5 flex items-center justify-between'>
                  <h2 className='text-xl font-bold text-gray-800 dark:text-gray-100'>
                    新番放送
                  </h2>
                  <Link
                    href='/douban?type=anime'
                    className='flex items-center text-sm text-purple-600 hover:text-purple-700 dark:text-purple-400 dark:hover:text-purple-300 transition-colors group'
                  >
                    查看更多
                    <ChevronRight className='w-4 h-4 ml-0.5 group-hover:translate-x-0.5 transition-transform' />
                  </Link>
                </div>
                <ScrollableRow>
                  {loading
                    ? Array.from({ length: 8 }).map((_, index) => (
                        <SkeletonCard key={index} />
                      ))
                    : todayBangumi.map((anime, index) => (
                        <div
                          key={`${anime.id}-${index}`}
                          className='min-w-24 w-24 sm:min-w-45 sm:w-44'
                        >
                          <VideoCard
                            from='douban'
                            title={anime.name_cn || anime.name}
                            poster={
                              anime.images?.large ||
                              anime.images?.common ||
                              anime.images?.medium ||
                              anime.images?.small ||
                              anime.images?.grid ||
                              '/logo.png'
                            }
                            douban_id={anime.id}
                            rate={anime.rating?.score?.toFixed(1) || ''}
                            year={anime.air_date?.split('-')?.[0] || ''}
                            isBangumi={true}
                          />
                        </div>
                      ))}
                </ScrollableRow>
              </section>

              {/* 热门综艺 */}
              <section className='mb-10'>
                <div className='mb-5 flex items-center justify-between'>
                  <h2 className='text-xl font-bold text-gray-800 dark:text-gray-100'>
                    热门综艺
                  </h2>
                  <Link
                    href='/douban?type=show'
                    className='flex items-center text-sm text-purple-600 hover:text-purple-700 dark:text-purple-400 dark:hover:text-purple-300 transition-colors group'
                  >
                    查看更多
                    <ChevronRight className='w-4 h-4 ml-0.5 group-hover:translate-x-0.5 transition-transform' />
                  </Link>
                </div>
                <ScrollableRow>
                  {loading
                    ? Array.from({ length: 8 }).map((_, index) => (
                        <SkeletonCard key={index} />
                      ))
                    : hotVarietyShows.map((show, index) => (
                        <div
                          key={index}
                          className='min-w-24 w-24 sm:min-w-45 sm:w-44'
                        >
                          <VideoCard
                            from='douban'
                            title={show.title}
                            poster={show.poster}
                            douban_id={Number(show.id)}
                            rate={show.rate}
                            year={show.year}
                          />
                        </div>
                      ))}
                </ScrollableRow>
              </section>
            </>
          )}
        </div>
      </div>

      {/* 公告弹窗 - Aurora 风格 */}
      {announcement && showAnnouncement && (
        <div
          className={`fixed inset-0 z-50 flex items-center justify-center p-4 transition-all duration-300 ${
            showAnnouncement ? 'opacity-100' : 'opacity-0 pointer-events-none'
          }`}
          onClick={(e) =>
            e.target === e.currentTarget &&
            handleCloseAnnouncement(announcement)
          }
        >
          {/* 背景遮罩 */}
          <div className='absolute inset-0 bg-black/40 dark:bg-black/60 backdrop-blur-sm' />

          {/* 弹窗内容 */}
          <div className='relative w-full max-w-md animate-fade-slide-up'>
            {/* 玻璃卡片 */}
            <div className='relative rounded-2xl overflow-hidden'>
              {/* 背景 */}
              <div className='absolute inset-0 bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl' />

              {/* 顶部渐变装饰 */}
              <div className='absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-violet-500 via-purple-500 to-fuchsia-500' />

              {/* 内容 */}
              <div className='relative p-6'>
                {/* 头部 */}
                <div className='flex justify-between items-center mb-5'>
                  <h3 className='text-xl font-bold flex items-center gap-2'>
                    <Sparkles className='w-5 h-5 text-purple-500' />
                    <span className='bg-gradient-to-r from-violet-600 via-purple-500 to-fuchsia-500 bg-clip-text text-transparent'>
                      公告
                    </span>
                  </h3>
                  <button
                    onClick={() => handleCloseAnnouncement(announcement)}
                    className='w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:text-gray-200 dark:hover:bg-gray-800 transition-all'
                  >
                    <X className='w-5 h-5' />
                  </button>
                </div>

                {/* 公告内容 */}
                <div className='mb-6 p-4 rounded-xl bg-gradient-to-br from-purple-50 to-fuchsia-50 dark:from-purple-900/20 dark:to-fuchsia-900/20 border border-purple-100 dark:border-purple-800/30'>
                  <p className='text-gray-700 dark:text-gray-200 leading-relaxed'>
                    {announcement}
                  </p>
                </div>

                {/* 按钮 */}
                <button
                  onClick={() => handleCloseAnnouncement(announcement)}
                  className='w-full py-3 rounded-xl font-medium text-white bg-gradient-to-r from-violet-600 via-purple-500 to-fuchsia-500 hover:from-violet-700 hover:via-purple-600 hover:to-fuchsia-600 shadow-lg shadow-purple-500/25 hover:shadow-purple-500/40 transition-all duration-300 active:scale-[0.98]'
                >
                  我知道了
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </PageLayout>
  );
}

export default function Home() {
  return (
    <Suspense>
      <HomeClient />
    </Suspense>
  );
}
