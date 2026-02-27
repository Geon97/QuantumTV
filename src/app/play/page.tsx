/* eslint-disable @typescript-eslint/no-explicit-any, react-hooks/exhaustive-deps, no-console, @next/next/no-img-element */

'use client';
import { invoke } from '@tauri-apps/api/core';
import Hls from 'hls.js';
import { Heart } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import * as Plyr from 'plyr';
import { Suspense, useEffect, useRef, useState } from 'react';

import {
  ApplySkipConfigResponse,
  ChangePlaySourceResponse,
  InitializePlayerByQueryResponse,
  PlayerInitialState,
  RustFavorite,
  SearchResult,
  SkipAction,
} from '@/lib/types';
import { appLayoutClasses } from '@/lib/ui-layout';
import { generateStorageKey, subscribeToDataUpdates } from '@/lib/utils';
import { useProxyImage } from '@/hooks/useProxyImage';

import EpisodeSelector from '@/components/EpisodeSelector';
import PageLayout from '@/components/PageLayout';
import SkipConfigPanel from '@/components/SkipConfigPanel';
import Toast from '@/components/Toast';

// 扩展 HTMLVideoElement 类型以支持 hls 属性
declare global {
  interface HTMLVideoElement {
    hls?: any;
  }
}

// Wake Lock API 类型声明
interface WakeLockSentinel {
  released: boolean;
  release(): Promise<void>;
  addEventListener(type: 'release', listener: () => void): void;
  removeEventListener(type: 'release', listener: () => void): void;
}

