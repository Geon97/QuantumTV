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

export default function AnalyticsDashboard() {
  const [userStats, setUserStats] = useState<UserBehaviorStats | null>(null);
  const [popularItems, setPopularItems] = useState<PopularItem[]>([]);
  const [categoryStats, setCategoryStats] = useState<CategoryStats[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAnalytics();
  }, []);

  const loadAnalytics = async () => {
    setLoading(true);
    try {
      const [stats, popular, categories] = await Promise.all([
        invoke<UserBehaviorStats>('get_user_behavior_stats'),
        invoke<PopularItem[]>('get_popular_items', { limit: 10 }),
        invoke<CategoryStats[]>('get_category_stats'),
      ]);

      setUserStats(stats);
      setPopularItems(popular);
      setCategoryStats(categories);
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

  if (loading) {
    return (
      <div className='flex items-center justify-center min-h-[400px]'>
        <div className='animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500'></div>
      </div>
    );
  }

  return (
    <div className='space-y-6'>
      {/* 核心指标卡片 */}
      {userStats && (
        <div className='grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4'>
          <div className='bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl p-6 border border-blue-200 dark:border-blue-800'>
            <div className='flex items-center gap-3 mb-2'>
              <div className='p-2 bg-blue-500 rounded-lg'>
                <Video className='w-5 h-5 text-white' />
              </div>
              <div className='text-sm text-blue-700 dark:text-blue-300'>
                观看视频
              </div>
            </div>
            <div className='text-3xl font-bold text-blue-900 dark:text-blue-100'>
              {userStats.total_videos_watched}
            </div>
            <div className='text-xs text-blue-600 dark:text-blue-400 mt-1'>
              完成率 {(userStats.completion_rate * 100).toFixed(1)}%
            </div>
          </div>

          <div className='bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-xl p-6 border border-green-200 dark:border-green-800'>
            <div className='flex items-center gap-3 mb-2'>
              <div className='p-2 bg-green-500 rounded-lg'>
                <Clock className='w-5 h-5 text-white' />
              </div>
              <div className='text-sm text-green-700 dark:text-green-300'>
                观看时长
              </div>
            </div>
            <div className='text-3xl font-bold text-green-900 dark:text-green-100'>
              {formatTime(userStats.total_watch_time)}
            </div>
            <div className='text-xs text-green-600 dark:text-green-400 mt-1'>
              平均 {formatTime(userStats.avg_watch_time)}
            </div>
          </div>

          <div className='bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl p-6 border border-purple-200 dark:border-purple-800'>
            <div className='flex items-center gap-3 mb-2'>
              <div className='p-2 bg-purple-500 rounded-lg'>
                <Heart className='w-5 h-5 text-white' />
              </div>
              <div className='text-sm text-purple-700 dark:text-purple-300'>
                收藏
              </div>
            </div>
            <div className='text-3xl font-bold text-purple-900 dark:text-purple-100'>
              {userStats.total_favorites}
            </div>
            <div className='text-xs text-purple-600 dark:text-purple-400 mt-1'>
              精选内容
            </div>
          </div>

          <div className='bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-xl p-6 border border-orange-200 dark:border-orange-800'>
            <div className='flex items-center gap-3 mb-2'>
              <div className='p-2 bg-orange-500 rounded-lg'>
                <Search className='w-5 h-5 text-white' />
              </div>
              <div className='text-sm text-orange-700 dark:text-orange-300'>
                搜索次数
              </div>
            </div>
            <div className='text-3xl font-bold text-orange-900 dark:text-orange-100'>
              {userStats.total_searches}
            </div>
            <div className='text-xs text-orange-600 dark:text-orange-400 mt-1'>
              探索内容
            </div>
          </div>
        </div>
      )}

      {/* 热门内容排行榜 */}
      {popularItems.length > 0 && (
        <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
          <div className='flex items-center gap-3 mb-4'>
            <TrendingUp className='w-6 h-6 text-red-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              热门内容排行榜
            </h2>
          </div>
          <div className='space-y-3'>
            {popularItems.map((item, index) => (
              <div
                key={index}
                className='flex items-center gap-4 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors'
              >
                <div className='flex-shrink-0 w-8 h-8 flex items-center justify-center bg-gradient-to-br from-red-500 to-orange-500 text-white font-bold rounded-full'>
                  {index + 1}
                </div>
                <div className='flex-1 min-w-0'>
                  <div className='font-medium text-gray-900 dark:text-gray-100 truncate'>
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

      {/* 分类统计 */}
      {categoryStats.length > 0 && (
        <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
          <div className='flex items-center gap-3 mb-4'>
            <BarChart3 className='w-6 h-6 text-blue-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              分类偏好
            </h2>
          </div>
          <div className='space-y-4'>
            {categoryStats.map((cat, index) => (
              <div key={index}>
                <div className='flex items-center justify-between mb-2'>
                  <span className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                    {cat.category}
                  </span>
                  <span className='text-sm text-gray-500 dark:text-gray-400'>
                    {cat.watch_count} 次 · {cat.percentage.toFixed(1)}%
                  </span>
                </div>
                <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2'>
                  <div
                    className='bg-gradient-to-r from-blue-500 to-purple-500 h-2 rounded-full transition-all duration-300'
                    style={{ width: `${cat.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 观看习惯 */}
      {userStats?.most_active_hour !== null && (
        <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
          <div className='flex items-center gap-3 mb-4'>
            <Clock className='w-6 h-6 text-green-500' />
            <h2 className='text-xl font-semibold text-gray-900 dark:text-gray-100'>
              观看习惯
            </h2>
          </div>
          <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
            <div className='text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg'>
              <div className='text-2xl font-bold text-green-600 dark:text-green-400'>
                {userStats?.most_active_hour}:00
              </div>
              <div className='text-sm text-gray-600 dark:text-gray-400 mt-1'>
                最活跃时段
              </div>
            </div>
            {userStats?.favorite_category && (
              <div className='text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg'>
                <div className='text-2xl font-bold text-blue-600 dark:text-blue-400'>
                  {userStats.favorite_category}
                </div>
                <div className='text-sm text-gray-600 dark:text-gray-400 mt-1'>
                  最喜欢的类型
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
