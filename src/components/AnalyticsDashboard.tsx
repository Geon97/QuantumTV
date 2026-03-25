'use client';

import { invoke } from '@tauri-apps/api/core';
import {
  BarChart3,
  Clock,
  Heart,
  Search,
  TrendingUp,
  Video,
} from 'lucide-react';
import { useEffect, useState } from 'react';

interface UserBehaviorStats {
  total_watch_time: number;
  total_videos_watched: number;
  total_favorites: number;
  total_searches: number;
  avg_watch_time: number;
  completion_rate: number;
  most_active_hour: number | null;
  favorite_category: string | null;
}

interface PopularItem {
  title: string;
  source_name: string;
  year: string;
  cover: string;
  play_count: number;
  favorite_count: number;
  popularity_score: number;
}

interface CategoryStats {
  category: string;
  watch_count: number;
  watch_duration: number;
  percentage: number;
}

interface RecentFavoriteInsight {
  title: string;
  source_name: string;
  year: string;
  cover: string;
  save_time: number;
}

interface WatchInsights {
  preferred_time_slot: string | null;
  streak_days: number;
  recent_favorites: RecentFavoriteInsight[];
}

export default function AnalyticsDashboard() {
  const [userStats, setUserStats] = useState<UserBehaviorStats | null>(null);
  const [popularItems, setPopularItems] = useState<PopularItem[]>([]);
  const [categoryStats, setCategoryStats] = useState<CategoryStats[]>([]);
  const [watchInsights, setWatchInsights] = useState<WatchInsights | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void loadAnalytics();
  }, []);

  const loadAnalytics = async () => {
    setLoading(true);
    try {
      await invoke('clear_analytics_cache');

      const [stats, popular, categories, insights] = await Promise.all([
        invoke<UserBehaviorStats>('get_user_behavior_stats'),
        invoke<PopularItem[]>('get_popular_items', { limit: 10 }),
        invoke<CategoryStats[]>('get_category_stats'),
        invoke<WatchInsights>('get_watch_insights'),
      ]);

      setUserStats(stats);
      setPopularItems(popular);
      setCategoryStats(categories);
      setWatchInsights(insights);
    } catch (error) {
      console.error('加载统计数据失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const formatTime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
      return `${hours}小时${minutes}分钟`;
    }

    return `${minutes}分钟`;
  };

  const formatFavoriteDate = (timestamp: number) =>
    new Date(timestamp * 1000).toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
    });

  if (loading) {
    return (
      <div className='flex items-center justify-center min-h-[400px]'>
        <div className='animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500'></div>
      </div>
    );
  }

  return (
    <div className='space-y-6'>
      {userStats && (
        <div className='grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <div className='rounded-xl border border-blue-200 bg-gradient-to-br from-blue-50 to-blue-100 p-6 dark:border-blue-800 dark:from-blue-900/20 dark:to-blue-800/20'>
            <div className='mb-2 flex items-center gap-3'>
              <div className='rounded-lg bg-blue-500 p-2'>
                <Video className='h-5 w-5 text-white' />
              </div>
              <div className='text-sm text-blue-700 dark:text-blue-300'>
                观看视频
              </div>
            </div>
            <div className='text-3xl font-bold text-blue-900 dark:text-blue-100'>
              {userStats.total_videos_watched}
            </div>
            <div className='mt-1 text-xs text-blue-600 dark:text-blue-400'>
              完成率 {(userStats.completion_rate * 100).toFixed(1)}%
            </div>
          </div>

          <div className='rounded-xl border border-green-200 bg-gradient-to-br from-green-50 to-green-100 p-6 dark:border-green-800 dark:from-green-900/20 dark:to-green-800/20'>
            <div className='mb-2 flex items-center gap-3'>
              <div className='rounded-lg bg-green-500 p-2'>
                <Clock className='h-5 w-5 text-white' />
              </div>
              <div className='text-sm text-green-700 dark:text-green-300'>
                观看时长
              </div>
            </div>
            <div className='text-3xl font-bold text-green-900 dark:text-green-100'>
              {formatTime(userStats.total_watch_time)}
            </div>
            <div className='mt-1 text-xs text-green-600 dark:text-green-400'>
              平均 {formatTime(userStats.avg_watch_time)}
            </div>
          </div>

          <div className='rounded-xl border border-purple-200 bg-gradient-to-br from-purple-50 to-purple-100 p-6 dark:border-purple-800 dark:from-purple-900/20 dark:to-purple-800/20'>
            <div className='mb-2 flex items-center gap-3'>
              <div className='rounded-lg bg-purple-500 p-2'>
                <Heart className='h-5 w-5 text-white' />
              </div>
              <div className='text-sm text-purple-700 dark:text-purple-300'>
                收藏
              </div>
            </div>
            <div className='text-3xl font-bold text-purple-900 dark:text-purple-100'>
              {userStats.total_favorites}
            </div>
            <div className='mt-1 text-xs text-purple-600 dark:text-purple-400'>
              已保存内容
            </div>
          </div>

          <div className='rounded-xl border border-orange-200 bg-gradient-to-br from-orange-50 to-orange-100 p-6 dark:border-orange-800 dark:from-orange-900/20 dark:to-orange-800/20'>
            <div className='mb-2 flex items-center gap-3'>
              <div className='rounded-lg bg-orange-500 p-2'>
                <Search className='h-5 w-5 text-white' />
              </div>
              <div className='text-sm text-orange-700 dark:text-orange-300'>
                搜索次数
              </div>
            </div>
            <div className='text-3xl font-bold text-orange-900 dark:text-orange-100'>
              {userStats.total_searches}
            </div>
            <div className='mt-1 text-xs text-orange-600 dark:text-orange-400'>
              探索内容
            </div>
          </div>
        </div>
      )}

      {watchInsights && (
        <div className='rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800'>
          <div className='mb-4 flex items-center gap-3'>
            <Clock className='h-6 w-6 text-green-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              观看洞察
            </h2>
          </div>
          <div className='grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]'>
            <div className='grid grid-cols-1 gap-4 sm:grid-cols-2'>
              <div className='rounded-lg bg-green-50 p-4 dark:bg-green-900/20'>
                <div className='text-sm text-gray-600 dark:text-gray-400'>
                  常看时间段
                </div>
                <div className='mt-2 text-2xl font-bold text-green-600 dark:text-green-400'>
                  {watchInsights.preferred_time_slot || '暂无数据'}
                </div>
              </div>
              <div className='rounded-lg bg-blue-50 p-4 dark:bg-blue-900/20'>
                <div className='text-sm text-gray-600 dark:text-gray-400'>
                  连续追剧天数
                </div>
                <div className='mt-2 text-2xl font-bold text-blue-600 dark:text-blue-400'>
                  {watchInsights.streak_days} 天
                </div>
              </div>
            </div>

            <div className='rounded-lg border border-gray-200 p-4 dark:border-gray-700'>
              <div className='mb-3 flex items-center justify-between'>
                <div className='text-sm font-medium text-gray-900 dark:text-gray-100'>
                  最近新增收藏
                </div>
                <Heart className='h-4 w-4 text-pink-500' />
              </div>
              {watchInsights.recent_favorites.length > 0 ? (
                <div className='space-y-3'>
                  {watchInsights.recent_favorites.map((item, index) => (
                    <div
                      key={`${item.title}-${item.source_name}-${index}`}
                      className='flex items-center justify-between gap-3'
                    >
                      <div className='min-w-0'>
                        <div className='truncate text-sm font-medium text-gray-900 dark:text-gray-100'>
                          {item.title}
                        </div>
                        <div className='text-xs text-gray-500 dark:text-gray-400'>
                          {item.source_name}
                          {item.year ? ` · ${item.year}` : ''}
                        </div>
                      </div>
                      <div className='shrink-0 text-xs text-gray-500 dark:text-gray-400'>
                        {formatFavoriteDate(item.save_time)}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className='text-sm text-gray-500 dark:text-gray-400'>
                  暂无收藏数据
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {popularItems.length > 0 && (
        <div className='rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800'>
          <div className='mb-4 flex items-center gap-3'>
            <TrendingUp className='h-6 w-6 text-red-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              热门内容排行
            </h2>
          </div>
          <div className='space-y-3'>
            {popularItems.map((item, index) => (
              <div
                key={index}
                className='flex items-center gap-4 rounded-lg bg-gray-50 p-3 transition-colors hover:bg-gray-100 dark:bg-gray-700/50 dark:hover:bg-gray-700'
              >
                <div className='flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-red-500 to-orange-500 font-bold text-white'>
                  {index + 1}
                </div>
                <div className='min-w-0 flex-1'>
                  <div className='truncate font-medium text-gray-900 dark:text-gray-100'>
                    {item.title}
                  </div>
                  <div className='text-sm text-gray-500 dark:text-gray-400'>
                    {item.source_name} · {item.year}
                  </div>
                </div>
                <div className='text-right'>
                  <div className='text-sm font-medium text-gray-900 dark:text-gray-100'>
                    播放 {item.play_count} 次
                  </div>
                  <div className='text-xs text-gray-500 dark:text-gray-400'>
                    收藏 {item.favorite_count}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {categoryStats.length > 0 && (
        <div className='rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800'>
          <div className='mb-4 flex items-center gap-3'>
            <BarChart3 className='h-6 w-6 text-blue-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              分类偏好
            </h2>
          </div>
          <div className='space-y-4'>
            {categoryStats.map((cat, index) => (
              <div key={index}>
                <div className='mb-2 flex items-center justify-between'>
                  <span className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                    {cat.category}
                  </span>
                  <span className='text-sm text-gray-500 dark:text-gray-400'>
                    {cat.watch_count} 次 · {cat.percentage.toFixed(1)}%
                  </span>
                </div>
                <div className='h-2 w-full rounded-full bg-gray-200 dark:bg-gray-700'>
                  <div
                    className='h-2 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-300'
                    style={{ width: `${cat.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