function PlayPageClient() {
  const router = useRouter();
  const searchParams = useSearchParams();

  // -----------------------------------------------------------------------------
  // 状态变量（State）
  // -----------------------------------------------------------------------------
  const [loading, setLoading] = useState(true);
  const [loadingStage, setLoadingStage] = useState<
    'searching' | 'preferring' | 'fetching' | 'ready'
  >('searching');
  const [loadingMessage, setLoadingMessage] = useState('正在搜索播放源...');
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<SearchResult | null>(null);

  // 收藏状态
  const [favorited, setFavorited] = useState(false);

  // 跳过片头片尾配置
  const [skipConfig, setSkipConfig] = useState<{
    enable: boolean;
    intro_time: number;
    outro_time: number;
  }>({
    enable: false,
    intro_time: 0,
    outro_time: 0,
  });
  const skipConfigRef = useRef(skipConfig);
  useEffect(() => {
    skipConfigRef.current = skipConfig;
  }, [
    skipConfig,
    skipConfig.enable,
    skipConfig.intro_time,
    skipConfig.outro_time,
  ]);

  // 跳过检查的时间间隔控制
  const lastSkipCheckRef = useRef(0);

  // 去广告开关（从 Rust 配置读取，默认 true）
  const [blockAdEnabled, setBlockAdEnabled] = useState<boolean>(true);
  const blockAdEnabledRef = useRef(blockAdEnabled);
  useEffect(() => {
    blockAdEnabledRef.current = blockAdEnabled;
  }, [blockAdEnabled]);

  // 视频基本信息
  const [videoTitle, setVideoTitle] = useState(searchParams.get('title') || '');
  const [videoYear, setVideoYear] = useState(searchParams.get('year') || '');
  const [videoCover, setVideoCover] = useState('');
  const [videoDoubanId, setVideoDoubanId] = useState(0);

  // 使用 Tauri proxy_image 命令加载封面图片
  const { url: proxiedCoverUrl } = useProxyImage(videoCover);
  // 当前源和ID
  const [currentSource, setCurrentSource] = useState(
    searchParams.get('source') || '',
  );
  const [currentId, setCurrentId] = useState(searchParams.get('id') || '');

  // 搜索所需信息
  const [searchTitle] = useState(searchParams.get('stitle') || '');
  const [searchType] = useState(searchParams.get('stype') || '');

  // 是否需要优选
  const [needPrefer, setNeedPrefer] = useState(
    searchParams.get('prefer') === 'true',
  );
  const needPreferRef = useRef(needPrefer);
  useEffect(() => {
    needPreferRef.current = needPrefer;
  }, [needPrefer]);
  // 集数相关
  const [currentEpisodeIndex, setCurrentEpisodeIndex] = useState(0);

  const currentSourceRef = useRef(currentSource);
  const currentIdRef = useRef(currentId);
  const videoTitleRef = useRef(videoTitle);
  const videoYearRef = useRef(videoYear);
  const detailRef = useRef<SearchResult | null>(detail);
  const currentEpisodeIndexRef = useRef(currentEpisodeIndex);

  // 同步最新值到 refs
  useEffect(() => {
    currentSourceRef.current = currentSource;
    currentIdRef.current = currentId;
    detailRef.current = detail;
    currentEpisodeIndexRef.current = currentEpisodeIndex;
    videoTitleRef.current = videoTitle;
    videoYearRef.current = videoYear;
  }, [
    currentSource,
    currentId,
    detail,
    currentEpisodeIndex,
    videoTitle,
    videoYear,
  ]);

  // 视频播放地址
  const [videoUrl, setVideoUrl] = useState('');

  // 总集数
  const totalEpisodes = detail?.episodes?.length || 0;

  // 用于记录是否需要在播放器 ready 后跳转到指定进度
  const resumeTimeRef = useRef<number | null>(null);
  // 上次使用的音量，默认 0.7
  const lastVolumeRef = useRef<number>(0.7);
  // 上次使用的播放速率，默认 1.0
  const lastPlaybackRateRef = useRef<number>(1.0);

  // 换源相关状态
  const [availableSources, setAvailableSources] = useState<SearchResult[]>([]);
  const [sourceSearchLoading, setSourceSearchLoading] = useState(false);
  const [sourceSearchError, setSourceSearchError] = useState<string | null>(
    null,
  );

  // 优选和测速开关（从 Rust 配置读取，默认 true）
  const [optimizationEnabled, setOptimizationEnabled] = useState<boolean>(true);

  // 保存优选时的测速结果，避免EpisodeSelector重复测速
  const [precomputedVideoInfo, setPrecomputedVideoInfo] = useState<
    Map<string, { quality: string; loadSpeed: string; pingTime: number }>
  >(new Map());

  // 折叠状态（仅在 lg 及以上屏幕有效）
  const [isEpisodeSelectorCollapsed, setIsEpisodeSelectorCollapsed] =
    useState(false);

  // 跳过片头片尾设置面板状态
  const [isSkipConfigPanelOpen, setIsSkipConfigPanelOpen] = useState(false);

  // Toast 通知状态
  const [toast, setToast] = useState<{
    show: boolean;
    message: string;
    type: 'success' | 'error' | 'info';
  }>({
    show: false,
    message: '',
    type: 'info',
  });

  // 显示 Toast 通知
  const showToast = (
    message: string,
    type: 'success' | 'error' | 'info' = 'info',
  ) => {
    setToast({ show: true, message, type });
  };

  // 换源加载状态
  const [isVideoLoading, setIsVideoLoading] = useState(true);
  const [videoLoadingStage, setVideoLoadingStage] = useState<
    'initing' | 'sourceChanging'
  >('initing');

  // 播放进度保存相关
  const saveIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastSaveTimeRef = useRef<number>(0);

  const plyrRef = useRef<Plyr | null>(null);
  const videoElementRef = useRef<HTMLVideoElement | null>(null);
  const playerContainerRef = useRef<HTMLDivElement | null>(null);
  const hlsRef = useRef<Hls | null>(null);

  // Wake Lock 相关
  const wakeLockRef = useRef<WakeLockSentinel | null>(null);

  // -----------------------------------------------------------------------------
  // 工具函数（Utils）

  // 更新视频地址
  const updateVideoUrl = (
    detailData: SearchResult | null,
    episodeIndex: number,
  ) => {
    if (
      !detailData ||
      !detailData.episodes ||
      episodeIndex >= detailData.episodes.length
    ) {
      setVideoUrl('');
      return;
    }
    const newUrl = detailData?.episodes[episodeIndex] || '';
    if (newUrl !== videoUrl) {
      setVideoUrl(newUrl);
    }
  };

  // 确保视频源
  const ensureVideoSource = (video: HTMLVideoElement | null, url: string) => {
    if (!video || !url) return;
    const sources = Array.from(video.getElementsByTagName('source'));
    const existed = sources.some((s) => s.src === url);
    if (!existed) {
      // 移除旧的 source，保持唯一
      sources.forEach((s) => s.remove());
      const sourceEl = document.createElement('source');
      sourceEl.src = url;
      video.appendChild(sourceEl);
    }

    // 始终允许远程播放（AirPlay / Cast）
    video.disableRemotePlayback = false;
    // 如果曾经有禁用属性，移除之
    if (video.hasAttribute('disableRemotePlayback')) {
      video.removeAttribute('disableRemotePlayback');
    }
  };

  // Wake Lock 相关函数
  const requestWakeLock = async () => {
    try {
      if ('wakeLock' in navigator) {
        wakeLockRef.current = await (navigator as any).wakeLock.request(
          'screen',
        );
        console.log('Wake Lock 已启用');
      }
    } catch (err) {
      console.warn('Wake Lock 请求失败:', err);
    }
  };

  const releaseWakeLock = async () => {
    try {
      if (wakeLockRef.current) {
        await wakeLockRef.current.release();
        wakeLockRef.current = null;
        console.log('Wake Lock 已释放');
      }
    } catch (err) {
      console.warn('Wake Lock 释放失败:', err);
    }
  };

  // 清理播放器资源的统一函数
  const cleanupPlayer = () => {
    try {
      if (hlsRef.current) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }

      if (videoElementRef.current?.hls) {
        videoElementRef.current.hls.destroy();
        delete videoElementRef.current.hls;
      }

      if (plyrRef.current) {
        plyrRef.current.destroy();
        plyrRef.current = null;
      }

      if (playerContainerRef.current) {
        playerContainerRef.current.innerHTML = '';
      }

      videoElementRef.current = null;
      console.log('清理播放器资源');
    } catch (err) {
      console.warn('清理播放器资源失败:', err);
      plyrRef.current = null;
      hlsRef.current = null;
      videoElementRef.current = null;
    }
  };

  // 跳过片头片尾配置相关函数
  const handleSkipConfigChange = async (newConfig: {
    enable: boolean;
    intro_time: number;
    outro_time: number;
  }) => {
    if (!currentSourceRef.current || !currentIdRef.current) return;

    console.log('[跳过配置] 更新配置', {
      old: skipConfigRef.current,
      new: newConfig,
    });

    try {
      setSkipConfig(newConfig);
      // 立即更新 ref，确保 timeupdate 事件处理器使用最新值
      skipConfigRef.current = newConfig;

      console.log('[跳过配置] 更新 ref', skipConfigRef.current);

      const response = await invoke<ApplySkipConfigResponse>(
        'apply_skip_config',
        {
          request: {
            source: currentSourceRef.current,
            id: currentIdRef.current,
            enable: newConfig.enable,
            intro_time: newConfig.intro_time,
            outro_time: newConfig.outro_time,
          },
        },
      );

      if (response.deleted) {
        showToast('已清除跳过设置', 'info');
      } else {
        const introText =
          newConfig.intro_time > 0
            ? `片头: ${formatTime(newConfig.intro_time)}`
            : '';
        const outroText =
          newConfig.outro_time < 0
            ? `片尾: ${formatTime(Math.abs(newConfig.outro_time))}`
            : '';
        const separator = introText && outroText ? '\n' : '';
        const message = newConfig.enable
          ? `已设置跳过配置：${introText}${separator}${outroText}`
          : '已取消跳过配置';

        showToast(message, 'success');
      }
      console.log('[跳过配置] 更新配置', newConfig);
    } catch (err) {
      console.error('[跳过配置] 更新配置失败:', err);
      showToast('更新跳过配置失败', 'error');
    }
  };

  const formatTime = (seconds: number): string => {
    if (seconds === 0) return '00:00';

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = Math.round(seconds % 60);

    if (hours === 0) {
      // 不到一小时，格式为 00:00
      return `${minutes.toString().padStart(2, '0')}:${remainingSeconds
        .toString()
        .padStart(2, '0')}`;
    } else {
      // 超过一小时，格式为 00:00:00
      return `${hours.toString().padStart(2, '0')}:${minutes
        .toString()
        .padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
    }
  };

  // 使用 Tauri fetch_binary 的 HLS.js Loader（带缓存和预取）
  class TauriHlsJsLoader {
    context: any;
    config: any;
    callbacks: any;
    stats: any;
    enableAdBlock: boolean;

    constructor(config: any) {
      this.config = config;
      this.enableAdBlock = config.enableAdBlock || false;

      console.log('[TauriHlsJsLoader] 初始化', {
        enableAdBlock: this.enableAdBlock,
      });

      // 在构造函数中立即初始化 stats
      this.stats = {
        aborted: false,
        loaded: 0,
        retry: 0,
        total: 0,
        chunkCount: 0,
        bwEstimate: 0,
        loading: { start: 0, first: 0, end: 0 },
        parsing: { start: 0, end: 0 },
        buffering: { start: 0, first: 0, end: 0 },
      };
    }

    destroy() {
      this.callbacks = null;
      this.config = null;
      this.stats = null;
      this.context = null;
    }

    abort() {
      if (this.stats) {
        this.stats.aborted = true;
      }
    }

    load(context: any, config: any, callbacks: any) {
      this.context = context;
      this.callbacks = callbacks;

      // 确保 stats 存在（以防万一）
      if (this.stats) {
        this.stats.loading.start = performance.now();
        this.stats.loading.first = 0;
        this.stats.loading.end = 0;
      }

      const { url } = context;

      // 对于 M3U8 manifest 和 level，使用 Rust 端的 fetch_m3u8 命令（支持去广告）
      if (context.type === 'manifest' || context.type === 'level') {
        console.log('[TauriHlsJsLoader] 加载M3U8', {
          url,
          type: context.type,
          enableAdBlock: this.enableAdBlock,
        });

        invoke<string>('fetch_m3u8', {
          url,
          enableAdBlock: this.enableAdBlock,
          headersOpt: null,
        })
          .then((m3u8Content) => {
            // 先检查 this.stats 是否为 null (即 loader 是否已被销毁)
            if (!this.stats || this.stats.aborted) return;

            this.stats.loading.end = performance.now();
            this.stats.loading.first = this.stats.loading.start;

            // M3U8 内容已经在 Rust 端处理完成（包括去广告）
            const textBytes = new TextEncoder().encode(m3u8Content);
            this.stats.loaded = textBytes.byteLength;
            this.stats.total = textBytes.byteLength;

            const response = {
              url,
              data: m3u8Content,
            };

            callbacks.onSuccess(response, this.stats, context);
          })
          .catch((error) => {
            // 同样在错误处理中检查 this.stats 是否存在
            if (!this.stats || this.stats.aborted) return;

            callbacks.onError({ code: 0, text: error.toString() }, context);
          });
      } else {
        // 对于 TS 分片等二进制内容，继续使用 fetch_binary
        invoke<{ status: number; body: number[] }>('fetch_binary', {
          url,
          method: 'GET',
          headersOpt: null,
        })
          .then((result) => {
            // 先检查 this.stats 是否为 null (即 loader 是否已被销毁)
            if (!this.stats || this.stats.aborted) return;

            this.stats.loading.end = performance.now();
            this.stats.loading.first = this.stats.loading.start;

            const data = new Uint8Array(result.body);
            this.stats.loaded = data.byteLength;
            this.stats.total = data.byteLength;
            const duration = this.stats.loading.end - this.stats.loading.start;
            this.stats.bwEstimate =
              duration > 0
                ? (this.stats.loaded * 8) / 1000 / 1000 / (duration / 1000)
                : 0;

            const response = {
              url,
              data: data.buffer,
            };

            callbacks.onSuccess(response, this.stats, context);
          })
          .catch((error) => {
            // 同样在错误处理中检查 this.stats 是否存在
            if (!this.stats || this.stats.aborted) return;

            callbacks.onError({ code: 0, text: error.toString() }, context);
          });
      }
    }
  }
  // 当集数索引变化时自动更新视频地址
  useEffect(() => {
    updateVideoUrl(detail, currentEpisodeIndex);
  }, [detail, currentEpisodeIndex]);

  // 进入页面时直接获取全部源信息
  useEffect(() => {
    const initAll = async () => {
      if (!currentSource && !currentId && !videoTitle && !searchTitle) {
        setError('缺少必要参数');
        setLoading(false);
        return;
      }

      setLoading(true);

      // 如果指定了 source 和 id，使用聚合初始化命令
      if (currentSource && currentId && !needPreferRef.current) {
        setLoadingStage('fetching');
        setLoadingMessage('🎬 正在初始化播放器...');

        try {
          const initialState = await invoke<PlayerInitialState>(
            'initialize_player_view',
            {
              source: currentSource,
              id: currentId,
              title: videoTitle || searchTitle,
            },
          );

          const detailData = initialState.detail;
          // 设置视频详情
          setNeedPrefer(false);
          setCurrentSource(detailData.source);
          setCurrentId(detailData.id);
          setVideoYear(detailData.year);
          setVideoTitle(detailData.title || videoTitleRef.current);
          setVideoCover(detailData.poster);
          setVideoDoubanId(detailData.douban_id || 0);
          setDetail(detailData);
          // 恢复播放记录
          setCurrentEpisodeIndex(initialState.initial_episode_index);
          resumeTimeRef.current = initialState.resume_time ?? null;

          // 设置收藏状态
          setFavorited(initialState.is_favorited);

          // 设置跳过配置
          if (initialState.skip_config) {
            setSkipConfig({
              enable: initialState.skip_config.enable,
              intro_time: initialState.skip_config.intro_time,
              outro_time: initialState.skip_config.outro_time,
            });
          }

          // 设置播放器配置
          setBlockAdEnabled(initialState.block_ad_enabled);
          setOptimizationEnabled(initialState.optimization_enabled);

          // 输出缓存统计信息
          invoke<
            Record<string, { entry_count: number; weighted_size: number }>
          >('get_cache_stats')
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

          // 更新 URL
          const newUrl = new URL(window.location.href);
          newUrl.searchParams.set('source', detailData.source);
          newUrl.searchParams.set('id', detailData.id);
          newUrl.searchParams.set('year', detailData.year);
          newUrl.searchParams.set('title', detailData.title);
          newUrl.searchParams.delete('prefer');
          window.history.replaceState({}, '', newUrl.toString());

          // 设置可用源
          setAvailableSources([detailData, ...initialState.other_sources]);

          setLoadingStage('ready');
          setLoadingMessage('✨ 准备就绪，即将开始播放...');
          setTimeout(() => setLoading(false), 1000);

          return;
        } catch (err) {
          console.error('初始化播放器失败:', err);
          setError('初始化播放器失败');
          setLoading(false);
          return;
        }
      }

      // 处理无 source/id 的情况 - 进行搜索
      const searchQuery = searchTitle || videoTitle;
      try {
        setLoadingStage('searching');
        setLoadingMessage('🔍 正在搜索播放源...');
        setSourceSearchLoading(true);
        setSourceSearchError(null);

        const response = await invoke<InitializePlayerByQueryResponse>(
          'initialize_player_by_query',
          {
            request: {
              query: searchQuery,
              filterTitle: videoTitleRef.current,
              year: videoYearRef.current || null,
              searchType: searchType || null,
              preferBest: optimizationEnabled,
            },
          },
        );

        if (!response.results || response.results.length === 0) {
          setError('未找到匹配结果');
          setLoading(false);
          return;
        }

        const detailData = response.results[0];

        setNeedPrefer(false);
        setCurrentSource(detailData.source);
        setCurrentId(detailData.id);
        setVideoYear(detailData.year);
        setVideoTitle(detailData.title || videoTitleRef.current);
        setVideoCover(detailData.poster);
        setVideoDoubanId(detailData.douban_id || 0);
        setDetail(detailData);
        if (currentEpisodeIndex >= detailData.episodes.length) {
          setCurrentEpisodeIndex(0);
        }

        const newUrl = new URL(window.location.href);
        newUrl.searchParams.set('source', detailData.source);
        newUrl.searchParams.set('id', detailData.id);
        newUrl.searchParams.set('year', detailData.year);
        newUrl.searchParams.set('title', detailData.title);
        newUrl.searchParams.delete('prefer');
        window.history.replaceState({}, '', newUrl.toString());

        setAvailableSources(response.results);

        if (response.test_results.length > 0) {
          const newVideoInfoMap = new Map<
            string,
            {
              quality: string;
              loadSpeed: string;
              pingTime: number;
              hasError?: boolean;
            }
          >();

          response.test_results.forEach(([key, result]) => {
            newVideoInfoMap.set(key, {
              quality: result.quality,
              loadSpeed: result.load_speed,
              pingTime: result.ping_time,
              hasError: result.has_error,
            });
          });

          setPrecomputedVideoInfo(newVideoInfoMap);
        }

        setLoadingStage('ready');
        setLoadingMessage('加载完成，正在准备播放...');
        setTimeout(() => setLoading(false), 1000);
      } catch (err) {
        console.error('加载失败:', err);
        setError('加载失败');
        setLoading(false);
      } finally {
        setSourceSearchLoading(false);
      }
    };

    initAll();
  }, []);

  // 处理换源
  const handleSourceChange = async (
    newSource: string,
    newId: string,
    newTitle: string,
  ) => {
    try {
      // 显示换源加载状态
      setVideoLoadingStage('sourceChanging');
      setIsVideoLoading(true);
      // 记录当前播放进度（仅在同一集数切换时恢复）
      const currentPlayTime = plyrRef.current?.currentTime || 0;
      console.log('换源前当前播放时间:', currentPlayTime);

      const response = await invoke<ChangePlaySourceResponse>(
        'change_play_source',
        {
          request: {
            currentSource: currentSourceRef.current || null,
            currentId: currentIdRef.current || null,
            newSource,
            newId,
            availableSources,
            currentEpisodeIndex: currentEpisodeIndexRef.current,
            currentPlayTime,
            resumeTime: resumeTimeRef.current ?? 0,
            skipConfig: skipConfigRef.current,
          },
        },
      );
      const newDetail = response.detail;
      const targetIndex = response.target_episode_index;
      resumeTimeRef.current = response.resume_time;

      // 更新URL参数（不刷新页面）
      const newUrl = new URL(window.location.href);
      newUrl.searchParams.set('source', newSource);
      newUrl.searchParams.set('id', newId);
      newUrl.searchParams.set('year', newDetail.year);
      window.history.replaceState({}, '', newUrl.toString());

      setVideoTitle(newDetail.title || newTitle);
      setVideoYear(newDetail.year);
      setVideoCover(newDetail.poster);
      setVideoDoubanId(newDetail.douban_id || 0);
      setCurrentSource(newSource);
      setCurrentId(newId);
      setDetail(newDetail);
      setCurrentEpisodeIndex(targetIndex);
    } catch (err) {
      // 隐藏换源加载状态
      setIsVideoLoading(false);
      setError(err instanceof Error ? err.message : '换源失败');
    }
  };

  useEffect(() => {
    document.addEventListener('keydown', handleKeyboardShortcuts);
    return () => {
      document.removeEventListener('keydown', handleKeyboardShortcuts);
    };
  }, []);

  // ---------------------------------------------------------------------------
  // 集数切换
  // ---------------------------------------------------------------------------
  // 处理集数切换
  const handleEpisodeChange = (episodeNumber: number) => {
    if (episodeNumber >= 0 && episodeNumber < totalEpisodes) {
      // 在更换集数前保存当前播放进度
      if (plyrRef.current && plyrRef.current.paused) {
        saveCurrentPlayProgress();
      }
      setCurrentEpisodeIndex(episodeNumber);
    }
  };

  const handlePreviousEpisode = () => {
    const d = detailRef.current;
    const idx = currentEpisodeIndexRef.current;
    if (d && d.episodes && idx > 0) {
      if (plyrRef.current && !plyrRef.current.paused) {
        saveCurrentPlayProgress();
      }
      setCurrentEpisodeIndex(idx - 1);
    }
  };

  const handleNextEpisode = () => {
    const d = detailRef.current;
    const idx = currentEpisodeIndexRef.current;
    if (d && d.episodes && idx < d.episodes.length - 1) {
      if (plyrRef.current && !plyrRef.current.paused) {
        saveCurrentPlayProgress();
      }
      setCurrentEpisodeIndex(idx + 1);
    }
  };

  const handleToggleBlockAd = async () => {
    const prevVal = blockAdEnabledRef.current;
    const newVal = !blockAdEnabledRef.current;
    // 乐观更新，保证 UI 立即反馈用户选择
    setBlockAdEnabled(newVal);
    blockAdEnabledRef.current = newVal;
    try {
      await invoke<void>('update_player_config', {
        config: { block_ad_enabled: newVal },
      });
      if (plyrRef.current) {
        resumeTimeRef.current = plyrRef.current.currentTime;
      }
      showToast(newVal ? '去广告已开启' : '去广告已关闭', 'success');
    } catch (err) {
      setBlockAdEnabled(prevVal);
      blockAdEnabledRef.current = prevVal;
      console.error('更新去广告配置失败', err);
      showToast('更新去广告配置失败', 'error');
    }
  };

  const handleToggleSkipEnable = () => {
    handleSkipConfigChange({
      ...skipConfigRef.current,
      enable: !skipConfigRef.current.enable,
    });
  };

  const handleSetIntroPoint = () => {
    const currentTime = plyrRef.current?.currentTime || 0;
    if (currentTime <= 0) return;
    handleSkipConfigChange({
      ...skipConfigRef.current,
      intro_time: currentTime,
    });
  };

  const handleSetOutroPoint = () => {
    const duration = plyrRef.current?.duration || 0;
    const currentTime = plyrRef.current?.currentTime || 0;
    const outroTime = -(duration - currentTime);
    if (outroTime >= 0) return;
    handleSkipConfigChange({
      ...skipConfigRef.current,
      outro_time: outroTime,
    });
  };

  const handleClearSkipConfig = () => {
    handleSkipConfigChange({
      enable: false,
      intro_time: 0,
      outro_time: 0,
    });
  };

  const enhancePlyrUi = () => {
    const container = playerContainerRef.current;
    if (!container) return;

    const controlsEl = container.querySelector<HTMLElement>('.plyr__controls');
    if (!controlsEl) return;

    let nextBtn = controlsEl.querySelector<HTMLButtonElement>(
      '.plyr__control--next-episode',
    );
    if (!nextBtn) {
      nextBtn = document.createElement('button');
      nextBtn.type = 'button';
      nextBtn.className = 'plyr__control plyr__control--next-episode';
      nextBtn.setAttribute('aria-label', '播放下一集');
      nextBtn.innerHTML =
        '<svg width="18" height="18" viewBox="0 0 22 22" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" fill="currentColor"/></svg>';

      const playBtn = controlsEl.querySelector<HTMLButtonElement>(
        '.plyr__control[data-plyr="play"]',
      );
      if (playBtn?.parentElement) {
        playBtn.parentElement.insertBefore(nextBtn, playBtn.nextSibling);
      } else {
        controlsEl.prepend(nextBtn);
      }
    }

    nextBtn.onclick = () => {
      handleNextEpisode();
    };
    const hasNext =
      !!detailRef.current?.episodes &&
      currentEpisodeIndexRef.current <
        (detailRef.current?.episodes?.length || 1) - 1;
    nextBtn.disabled = !hasNext;
    nextBtn.title = hasNext ? '播放下一集' : '已是最后一集';

    const settingsBtn = controlsEl.querySelector<HTMLButtonElement>(
      '.plyr__control[data-plyr="settings"]',
    );
    if (!settingsBtn) return;

    if (!settingsBtn.dataset.quantumHooked) {
      settingsBtn.dataset.quantumHooked = 'true';
      settingsBtn.addEventListener('click', () => {
        setTimeout(() => {
          enhancePlyrUi();
        }, 0);
      });
    }

    const menuId = settingsBtn.getAttribute('aria-controls');
    if (!menuId) return;

    const menuPanel = document.getElementById(menuId);
    const menuRoot =
      menuPanel?.querySelector<HTMLElement>('[id$="-home"] [role="menu"]') ||
      menuPanel?.querySelector<HTMLElement>('[role="menu"]');
    if (!menuRoot) return;

    let customGroup = menuRoot.querySelector<HTMLElement>(
      '[data-quantum-plyr-settings]',
    );
    if (!customGroup) {
      customGroup = document.createElement('div');
      customGroup.setAttribute('data-quantum-plyr-settings', 'true');
      customGroup.className = 'quantum-plyr-settings';
      menuRoot.appendChild(customGroup);
    }
    customGroup.innerHTML = '';

    const closeNativeSettings = () => {
      if (settingsBtn.getAttribute('aria-expanded') === 'true') {
        settingsBtn.click();
      }
    };

    const appendItem = (
      label: string,
      onClick: () => void,
      options?: { active?: boolean; danger?: boolean },
    ) => {
      const item = document.createElement('button');
      item.type = 'button';
      item.className = `plyr__control quantum-plyr-setting-item${
        options?.active ? ' is-active' : ''
      }${options?.danger ? ' is-danger' : ''}`;
      item.textContent = label;
      item.onclick = (event) => {
        event.preventDefault();
        event.stopPropagation();
        onClick();
        closeNativeSettings();
      };
      customGroup!.appendChild(item);
    };

    appendItem(
      `去广告${blockAdEnabledRef.current ? '(已开启)' : '(已关闭)'}`,
      () => {
        void handleToggleBlockAd();
      },
      { active: blockAdEnabledRef.current },
    );

    appendItem(
      `跳过片头片尾${skipConfigRef.current.enable ? '(已开启)' : '(已关闭)'}`,
      () => {
        handleToggleSkipEnable();
      },
      { active: skipConfigRef.current.enable },
    );

    appendItem(
      `设置片头 ${formatTime(skipConfigRef.current.intro_time)}`,
      () => {
        handleSetIntroPoint();
      },
      { active: skipConfigRef.current.intro_time > 0 },
    );

    appendItem(
      `设置片尾 ${
        skipConfigRef.current.outro_time < 0
          ? `-${formatTime(Math.abs(skipConfigRef.current.outro_time))}`
          : '--:--'
      }`,
      () => {
        handleSetOutroPoint();
      },
      { active: skipConfigRef.current.outro_time < 0 },
    );

    appendItem(
      '删除跳过配置',
      () => {
        handleClearSkipConfig();
      },
      { danger: true },
    );

    appendItem('打开跳过设置', () => {
      setIsSkipConfigPanelOpen(true);
    });
  };

  useEffect(() => {
    enhancePlyrUi();
  }, [
    blockAdEnabled,
    skipConfig.enable,
    skipConfig.intro_time,
    skipConfig.outro_time,
    currentEpisodeIndex,
    detail,
  ]);

  // ---------------------------------------------------------------------------
  // 键盘快捷键
  // ---------------------------------------------------------------------------
  // 处理全局快捷键
  const handleKeyboardShortcuts = (e: KeyboardEvent) => {
    // 忽略输入框中的按键事件
    if (
      (e.target as HTMLElement).tagName === 'INPUT' ||
      (e.target as HTMLElement).tagName === 'TEXTAREA'
    )
      return;

    // Alt + 左箭头 = 上一集
    if (e.altKey && e.key === 'ArrowLeft') {
      if (detailRef.current && currentEpisodeIndexRef.current > 0) {
        handlePreviousEpisode();
        e.preventDefault();
      }
    }

    // Alt + 右箭头 = 下一集
    if (e.altKey && e.key === 'ArrowRight') {
      const d = detailRef.current;
      const idx = currentEpisodeIndexRef.current;
      if (d && idx < d.episodes.length - 1) {
        handleNextEpisode();
        e.preventDefault();
      }
    }

    // 左箭头 = 快退
    if (!e.altKey && e.key === 'ArrowLeft') {
      if (plyrRef.current && plyrRef.current.currentTime > 5) {
        plyrRef.current.currentTime -= 10;
        e.preventDefault();
      }
    }

    // 右箭头 = 快进
    if (!e.altKey && e.key === 'ArrowRight') {
      if (
        plyrRef.current &&
        plyrRef.current.currentTime < plyrRef.current.duration - 5
      ) {
        plyrRef.current.currentTime += 10;
        e.preventDefault();
      }
    }

    // 上箭头 = 音量+
    if (e.key === 'ArrowUp') {
      if (plyrRef.current && plyrRef.current.volume < 1) {
        plyrRef.current.volume =
          Math.round((plyrRef.current.volume + 0.1) * 10) / 10;
        showToast(`音量: ${Math.round(plyrRef.current.volume * 100)}%`, 'info');
        e.preventDefault();
      }
    }

    // 下箭头 = 音量-
    if (e.key === 'ArrowDown') {
      if (plyrRef.current && plyrRef.current.volume > 0) {
        plyrRef.current.volume =
          Math.round((plyrRef.current.volume - 0.1) * 10) / 10;
        showToast(`音量: ${Math.round(plyrRef.current.volume * 100)}%`, 'info');
        e.preventDefault();
      }
    }

    // 空格 = 播放/暂停
    if (e.key === ' ') {
      if (plyrRef.current) {
        plyrRef.current.togglePlay();
        e.preventDefault();
      }
    }

    // f 键 = 切换全屏
    if (e.key === 'f' || e.key === 'F') {
      if (plyrRef.current) {
        plyrRef.current.fullscreen.toggle();
        e.preventDefault();
      }
    }
  };

  // ---------------------------------------------------------------------------
  // 播放记录相关
  // ---------------------------------------------------------------------------
  // 保存播放进度
  const saveCurrentPlayProgress = async () => {
    if (
      !plyrRef.current ||
      !currentSourceRef.current ||
      !currentIdRef.current
    ) {
      return;
    }

    const player = plyrRef.current;
    const currentTime = player.currentTime || 0;
    const duration = player.duration || 0;

    try {
      const saved = await invoke<boolean>('save_play_progress', {
        request: {
          source: currentSourceRef.current,
          id: currentIdRef.current,
          title: videoTitleRef.current,
          sourceName: detailRef.current?.source_name || '',
          year: detailRef.current?.year || '',
          cover: detailRef.current?.poster || '',
          episodeIndex: currentEpisodeIndexRef.current,
          totalEpisodes: detailRef.current?.episodes.length || 1,
          playTime: currentTime,
          totalTime: duration,
          searchTitle: searchTitle || '',
        },
      });

      if (!saved) {
        return;
      }

      // Notify other components

      lastSaveTimeRef.current = Date.now();
      console.log('Play progress saved:', {
        title: videoTitleRef.current,
        episode: currentEpisodeIndexRef.current + 1,
        year: detailRef.current?.year,
        progress: `${Math.floor(currentTime)}/${Math.floor(duration)}`,
      });
    } catch (err) {
      console.error('Failed to save play progress:', err);
    }
  };

  useEffect(() => {
    // 页面即将卸载时保存播放进度和清理资源
    const handleBeforeUnload = () => {
      saveCurrentPlayProgress();
      releaseWakeLock();
      cleanupPlayer();
    };

    // 页面可见性变化时保存播放进度和释放 Wake Lock
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'hidden') {
        saveCurrentPlayProgress();
        releaseWakeLock();
      } else if (document.visibilityState === 'visible') {
        // 页面可见时保存播放进度和请求 Wake Lock
        if (plyrRef.current && !plyrRef.current.paused) {
          requestWakeLock();
        }
      }
    };

    // 添加事件监听器
    window.addEventListener('beforeunload', handleBeforeUnload);
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      // 清理事件监听器
      window.removeEventListener('beforeunload', handleBeforeUnload);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [currentEpisodeIndex, detail, plyrRef.current]);

  // 清理定时器
  useEffect(() => {
    return () => {
      if (saveIntervalRef.current) {
        clearInterval(saveIntervalRef.current);
      }
    };
  }, []);

  // ---------------------------------------------------------------------------
  // 收藏相关
  // ---------------------------------------------------------------------------
  // 每当 source 或 id 变化时检查收藏状态
  useEffect(() => {
    if (!currentSource || !currentId) return;
    (async () => {
      try {
        const allFavorites = await invoke<RustFavorite[]>('get_play_favorites');
        const key = generateStorageKey(currentSource, currentId);
        const fav = allFavorites.some((f) => f.key === key);
        setFavorited(fav);
      } catch (err) {
        console.error('检查收藏状态失败:', err);
      }
    })();
  }, [currentSource, currentId]);

  // 监听收藏数据更新事件
  useEffect(() => {
    if (!currentSource || !currentId) return;

    const unsubscribe = subscribeToDataUpdates('favoritesUpdated', async () => {
      try {
        const allFavorites = await invoke<RustFavorite[]>('get_play_favorites');
        const key = generateStorageKey(currentSource, currentId);
        const isFav = allFavorites.some((f) => f.key === key);
        setFavorited(isFav);
      } catch (err) {
        console.error('检查收藏状态失败:', err);
      }
    });

    return unsubscribe;
  }, [currentSource, currentId]);

  // 切换收藏
  const handleToggleFavorite = async () => {
    if (
      !videoTitleRef.current ||
      !detailRef.current ||
      !currentSourceRef.current ||
      !currentIdRef.current
    )
      return;

    try {
      const key = generateStorageKey(
        currentSourceRef.current,
        currentIdRef.current,
      );
      const response = await invoke<{ favorited: boolean }>(
        'toggle_play_favorite',
        {
          record: {
            key,
            title: videoTitleRef.current,
            source_name: detailRef.current?.source_name || '',
            year: detailRef.current?.year || '',
            cover: detailRef.current?.poster || '',
            episode_index: currentEpisodeIndexRef.current + 1,
            total_episodes: detailRef.current?.episodes.length || 1,
            save_time: Math.floor(Date.now() / 1000),
            search_title: searchTitle || '',
          },
        },
      );
      setFavorited(response.favorited);
    } catch (err) {
      console.error('切换收藏失败:', err);
    }
  };

  useEffect(() => {
    if (
      !videoUrl ||
      loading ||
      currentEpisodeIndex === null ||
      !playerContainerRef.current
    ) {
      return;
    }

    if (
      !detail ||
      !detail.episodes ||
      currentEpisodeIndex >= detail.episodes.length ||
      currentEpisodeIndex < 0
    ) {
      setError(`选集索引无效，当前共 ${totalEpisodes} 集`);
      return;
    }

    const loadSource = (video: HTMLVideoElement, url: string) => {
      if (!url) return;

      if (hlsRef.current) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }
      if (video.hls) {
        video.hls.destroy();
        delete video.hls;
      }

      const isM3u8 = /\.m3u8($|\?)/i.test(url);
      if (isM3u8 && Hls.isSupported()) {
        const hls = new Hls({
          debug: false,
          enableWorker: true,
          lowLatencyMode: false,
          backBufferLength: 90,
          maxBufferLength: 120,
          maxMaxBufferLength: 240,
          maxBufferSize: 300 * 1000 * 1000,
          maxBufferHole: 0.8,
          maxFragLookUpTolerance: 0.5,
          nudgeOffset: 0.1,
          nudgeMaxRetry: 5,
          startLevel: -1,
          autoStartLoad: true,
          startPosition: -1,
          progressive: true,
          abrEwmaDefaultEstimate: 300000,
          abrBandWidthFactor: 0.85,
          abrBandWidthUpFactor: 0.6,
          abrEwmaFastLive: 2.0,
          abrEwmaSlowLive: 6.0,
          fragLoadingTimeOut: 25000,
          fragLoadingMaxRetry: 6,
          fragLoadingRetryDelay: 500,
          fragLoadingMaxRetryTimeout: 8000,
          manifestLoadingTimeOut: 15000,
          manifestLoadingMaxRetry: 4,
          manifestLoadingRetryDelay: 500,
          manifestLoadingMaxRetryTimeout: 8000,
          levelLoadingTimeOut: 15000,
          levelLoadingMaxRetry: 4,
          levelLoadingRetryDelay: 500,
          levelLoadingMaxRetryTimeout: 8000,
          loader: TauriHlsJsLoader,
          enableAdBlock: blockAdEnabledRef.current,
        } as any);

        hls.loadSource(url);
        hls.attachMedia(video);
        hlsRef.current = hls;
        video.hls = hls;

        hls.on(Hls.Events.ERROR, function (_event: any, data: any) {
          if (!data?.fatal) return;
          switch (data.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              hls.startLoad();
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              hls.recoverMediaError();
              break;
            default:
              hls.destroy();
              break;
          }
        });
      } else {
        video.src = url;
      }

      ensureVideoSource(video, url);
      video.load();
    };

    let cancelled = false;

    const initPlyr = async () => {
      try {
        const { default: PlyrConstructor } = await import('plyr');
        if (cancelled || !playerContainerRef.current) return;

        let video = videoElementRef.current;
        let player = plyrRef.current;

        if (!video) {
          if (typeof document === 'undefined') return;
          video = document.createElement('video');
          video.className = 'quantum-plyr-video';
          video.playsInline = true;
          video.controls = true;
          video.crossOrigin = 'anonymous';
          video.disableRemotePlayback = false;
          playerContainerRef.current.innerHTML = '';
          playerContainerRef.current.appendChild(video);
          videoElementRef.current = video;
        }

        if (!player) {
          player = new PlyrConstructor(video, {
            autoplay: true,
            muted: false,
            volume: lastVolumeRef.current,
            seekTime: 10,
            clickToPlay: true,
            resetOnEnd: false,
            fullscreen: {
              enabled: true,
              fallback: true,
              iosNative: false,
            },
            keyboard: {
              focused: false,
              global: false,
            },
            speed: {
              selected: lastPlaybackRateRef.current,
              options: [0.5, 0.75, 1, 1.25, 1.5, 2, 3],
            },
            controls: [
              'play-large',
              'play',
              'progress',
              'current-time',
              'duration',
              'mute',
              'volume',
              'settings',
              'pip',
              'airplay',
              'fullscreen',
            ],
            settings: ['speed', 'loop'],
            i18n: {
              speed: '速度',
              normal: '正常',
              settings: '设置',
              disabled: '关闭',
              enabled: '开启',
            },
          });

          plyrRef.current = player;

          player.on('ready', () => {
            setError(null);
            enhancePlyrUi();
            if (!player!.paused) {
              requestWakeLock();
            }
          });

          player.on('play', () => {
            requestWakeLock();
          });

          player.on('pause', () => {
            releaseWakeLock();
            saveCurrentPlayProgress();
          });

          player.on('ended', () => {
            releaseWakeLock();
            const d = detailRef.current;
            const idx = currentEpisodeIndexRef.current;
            if (d && d.episodes && idx < d.episodes.length - 1) {
              setTimeout(() => {
                setCurrentEpisodeIndex(idx + 1);
              }, 1000);
            }
          });

          player.on('volumechange', () => {
            lastVolumeRef.current = player!.volume;
          });

          player.on('ratechange', () => {
            lastPlaybackRateRef.current = player!.speed;
          });

          player.on('canplay', () => {
            if (resumeTimeRef.current && resumeTimeRef.current > 0) {
              try {
                const duration = player!.duration || 0;
                let target = resumeTimeRef.current;
                if (duration && target >= duration - 2) {
                  target = Math.max(0, duration - 5);
                }
                player!.currentTime = target;
              } catch (err) {
                console.warn('设置播放位置失败:', err);
              }
            }

            resumeTimeRef.current = null;
            setTimeout(() => {
              if (Math.abs(player!.volume - lastVolumeRef.current) > 0.01) {
                player!.volume = lastVolumeRef.current;
              }
              if (
                Math.abs(player!.speed - lastPlaybackRateRef.current) > 0.01
              ) {
                player!.speed = lastPlaybackRateRef.current;
              }
            }, 0);

            setIsVideoLoading(false);
          });

          player.on('timeupdate', async () => {
            const currentTime = player!.currentTime || 0;
            const duration = player!.duration || 0;
            const now = Date.now();

            let interval = 5000;
            if (process.env.NEXT_PUBLIC_STORAGE_TYPE === 'upstash') {
              interval = 20000;
            }
            if (now - lastSaveTimeRef.current > interval) {
              saveCurrentPlayProgress();
              lastSaveTimeRef.current = now;
            }

            if (now - lastSkipCheckRef.current < 1500) return;
            lastSkipCheckRef.current = now;

            if (skipConfigRef.current.enable && duration > 0) {
              try {
                const skipAction = await invoke<SkipAction>(
                  'check_skip_action',
                  {
                    introTime: skipConfigRef.current.intro_time,
                    outroTime: Math.abs(skipConfigRef.current.outro_time),
                    currentTime,
                    totalDuration: duration,
                  },
                );

                if (
                  typeof skipAction === 'object' &&
                  'SkipIntro' in skipAction &&
                  currentTime > 0.5
                ) {
                  const targetTime = skipAction.SkipIntro;
                  player!.currentTime = targetTime;
                  showToast(
                    `跳过片头，跳转到 ${formatTime(targetTime)}`,
                    'success',
                  );
                } else if (
                  skipAction === 'SkipOutro' &&
                  currentTime < duration - 1
                ) {
                  if (
                    currentEpisodeIndexRef.current <
                    (detailRef.current?.episodes?.length || 1) - 1
                  ) {
                    showToast('跳过片尾，跳转到下一集', 'info');
                    setTimeout(() => {
                      handleNextEpisode();
                    }, 500);
                  } else {
                    showToast('跳过片尾，但当前已是最后一集', 'info');
                    player!.pause();
                  }
                }
              } catch (err) {
                console.error('跳过检测失败', err);
              }
            }

            const detail = detailRef.current;
            const currentIdx = currentEpisodeIndexRef.current;
            if (detail && detail.episodes) {
              try {
                const decision = await invoke<{ did_preload: boolean }>(
                  'preload_next_episode_if_needed',
                  {
                    source: detail.source,
                    id: detail.id,
                    currentEpisode: currentIdx,
                    totalEpisodes: detail.episodes.length,
                    currentTime,
                    totalDuration: duration,
                  },
                );

                if (decision.did_preload) {
                  const stats =
                    await invoke<
                      Record<
                        string,
                        { entry_count: number; weighted_size: number }
                      >
                    >('get_cache_stats');
                  console.log(
                    '📊 预载后缓存统计 | 视频缓存:',
                    stats.video.entry_count,
                    '条 | 搜索缓存:',
                    stats.search.entry_count,
                    '条',
                  );
                }
              } catch (err) {
                console.error('预载失败:', err);
              }
            }
          });

          player.on('error', (err: any) => {
            console.error('播放器错误', err);
            if ((player?.currentTime || 0) <= 0) {
              setError('无法播放');
            }
          });

          if (!player.paused) {
            requestWakeLock();
          }
        }

        if (cancelled) return;
        player.poster = videoCover;
        setIsVideoLoading(true);
        loadSource(video, videoUrl);
        setTimeout(() => {
          enhancePlyrUi();
        }, 0);
      } catch (err) {
        console.error('创建播放器失败:', err);
        setError('播放器初始化失败');
      }
    };

    void initPlyr();
    return () => {
      cancelled = true;
    };
  }, [
    videoUrl,
    loading,
    currentEpisodeIndex,
    detail,
    totalEpisodes,
    videoCover,
    blockAdEnabled,
  ]);

  // 当组件卸载时清理定时器、Wake Lock 和播放器资源
  useEffect(() => {
    return () => {
      // 清理定时器
      if (saveIntervalRef.current) {
        clearInterval(saveIntervalRef.current);
      }

      // 释放 Wake Lock
      releaseWakeLock();

      // 销毁播放器实例
      cleanupPlayer();
    };
  }, []);

  if (loading) {
    return (
      <PageLayout activePath='/play'>
        <div className='flex items-center justify-center min-h-screen bg-transparent'>
          <div className='text-center max-w-md mx-auto px-6'>
            {/* 动画影院图标 */}
            <div className='relative mb-8'>
              <div className='relative mx-auto w-24 h-24 bg-linear-to-r from-green-500 to-emerald-600 rounded-2xl shadow-2xl flex items-center justify-center transform hover:scale-105 transition-transform duration-300'>
                <div className='text-white text-4xl'>
                  {loadingStage === 'searching' && '🔍'}
                  {loadingStage === 'preferring' && '⚡'}
                  {loadingStage === 'fetching' && '🎬'}
                  {loadingStage === 'ready' && '✨'}
                </div>
                {/* 旋转光环 */}
                <div className='absolute -inset-2 bg-linear-to-r from-green-500 to-emerald-600 rounded-2xl opacity-20 animate-spin'></div>
              </div>

              {/* 浮动粒子效果 */}
              <div className='absolute top-0 left-0 w-full h-full pointer-events-none'>
                <div className='absolute top-2 left-2 w-2 h-2 bg-green-400 rounded-full animate-bounce'></div>
                <div
                  className='absolute top-4 right-4 w-1.5 h-1.5 bg-emerald-400 rounded-full animate-bounce'
                  style={{ animationDelay: '0.5s' }}
                ></div>
                <div
                  className='absolute bottom-3 left-6 w-1 h-1 bg-lime-400 rounded-full animate-bounce'
                  style={{ animationDelay: '1s' }}
                ></div>
              </div>
            </div>

            {/* 进度指示器 */}
            <div className='mb-6 w-80 mx-auto'>
              <div className='flex justify-center space-x-2 mb-4'>
                <div
                  className={`w-3 h-3 rounded-full transition-all duration-500 ${
                    loadingStage === 'searching' || loadingStage === 'fetching'
                      ? 'bg-green-500 scale-125'
                      : loadingStage === 'preferring' ||
                          loadingStage === 'ready'
                        ? 'bg-green-500'
                        : 'bg-gray-300'
                  }`}
                ></div>
                <div
                  className={`w-3 h-3 rounded-full transition-all duration-500 ${
                    loadingStage === 'preferring'
                      ? 'bg-green-500 scale-125'
                      : loadingStage === 'ready'
                        ? 'bg-green-500'
                        : 'bg-gray-300'
                  }`}
                ></div>
                <div
                  className={`w-3 h-3 rounded-full transition-all duration-500 ${
                    loadingStage === 'ready'
                      ? 'bg-green-500 scale-125'
                      : 'bg-gray-300'
                  }`}
                ></div>
              </div>

              {/* 进度条 */}
              <div className='w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden'>
                <div
                  className='h-full bg-linear-to-r from-green-500 to-emerald-600 rounded-full transition-all duration-1000 ease-out'
                  style={{
                    width:
                      loadingStage === 'searching' ||
                      loadingStage === 'fetching'
                        ? '33%'
                        : loadingStage === 'preferring'
                          ? '66%'
                          : '100%',
                  }}
                ></div>
              </div>
            </div>

            {/* 加载消息 */}
            <div className='space-y-2'>
              <p className='text-xl font-semibold text-gray-800 dark:text-gray-200 animate-pulse'>
                {loadingMessage}
              </p>
            </div>
          </div>
        </div>
      </PageLayout>
    );
  }

  if (error) {
    return (
      <PageLayout activePath='/play'>
        <div className='flex items-center justify-center min-h-screen bg-transparent'>
          <div className='text-center max-w-md mx-auto px-6'>
            {/* 错误图标 */}
            <div className='relative mb-8'>
              <div className='relative mx-auto w-24 h-24 bg-linear-to-r from-red-500 to-orange-500 rounded-2xl shadow-2xl flex items-center justify-center transform hover:scale-105 transition-transform duration-300'>
                <div className='text-white text-4xl'>😵</div>
                {/* 脉冲效果 */}
                <div className='absolute -inset-2 bg-linear-to-r from-red-500 to-orange-500 rounded-2xl opacity-20 animate-pulse'></div>
              </div>

              {/* 浮动错误粒子 */}
              <div className='absolute top-0 left-0 w-full h-full pointer-events-none'>
                <div className='absolute top-2 left-2 w-2 h-2 bg-red-400 rounded-full animate-bounce'></div>
                <div
                  className='absolute top-4 right-4 w-1.5 h-1.5 bg-orange-400 rounded-full animate-bounce'
                  style={{ animationDelay: '0.5s' }}
                ></div>
                <div
                  className='absolute bottom-3 left-6 w-1 h-1 bg-yellow-400 rounded-full animate-bounce'
                  style={{ animationDelay: '1s' }}
                ></div>
              </div>
            </div>

            {/* 错误信息 */}
            <div className='space-y-4 mb-8'>
              <h2 className='text-2xl font-bold text-gray-800 dark:text-gray-200'>
                哎呀，出现了一些问题
              </h2>
              <div className='bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4'>
                <p className='text-red-600 dark:text-red-400 font-medium'>
                  {error}
                </p>
              </div>
              <p className='text-sm text-gray-500 dark:text-gray-400'>
                请检查网络连接或尝试刷新页面
              </p>
            </div>

            {/* 操作按钮 */}
            <div className='space-y-3'>
              <button
                onClick={() =>
                  videoTitle
                    ? router.push(`/search?q=${encodeURIComponent(videoTitle)}`)
                    : router.back()
                }
                className='w-full px-6 py-3 bg-linear-to-r from-green-500 to-emerald-600 text-white rounded-xl font-medium hover:from-green-600 hover:to-emerald-700 transform hover:scale-105 transition-all duration-200 shadow-lg hover:shadow-xl'
              >
                {videoTitle ? '🔍 返回搜索' : '← 返回上页'}
              </button>

              <button
                onClick={() => window.location.reload()}
                className='w-full px-6 py-3 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-xl font-medium hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors duration-200'
              >
                🔄 重新尝试
              </button>
            </div>
          </div>
        </div>
      </PageLayout>
    );
  }

  return (
    <PageLayout activePath='/play'>
      <div
        className={`${appLayoutClasses.pageShell} flex flex-col gap-4 py-4 max-[375px]:py-3.5 min-[834px]:py-6 min-[1440px]:py-8`}
      >
        {/* 第一行：影片标题 */}
        <div className='py-1 flex justify-between items-center gap-2'>
          <h1 className='truncate text-lg font-semibold text-gray-900 max-[375px]:text-base min-[834px]:text-2xl min-[1440px]:text-[1.75rem] dark:text-gray-100'>
            {videoTitle || '影片标题'}
            {totalEpisodes > 1 && (
              <span className='text-gray-500 dark:text-gray-400 ml-2 text-base font-normal'>
                {`> ${
                  detail?.episodes_titles?.[currentEpisodeIndex] ||
                  `第${currentEpisodeIndex + 1} 集`
                }`}
              </span>
            )}
          </h1>

          {/* 移动端跳过设置按钮 */}
          <button
            onClick={() => setIsSkipConfigPanelOpen(true)}
            className={`tap-target lg:hidden shrink-0 flex items-center gap-1.5 px-3 py-1.5 max-[375px]:px-2.5 min-[834px]:px-4 rounded-full text-xs min-[834px]:text-sm font-medium transition-all duration-200 ${
              skipConfig.enable
                ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300 ring-1 ring-purple-500/20'
                : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400 ring-1 ring-gray-500/10'
            }`}
          >
            <svg
              className='w-3.5 h-3.5'
              fill='none'
              stroke='currentColor'
              viewBox='0 0 24 24'
            >
              <path
                strokeLinecap='round'
                strokeLinejoin='round'
                strokeWidth={2}
                d='M13 5l7 7-7 7M5 5l7 7-7 7'
              />
            </svg>
            <span>{skipConfig.enable ? '已跳过' : '跳过'}</span>
          </button>
        </div>
        {/* 第二行：播放器和选集 */}
        <div className='space-y-2'>
          {/* 折叠控制和跳过设置 - 仅在 lg 及以上屏幕显示 */}
          <div className='hidden lg:flex items-center justify-between'>
            {/* 跳过片头片尾设置按钮 */}
            <button
              onClick={() => setIsSkipConfigPanelOpen(true)}
              className={`tap-target group relative flex items-center space-x-2 px-4 py-2 rounded-xl bg-linear-to-r transition-all duration-200 shadow-md hover:shadow-lg transform hover:scale-105 ${
                skipConfig.enable
                  ? 'from-purple-600 via-pink-500 to-indigo-600 text-white'
                  : 'from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-800 text-gray-700 dark:text-gray-300'
              }`}
              title='设置跳过片头片尾'
            >
              <svg
                className='w-5 h-5'
                fill='none'
                stroke='currentColor'
                viewBox='0 0 24 24'
              >
                <path
                  strokeLinecap='round'
                  strokeLinejoin='round'
                  strokeWidth={2}
                  d='M13 5l7 7-7 7M5 5l7 7-7 7'
                />
              </svg>
              <span className='text-sm font-medium'>
                {skipConfig.enable ? '✨ 跳过已启用' : '⚙️ 跳过设置'}
              </span>
              {skipConfig.enable && (
                <div className='absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full animate-pulse'></div>
              )}
            </button>

            <button
              onClick={() =>
                setIsEpisodeSelectorCollapsed(!isEpisodeSelectorCollapsed)
              }
              className='tap-target group relative flex items-center space-x-1.5 px-3 py-1.5 rounded-full bg-white/80 hover:bg-white dark:bg-gray-800/80 dark:hover:bg-gray-800 backdrop-blur-sm border border-gray-200/50 dark:border-gray-700/50 shadow-sm hover:shadow-md transition-all duration-200'
              title={
                isEpisodeSelectorCollapsed ? '显示选集面板' : '隐藏选集面板'
              }
            >
              <svg
                className={`w-3.5 h-3.5 text-gray-500 dark:text-gray-400 transition-transform duration-200 ${
                  isEpisodeSelectorCollapsed ? 'rotate-180' : 'rotate-0'
                }`}
                fill='none'
                stroke='currentColor'
                viewBox='0 0 24 24'
              >
                <path
                  strokeLinecap='round'
                  strokeLinejoin='round'
                  strokeWidth='2'
                  d='M9 5l7 7-7 7'
                />
              </svg>
              <span className='text-xs font-medium text-gray-600 dark:text-gray-300'>
                {isEpisodeSelectorCollapsed ? '显示' : '隐藏'}
              </span>

              {/* 精致的状态指示点 */}
              <div
                className={`absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full transition-all duration-200 ${
                  isEpisodeSelectorCollapsed
                    ? 'bg-orange-400 animate-pulse'
                    : 'bg-green-400'
                }`}
              ></div>
            </button>
          </div>

          <div
            className={`grid gap-4 transition-all duration-300 ease-in-out lg:h-[70vh] min-[834px]:h-[72vh] min-[1440px]:h-[78vh] 2xl:h-[80vh] ${
              isEpisodeSelectorCollapsed
                ? 'grid-cols-1'
                : 'grid-cols-1 md:grid-cols-4'
            }`}
          >
            {/* 播放器 */}
            <div
              className={`h-full transition-all duration-300 ease-in-out rounded-xl border border-white/0 dark:border-white/30 ${
                isEpisodeSelectorCollapsed ? 'col-span-1' : 'md:col-span-3'
              }`}
            >
              <div className='group/player relative h-[18rem] w-full max-[375px]:h-[16rem] sm:h-[20rem] md:h-[24rem] min-[834px]:h-[26rem] lg:h-full'>
                <div
                  ref={playerContainerRef}
                  className='quantum-plyr-shell bg-black w-full h-full rounded-xl overflow-hidden shadow-lg'
                ></div>

                {/* 加载中的提示 */}
                {isVideoLoading && (
                  <div className='absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm rounded-xl'>
                    <div className='flex flex-col items-center gap-3'>
                      {/* <div className='w-10 h-10 border-4 border-green-500 border-t-transparent rounded-full animate-spin' /> */}
                      <span className='text-white/80 text-sm'>
                        {videoLoadingStage === 'sourceChanging'
                          ? '切换播放源...'
                          : '正在加载视频...'}
                      </span>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* 选集和换源 - 在移动端始终显示，在 lg 及以上可折叠 */}
            <div
              className={`h-[18rem] max-[375px]:h-[16rem] sm:h-[20rem] md:h-[24rem] min-[834px]:h-[26rem] lg:h-full md:overflow-hidden transition-all duration-300 ease-in-out ${
                isEpisodeSelectorCollapsed
                  ? 'md:col-span-1 lg:hidden lg:opacity-0 lg:scale-95'
                  : 'md:col-span-1 lg:opacity-100 lg:scale-100'
              }`}
            >
              <EpisodeSelector
                totalEpisodes={totalEpisodes}
                episodes_titles={detail?.episodes_titles || []}
                value={currentEpisodeIndex + 1}
                onChange={handleEpisodeChange}
                onSourceChange={handleSourceChange}
                currentSource={currentSource}
                currentId={currentId}
                videoTitle={searchTitle || videoTitle}
                availableSources={availableSources}
                sourceSearchLoading={sourceSearchLoading}
                sourceSearchError={sourceSearchError}
                precomputedVideoInfo={precomputedVideoInfo}
                optimizationEnabled={optimizationEnabled}
              />
            </div>
          </div>
        </div>

        {/* 详情展示 */}
        <div className='grid grid-cols-1 gap-4 lg:grid-cols-5 lg:gap-6'>
          {/* 文字区 */}
          <div className='lg:col-span-3'>
            <div className='flex min-h-0 flex-col p-5 max-[375px]:p-4 min-[834px]:p-6 min-[1440px]:p-7'>
              {/* 标题 */}
              <h1 className='mb-2 flex w-full shrink-0 items-center text-center text-xl font-bold tracking-wide text-slate-900 dark:text-gray-100 sm:text-2xl lg:text-left lg:text-3xl'>
                {videoTitle || '影片标题'}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleToggleFavorite();
                  }}
                  className='ml-3 shrink-0 hover:opacity-80 transition-opacity'
                >
                  <FavoriteIcon filled={favorited} />
                </button>
              </h1>

              {/* 关键信息行 */}
              <div className='mb-4 flex shrink-0 flex-wrap items-center gap-3 text-sm min-[834px]:text-base min-[1440px]:text-[1.05rem] text-slate-700 dark:text-gray-300'>
                {detail?.class && (
                  <span className='text-green-600 dark:text-green-400 font-semibold'>
                    {detail.class}
                  </span>
                )}
                {(detail?.year || videoYear) && (
                  <span className='text-gray-600 dark:text-gray-400'>
                    {detail?.year || videoYear}
                  </span>
                )}
                {detail?.source_name && (
                  <span className='border border-gray-400 dark:border-gray-500 px-2 py-px rounded text-gray-700 dark:text-gray-300'>
                    {detail.source_name}
                  </span>
                )}
                {detail?.type_name && (
                  <span className='text-gray-600 dark:text-gray-400'>
                    {detail.type_name}
                  </span>
                )}
              </div>
              {/* 剧情简介 */}
              {detail?.desc && (
                <div
                  className='mt-0 text-base leading-relaxed text-slate-700 dark:text-gray-300 overflow-y-auto pr-2 flex-1 min-h-0 scrollbar-hide'
                  style={{ whiteSpace: 'pre-line' }}
                >
                  {detail.desc}
                </div>
              )}
            </div>
          </div>

          {/* 封面展示 */}
          <div className='hidden lg:order-first lg:col-span-2 lg:block'>
            <div className='px-0 py-4 lg:pr-6'>
              <div className='relative bg-gray-300 dark:bg-gray-700 aspect-2/3 flex items-center justify-center rounded-xl overflow-hidden'>
                {videoCover ? (
                  <>
                    <img
                      src={proxiedCoverUrl}
                      alt={videoTitle}
                      className='w-full h-full object-cover'
                    />
                  </>
                ) : (
                  <span className='text-gray-600 dark:text-gray-400'>
                    封面图片
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 跳过片头片尾设置面板 */}
      <SkipConfigPanel
        isOpen={isSkipConfigPanelOpen}
        onClose={() => setIsSkipConfigPanelOpen(false)}
        config={skipConfig}
        onChange={handleSkipConfigChange}
        videoDuration={plyrRef.current?.duration || 0}
        currentTime={plyrRef.current?.currentTime || 0}
      />

      {/* Toast 通知 */}
      {toast.show && (
        <Toast
          message={toast.message}
          type={toast.type}
          duration={3000}
          onClose={() => setToast({ show: false, message: '', type: 'info' })}
        />
      )}
    </PageLayout>
  );
}

// FavoriteIcon 组件
const FavoriteIcon = ({ filled }: { filled: boolean }) => {
  if (filled) {
    return (
      <svg
        className='h-7 w-7'
        viewBox='0 0 24 24'
        xmlns='http://www.w3.org/2000/svg'
      >
        <path
          d='M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z'
          fill='#ef4444' /* Tailwind red-500 */
          stroke='#ef4444'
          strokeWidth='2'
          strokeLinecap='round'
          strokeLinejoin='round'
        />
      </svg>
    );
  }
  return (
    <Heart className='h-7 w-7 stroke-1 text-gray-600 dark:text-gray-300' />
  );
};

export default function PlayPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <PlayPageClient />
    </Suspense>
  );
}
