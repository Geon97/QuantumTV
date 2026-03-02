/* eslint-disable no-console */
'use client';

import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

import { ContinueWatchingCard } from '@/lib/types';
import { getRailItemClass } from '@/lib/ui-layout';
import { subscribeToDataUpdates } from '@/lib/utils';

import ScrollableRow from '@/components/ScrollableRow';
import VideoCard from '@/components/VideoCard';

interface ContinueWatchingProps {
  className?: string;
}

export default function ContinueWatching({ className }: ContinueWatchingProps) {
  const [playRecords, setPlayRecords] = useState<ContinueWatchingCard[]>([]);
  const [loading, setLoading] = useState(true);

  // 处理播放记录数据更新的函数

  useEffect(() => {
    const fetchPlayRecords = async () => {
      try {
        setLoading(true);

        // 从 Rust 获取所有播放记录
        const allRecords = await invoke<ContinueWatchingCard[]>(
          'get_continue_watching',
        );
        setPlayRecords(allRecords);
      } catch (error) {
        console.error('获取播放记录失败:', error);
        setPlayRecords([]);
      } finally {
        setLoading(false);
      }
    };

    fetchPlayRecords();

    // 监听播放记录更新事件
    const unsubscribe = subscribeToDataUpdates(
      'playRecordsUpdated',
      async () => {
        try {
          const allRecords = await invoke<ContinueWatchingCard[]>(
            'get_continue_watching',
          );
          setPlayRecords(allRecords);
        } catch (err) {
          console.error('获取播放记录失败:', err);
        }
      },
    );

    return unsubscribe;
  }, []);

  // 如果没有播放记录，则不渲染组件
  if (!loading && playRecords.length === 0) {
    return null;
  }

  return (
    <section
      className={`mb-8 max-[375px]:mb-7 min-[834px]:mb-10 ${className || ''}`}
    >
      <div className='mb-4 flex items-center justify-between'>
        <h2 className='text-lg font-bold text-gray-800 max-[375px]:text-base min-[834px]:text-[1.35rem] min-[1440px]:text-[1.5rem] dark:text-gray-200'>
          继续观看
        </h2>
        {!loading && playRecords.length > 0 && (
          <button
            className='tap-target px-2 text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            onClick={async () => {
              await invoke('clear_all_play_records');
              setPlayRecords([]);
            }}
          >
            清空
          </button>
        )}
      </div>
      <ScrollableRow>
        {loading
          ? // 加载状态显示灰色占位数据
            Array.from({ length: 6 }).map((_, index) => (
              <div key={index} className={getRailItemClass('default')}>
                <div className='relative aspect-2/3 w-full overflow-hidden rounded-lg bg-gray-200 animate-pulse dark:bg-gray-800'>
                  <div className='absolute inset-0 bg-gray-300 dark:bg-gray-700'></div>
                </div>
                <div className='mt-2 h-4 bg-gray-200 rounded animate-pulse dark:bg-gray-800'></div>
                <div className='mt-1 h-3 bg-gray-200 rounded animate-pulse dark:bg-gray-800'></div>
              </div>
            ))
          : // 显示真实数据
            playRecords.map((record) => {
              const { key: recordKey, ...cardProps } = record;
              return (
                <div key={recordKey} className={getRailItemClass('default')}>
                  <VideoCard
                    {...cardProps}
                    from='playrecord'
                    onDelete={() =>
                      setPlayRecords((prev) =>
                        prev.filter((r) => r.key !== recordKey),
                      )
                    }
                  />
                </div>
              );
            })}
      </ScrollableRow>
    </section>
  );
}
