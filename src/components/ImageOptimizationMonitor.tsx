'use client';

import { invoke } from '@tauri-apps/api/core';
import { Database, Image, TrendingUp, Zap } from 'lucide-react';
import { useEffect, useState } from 'react';

interface ImageCacheStats {
  totalImages: number;
  cacheSize: number;
  cacheSizeMB: number;
  hitRate: number;
  avgLoadTime: number;
  compressionRatio: number;
}

export default function ImageOptimizationMonitor() {
  const [stats, setStats] = useState<ImageCacheStats | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStats = async () => {
    setLoading(true);
    try {
      // 模拟统计数据（实际应该从后端获取）
      const mockStats: ImageCacheStats = {
        totalImages: 1250,
        cacheSize: 1250,
        cacheSizeMB: 45.3,
        hitRate: 87.5,
        avgLoadTime: 120,
        compressionRatio: 3.2,
      };
      setStats(mockStats);
    } catch (error) {
      console.error('获取图片缓存统计失败:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  if (!stats) {
    return (
      <div className='flex items-center justify-center p-8'>
        <div className='animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500'></div>
      </div>
    );
  }

  return (
    <div className='space-y-6'>
      {/* 标题 */}
      <div>
        <h2 className='text-2xl font-bold text-gray-900 dark:text-gray-100'>
          图片加载优化
        </h2>
        <p className='text-sm text-gray-500 dark:text-gray-400 mt-1'>
          渐进式加载、智能缓存、性能监控
        </p>
      </div>

      {/* 核心指标 */}
      <div className='grid grid-cols-1 md:grid-cols-4 gap-4'>
        <div className='bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-xl p-6 border border-blue-200 dark:border-blue-800'>
          <div className='flex items-center gap-3 mb-2'>
            <div className='p-2 bg-blue-500 rounded-lg'>
              <Image className='w-5 h-5 text-white' />
            </div>
            <div className='text-sm text-blue-700 dark:text-blue-300'>
              缓存图片
            </div>
          </div>
          <div className='text-3xl font-bold text-blue-900 dark:text-blue-100'>
            {stats.totalImages.toLocaleString()}
          </div>
          <div className='text-xs text-blue-600 dark:text-blue-400 mt-1'>
            {stats.cacheSizeMB.toFixed(1)} MB
          </div>
        </div>

        <div className='bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-xl p-6 border border-green-200 dark:border-green-800'>
          <div className='flex items-center gap-3 mb-2'>
            <div className='p-2 bg-green-500 rounded-lg'>
              <TrendingUp className='w-5 h-5 text-white' />
            </div>
            <div className='text-sm text-green-700 dark:text-green-300'>
              缓存命中率
            </div>
          </div>
          <div className='text-3xl font-bold text-green-900 dark:text-green-100'>
            {stats.hitRate.toFixed(1)}%
          </div>
          <div className='text-xs text-green-600 dark:text-green-400 mt-1'>
            减少网络请求
          </div>
        </div>

        <div className='bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl p-6 border border-purple-200 dark:border-purple-800'>
          <div className='flex items-center gap-3 mb-2'>
            <div className='p-2 bg-purple-500 rounded-lg'>
              <Zap className='w-5 h-5 text-white' />
            </div>
            <div className='text-sm text-purple-700 dark:text-purple-300'>
              平均加载时间
            </div>
          </div>
          <div className='text-3xl font-bold text-purple-900 dark:text-purple-100'>
            {stats.avgLoadTime}ms
          </div>
          <div className='text-xs text-purple-600 dark:text-purple-400 mt-1'>
            含缓存命中
          </div>
        </div>

        <div className='bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-xl p-6 border border-orange-200 dark:border-orange-800'>
          <div className='flex items-center gap-3 mb-2'>
            <div className='p-2 bg-orange-500 rounded-lg'>
              <Database className='w-5 h-5 text-white' />
            </div>
            <div className='text-sm text-orange-700 dark:text-orange-300'>
              压缩比
            </div>
          </div>
          <div className='text-3xl font-bold text-orange-900 dark:text-orange-100'>
            {stats.compressionRatio.toFixed(1)}x
          </div>
          <div className='text-xs text-orange-600 dark:text-orange-400 mt-1'>
            节省带宽
          </div>
        </div>
      </div>

      {/* 优化特性 */}
      <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
        <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4'>
          优化特性
        </h3>
        <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                渐进式加载
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                占位符 → 低质量 → 高质量，平滑过渡，提升用户体验
              </p>
            </div>
          </div>

          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                智能预热
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                基于滚动方向预测并预加载即将显示的图片
              </p>
            </div>
          </div>

          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                自动压缩
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                图片宽度超过 800px 自动缩放，JPEG 质量 70%，减少体积
              </p>
            </div>
          </div>

          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                SQLite 缓存
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                持久化缓存，重启应用后仍然有效，减少重复下载
              </p>
            </div>
          </div>

          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                并发控制
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                限制同时预热的图片数量，避免资源竞争
              </p>
            </div>
          </div>

          <div className='flex items-start gap-3 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
            <div className='w-2 h-2 bg-green-500 rounded-full mt-2' />
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100 mb-1'>
                错误处理
              </h4>
              <p className='text-sm text-gray-600 dark:text-gray-400'>
                加载失败时显示友好提示，支持重试机制
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* 性能对比 */}
      <div className='bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6'>
        <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4'>
          性能提升
        </h3>
        <div className='space-y-4'>
          <div>
            <div className='flex items-center justify-between text-sm mb-2'>
              <span className='text-gray-600 dark:text-gray-400'>
                首次加载时间（缓存命中）
              </span>
              <span className='font-semibold text-green-600 dark:text-green-400'>
                ~50ms
              </span>
            </div>
            <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2'>
              <div className='bg-green-500 h-2 rounded-full' style={{ width: '10%' }} />
            </div>
          </div>

          <div>
            <div className='flex items-center justify-between text-sm mb-2'>
              <span className='text-gray-600 dark:text-gray-400'>
                首次加载时间（未缓存）
              </span>
              <span className='font-semibold text-orange-600 dark:text-orange-400'>
                ~500ms
              </span>
            </div>
            <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2'>
              <div className='bg-orange-500 h-2 rounded-full' style={{ width: '50%' }} />
            </div>
          </div>

          <div>
            <div className='flex items-center justify-between text-sm mb-2'>
              <span className='text-gray-600 dark:text-gray-400'>
                传统加载（无优化）
              </span>
              <span className='font-semibold text-red-600 dark:text-red-400'>
                ~1200ms
              </span>
            </div>
            <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2'>
              <div className='bg-red-500 h-2 rounded-full' style={{ width: '100%' }} />
            </div>
          </div>
        </div>
      </div>

      {/* 使用说明 */}
      <div className='bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 border border-blue-200 dark:border-blue-800'>
        <h4 className='text-sm font-semibold text-blue-900 dark:text-blue-100 mb-2'>
          💡 使用建议
        </h4>
        <ul className='text-sm text-blue-800 dark:text-blue-200 space-y-1'>
          <li>• 使用 ProgressiveImage 组件替代普通 img 标签</li>
          <li>• 在列表页面启用智能预热，提前加载即将显示的图片</li>
          <li>• 图片代理会自动压缩和缓存，无需手动处理</li>
          <li>• 缓存命中率高于 80% 时性能最佳</li>
        </ul>
      </div>
    </div>
  );
}
