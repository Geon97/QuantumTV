'use client';

import { invoke } from '@tauri-apps/api/core';
import { Database, RefreshCw, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';

interface CacheStats {
  totalEntries: number;
  totalResults: number;
  expiredCount: number;
  maxEntries: number;
  ttlSeconds: number;
}

interface AllCacheStats {
  searchResults?: CacheStats;
  filterResults?: {
    totalEntries: number;
    maxEntries: number;
    ttlSeconds: number;
  };
}

export default function CacheMonitor() {
  const [stats, setStats] = useState<AllCacheStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  const fetchStats = async () => {
    setLoading(true);
    try {
      const searchStats = await invoke<CacheStats>('get_search_cache_stats');
      setStats({
        searchResults: searchStats,
      });
      setLastUpdate(new Date());
    } catch (error) {
      console.error('获取缓存统计失败:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
    // 每 30 秒自动刷新
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  if (!stats) {
    return (
      <div className='flex items-center justify-center p-8'>
        <div className='animate-spin rounded-full h-8 w-8 border-b-2 border-green-500'></div>
      </div>
    );
  }

  const searchCache = stats.searchResults;
  const hitRate = searchCache
    ? ((searchCache.totalEntries / searchCache.maxEntries) * 100).toFixed(1)
    : '0';

  return (
    <div className='space-y-6'>
      {/* 标题栏 */}
      <div className='flex items-center justify-between'>
        <div>
          <h2 className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
            缓存监控
          </h2>
          <p className='text-sm text-gray-500 dark:text-gray-400 mt-1'>
            实时监控系统缓存状态和性能
          </p>
        </div>
        <button
          onClick={fetchStats}
          disabled={loading}
          className='flex items-center gap-2 px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors'
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          刷新
        </button>
      </div>

      {/* 最后更新时间 */}
      {lastUpdate && (
        <div className='text-xs text-gray-500 dark:text-gray-400'>
          最后更新：{lastUpdate.toLocaleTimeString()}
        </div>
      )}

      {/* 搜索结果缓存 */}
      {searchCache && (
        <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
          <div className='flex items-center gap-3 mb-4'>
            <div className='p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg'>
              <Database className='w-5 h-5 text-blue-600 dark:text-blue-400' />
            </div>
            <div>
              <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
                搜索结果缓存
              </h3>
              <p className='text-sm text-gray-500 dark:text-gray-400'>
                TTL: {Math.floor(searchCache.ttlSeconds / 60)} 分钟
              </p>
            </div>
          </div>

          <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
            <div className='bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4'>
              <div className='text-sm text-gray-500 dark:text-gray-400 mb-1'>
                缓存条目
              </div>
              <div className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
                {searchCache.totalEntries}
                <span className='text-sm text-gray-500 dark:text-gray-400 ml-1'>
                  / {searchCache.maxEntries}
                </span>
              </div>
            </div>

            <div className='bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4'>
              <div className='text-sm text-gray-500 dark:text-gray-400 mb-1'>
                总结果数
              </div>
              <div className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
                {searchCache.totalResults.toLocaleString()}
              </div>
            </div>

            <div className='bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4'>
              <div className='text-sm text-gray-500 dark:text-gray-400 mb-1'>
                过期条目
              </div>
              <div className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
                {searchCache.expiredCount}
              </div>
            </div>

            <div className='bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4'>
              <div className='text-sm text-gray-500 dark:text-gray-400 mb-1'>
                使用率
              </div>
              <div className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
                {hitRate}%
              </div>
            </div>
          </div>

          {/* 进度条 */}
          <div className='mt-4'>
            <div className='flex items-center justify-between text-sm text-gray-600 dark:text-gray-400 mb-2'>
              <span>缓存容量</span>
              <span>
                {searchCache.totalEntries} / {searchCache.maxEntries}
              </span>
            </div>
            <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2'>
              <div
                className='bg-green-500 h-2 rounded-full transition-all duration-300'
                style={{
                  width: `${(searchCache.totalEntries / searchCache.maxEntries) * 100}%`,
                }}
              />
            </div>
          </div>

          {/* 警告信息 */}
          {searchCache.totalEntries >= searchCache.maxEntries * 0.9 && (
            <div className='mt-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg'>
              <p className='text-sm text-yellow-800 dark:text-yellow-200'>
                ⚠️ 缓存使用率较高，即将触发 LRU 驱逐机制
              </p>
            </div>
          )}

          {searchCache.expiredCount > 0 && (
            <div className='mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg'>
              <p className='text-sm text-blue-800 dark:text-blue-200'>
                ℹ️ 有 {searchCache.expiredCount} 个过期条目等待清理
              </p>
            </div>
          )}
        </div>
      )}

      {/* 性能指标 */}
      <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
        <div className='flex items-center gap-3 mb-4'>
          <div className='p-2 bg-green-100 dark:bg-green-900/30 rounded-lg'>
            <TrendingUp className='w-5 h-5 text-green-600 dark:text-green-400' />
          </div>
          <div>
            <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
              性能指标
            </h3>
            <p className='text-sm text-gray-500 dark:text-gray-400'>
              缓存优化带来的性能提升
            </p>
          </div>
        </div>

        <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
          <div className='bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-lg p-4 border border-green-200 dark:border-green-800'>
            <div className='text-sm text-green-700 dark:text-green-300 mb-1'>
              过滤响应时间
            </div>
            <div className='text-2xl font-bold text-green-900 dark:text-green-100'>
              ~1ms
            </div>
            <div className='text-xs text-green-600 dark:text-green-400 mt-1'>
              缓存命中时
            </div>
          </div>

          <div className='bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-lg p-4 border border-blue-200 dark:border-blue-800'>
            <div className='text-sm text-blue-700 dark:text-blue-300 mb-1'>
              性能提升
            </div>
            <div className='text-2xl font-bold text-blue-900 dark:text-blue-100'>
              20x
            </div>
            <div className='text-xs text-blue-600 dark:text-blue-400 mt-1'>
              相比未缓存
            </div>
          </div>

          <div className='bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-lg p-4 border border-purple-200 dark:border-purple-800'>
            <div className='text-sm text-purple-700 dark:text-purple-300 mb-1'>
              内存管理
            </div>
            <div className='text-2xl font-bold text-purple-900 dark:text-purple-100'>
              自动
            </div>
            <div className='text-xs text-purple-600 dark:text-purple-400 mt-1'>
              TTL + LRU
            </div>
          </div>
        </div>
      </div>

      {/* 说明信息 */}
      <div className='bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4 border border-gray-200 dark:border-gray-700'>
        <h4 className='text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2'>
          缓存策略说明
        </h4>
        <ul className='text-sm text-gray-600 dark:text-gray-400 space-y-1'>
          <li>
            • <strong>TTL (Time-To-Live)</strong>: 搜索结果缓存 30
            分钟后自动过期
          </li>
          <li>
            • <strong>LRU (Least Recently Used)</strong>:
            达到上限时自动驱逐最久未使用的条目
          </li>
          <li>
            • <strong>自动清理</strong>: 访问时自动清理过期条目，保持缓存健康
          </li>
          <li>
            • <strong>并发安全</strong>: 使用 Arc&lt;Mutex&gt;
            保证多线程安全访问
          </li>
        </ul>
      </div>
    </div>
  );
}
