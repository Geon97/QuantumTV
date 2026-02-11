/* eslint-disable no-console,@typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

'use client';

import { invoke } from '@tauri-apps/api/core';
import {
  Check,
  ChevronDown,
  Database,
  ExternalLink,
  Settings,
  Shield,
  Trash2,
  User,
  X,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

import { UpdateStatus } from '@/lib/types';
import { usePageCache, CacheStats } from '@/hooks/usePageCache';

import { VersionPanel } from './VersionPanel';

// 本地定义 VersionCheckResult 类型
interface VersionCheckResult {
  status: UpdateStatus;
  local_timestamp?: string;
  remote_timestamp?: string;
  formatted_local_time?: string;
  formatted_remote_time?: string;
  error?: string;
}

// 获取当前版本
async function getCurrentVersion(): Promise<string> {
  // 调用 Rust 后端的 get_current_version 函数
  try {
    return await invoke('get_current_version');
  } catch (error) {
    console.warn('获取当前版本失败:', error);
    return '0.0.0'; // 默认版本
  }
}

// 检查更新
async function checkForUpdates(): Promise<VersionCheckResult> {
  try {
    // 调用 Rust 后端的 check_for_updates 函数
    return await invoke('check_for_updates');
  } catch (error) {
    console.warn('版本检查失败:', error);
    return {
      status: UpdateStatus.FETCH_FAILED,
      error: error instanceof Error ? error.message : '未知错误',
    };
  }
}

export const UserMenu: React.FC = () => {
  const router = useRouter();
  const [isOpen, setIsOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isVersionPanelOpen, setIsVersionPanelOpen] = useState(false);
  const [mounted, setMounted] = useState(false);

  // 当前版本状态
  const [currentVersion, setCurrentVersion] = useState<string>('0.0.0');

  // Body 滚动锁定 - 使用 overflow 方式避免布局问题
  useEffect(() => {
    if (isSettingsOpen) {
      const body = document.body;
      const html = document.documentElement;

      // 保存原始样式
      const originalBodyOverflow = body.style.overflow;
      const originalHtmlOverflow = html.style.overflow;

      // 只设置 overflow 来阻止滚动
      body.style.overflow = 'hidden';
      html.style.overflow = 'hidden';

      return () => {
        // 恢复所有原始样式
        body.style.overflow = originalBodyOverflow;
        html.style.overflow = originalHtmlOverflow;
      };
    }
  }, [isSettingsOpen]);

  // 设置相关状态
  const [defaultAggregateSearch, setDefaultAggregateSearch] = useState(true);
  const [doubanProxyUrl, setDoubanProxyUrl] = useState('');
  const [enableOptimization, setEnableOptimization] = useState(true);
  const [fluidSearch, setFluidSearch] = useState(true);
  const [liveDirectConnect, setLiveDirectConnect] = useState(false);
  const [playerBufferMode, setPlayerBufferMode] = useState<
    'standard' | 'enhanced' | 'max'
  >('standard');
  const [doubanDataSource, setDoubanDataSource] = useState(
    'cmliussss-cdn-tencent',
  );
  const [doubanImageProxyType, setDoubanImageProxyType] = useState(
    'cmliussss-cdn-tencent',
  );
  const [doubanImageProxyUrl, setDoubanImageProxyUrl] = useState('');
  const [isDoubanDropdownOpen, setIsDoubanDropdownOpen] = useState(false);
  const [isDoubanImageProxyDropdownOpen, setIsDoubanImageProxyDropdownOpen] =
    useState(false);

  // 豆瓣数据源选项
  const doubanDataSourceOptions = [
    { value: 'direct', label: '直连（服务器直接请求豆瓣）' },
    { value: 'cors-proxy-zwei', label: 'Cors Proxy By Zwei' },
    {
      value: 'cmliussss-cdn-tencent',
      label: '豆瓣 CDN By CMLiussss（腾讯云）',
    },
    { value: 'cmliussss-cdn-ali', label: '豆瓣 CDN By CMLiussss（阿里云）' },
    { value: 'custom', label: '自定义代理' },
  ];

  // 豆瓣图片代理选项
  const doubanImageProxyTypeOptions = [
    { value: 'direct', label: '直连（浏览器直接请求豆瓣）' },
    {
      value: 'cmliussss-cdn-tencent',
      label: '豆瓣 CDN By CMLiussss（腾讯云）',
    },
    { value: 'cmliussss-cdn-ali', label: '豆瓣 CDN By CMLiussss（阿里云）' },
    { value: 'custom', label: '自定义代理' },
  ];

  // 版本检查相关状态
  const [updateStatus, setUpdateStatus] = useState<{
    status: UpdateStatus;
    localTimestamp?: string;
    remoteTimestamp?: string;
  } | null>(null);
  const [isChecking, setIsChecking] = useState(true);

  // 缓存管理相关状态
  const [cacheStats, setCacheStats] = useState<CacheStats | null>(null);
  const [isClearingCache, setIsClearingCache] = useState(false);
  const { getStats, clearAll, cleanupExpired } = usePageCache();

  // 确保组件已挂载
  useEffect(() => {
    setMounted(true);
  }, []);

  // 从 localStorage 读取设置
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const savedAggregateSearch = localStorage.getItem(
        'defaultAggregateSearch',
      );
      if (savedAggregateSearch !== null) {
        setDefaultAggregateSearch(JSON.parse(savedAggregateSearch));
      }

      const savedDoubanDataSource = localStorage.getItem('doubanDataSource');
      const defaultDoubanProxyType =
        (window as any).RUNTIME_CONFIG?.DOUBAN_PROXY_TYPE ||
        'cmliussss-cdn-tencent';
      if (savedDoubanDataSource !== null) {
        setDoubanDataSource(savedDoubanDataSource);
      } else if (defaultDoubanProxyType) {
        setDoubanDataSource(defaultDoubanProxyType);
      }

      const savedDoubanProxyUrl = localStorage.getItem('doubanProxyUrl');
      const defaultDoubanProxy =
        (window as any).RUNTIME_CONFIG?.DOUBAN_PROXY || '';
      if (savedDoubanProxyUrl !== null) {
        setDoubanProxyUrl(savedDoubanProxyUrl);
      } else if (defaultDoubanProxy) {
        setDoubanProxyUrl(defaultDoubanProxy);
      }

      const savedDoubanImageProxyType = localStorage.getItem(
        'doubanImageProxyType',
      );
      const defaultDoubanImageProxyType =
        (window as any).RUNTIME_CONFIG?.DOUBAN_IMAGE_PROXY_TYPE ||
        'cmliussss-cdn-tencent';
      if (savedDoubanImageProxyType !== null) {
        setDoubanImageProxyType(savedDoubanImageProxyType);
      } else if (defaultDoubanImageProxyType) {
        setDoubanImageProxyType(defaultDoubanImageProxyType);
      }

      const savedDoubanImageProxyUrl = localStorage.getItem(
        'doubanImageProxyUrl',
      );
      const defaultDoubanImageProxyUrl =
        (window as any).RUNTIME_CONFIG?.DOUBAN_IMAGE_PROXY || '';
      if (savedDoubanImageProxyUrl !== null) {
        setDoubanImageProxyUrl(savedDoubanImageProxyUrl);
      } else if (defaultDoubanImageProxyUrl) {
        setDoubanImageProxyUrl(defaultDoubanImageProxyUrl);
      }

      const savedEnableOptimization =
        localStorage.getItem('enableOptimization');
      if (savedEnableOptimization !== null) {
        setEnableOptimization(JSON.parse(savedEnableOptimization));
      }

      // 读取流式搜索设置 - 从 Tauri 后端读取
      const loadFluidSearchSetting = async () => {
        try {
          const enabled = await invoke<boolean>('get_fluid_search');
          setFluidSearch(enabled);
        } catch (error) {
          console.error('读取流式搜索设置失败:', error);
          // 降级到 localStorage
          const savedFluidSearch = localStorage.getItem('fluidSearch');
          const defaultFluidSearch =
            (window as any).RUNTIME_CONFIG?.FLUID_SEARCH !== false;
          if (savedFluidSearch !== null) {
            setFluidSearch(JSON.parse(savedFluidSearch));
          } else if (defaultFluidSearch !== undefined) {
            setFluidSearch(defaultFluidSearch);
          }
        }
      };

      loadFluidSearchSetting();

      const savedLiveDirectConnect = localStorage.getItem('liveDirectConnect');
      if (savedLiveDirectConnect !== null) {
        setLiveDirectConnect(JSON.parse(savedLiveDirectConnect));
      }
    }
  }, []);

  // 获取当前版本
  useEffect(() => {
    const getVersion = async () => {
      try {
        const version = await getCurrentVersion();
        setCurrentVersion(version);
      } catch (error) {
        console.warn('获取当前版本失败:', error);
      }
    };

    getVersion();
  }, []);

  // 版本检查
  useEffect(() => {
    const checkUpdate = async () => {
      try {
        const result = await checkForUpdates();
        setUpdateStatus({
          status: result.status,
          localTimestamp: result.local_timestamp,
          remoteTimestamp: result.remote_timestamp,
        });
      } catch (error) {
        console.warn('版本检查失败:', error);
      } finally {
        setIsChecking(false);
      }
    };

    checkUpdate();
  }, []);

  // 点击外部区域关闭下拉框
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (isDoubanDropdownOpen) {
        const target = event.target as Element;
        if (!target.closest('[data-dropdown="douban-datasource"]')) {
          setIsDoubanDropdownOpen(false);
        }
      }
    };

    if (isDoubanDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () =>
        document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isDoubanDropdownOpen]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (isDoubanImageProxyDropdownOpen) {
        const target = event.target as Element;
        if (!target.closest('[data-dropdown="douban-image-proxy"]')) {
          setIsDoubanImageProxyDropdownOpen(false);
        }
      }
    };

    if (isDoubanImageProxyDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () =>
        document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isDoubanImageProxyDropdownOpen]);

  const handleMenuClick = () => {
    setIsOpen(!isOpen);
  };

  const handleCloseMenu = () => {
    setIsOpen(false);
  };

  const handleAdminPanel = () => {
    router.push('/admin');
  };

  const handleSettings = () => {
    setIsOpen(false);
    setIsSettingsOpen(true);
    // 加载缓存统计
    loadCacheStats();
  };

  const handleCloseSettings = () => {
    setIsSettingsOpen(false);
  };

  // 加载缓存统计
  const loadCacheStats = async () => {
    try {
      const stats = await getStats();
      setCacheStats(stats);
    } catch (error) {
      console.error('加载缓存统计失败:', error);
    }
  };

  // 清空所有缓存
  const handleClearAllCache = async () => {
    if (!confirm('确定要清空所有页面缓存吗？这将删除首页、电影、剧集等所有缓存数据。')) {
      return;
    }

    setIsClearingCache(true);
    try {
      await clearAll();
      await loadCacheStats();
      alert('缓存已清空');
    } catch (error) {
      console.error('清空缓存失败:', error);
      alert('清空缓存失败');
    } finally {
      setIsClearingCache(false);
    }
  };

  // 清理过期缓存
  const handleCleanupExpiredCache = async () => {
    setIsClearingCache(true);
    try {
      const count = await cleanupExpired();
      await loadCacheStats();
      alert(`已清理 ${count} 个过期缓存`);
    } catch (error) {
      console.error('清理过期缓存失败:', error);
      alert('清理过期缓存失败');
    } finally {
      setIsClearingCache(false);
    }
  };

  // 设置相关的处理函数
  const handleAggregateToggle = (value: boolean) => {
    setDefaultAggregateSearch(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('defaultAggregateSearch', JSON.stringify(value));
    }
  };

  const handleDoubanProxyUrlChange = (value: string) => {
    setDoubanProxyUrl(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('doubanProxyUrl', value);
    }
  };

  const handleOptimizationToggle = (value: boolean) => {
    setEnableOptimization(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('enableOptimization', JSON.stringify(value));
    }
  };

  const handleFluidSearchToggle = async (value: boolean) => {
    setFluidSearch(value);

    // 保存到 Tauri 后端
    try {
      await invoke('set_fluid_search', { enabled: value });
    } catch (error) {
      console.error('保存流式搜索设置失败:', error);
    }

    // 同时保存到 localStorage 作为备份
    if (typeof window !== 'undefined') {
      localStorage.setItem('fluidSearch', JSON.stringify(value));
    }
  };

  const handleDoubanDataSourceChange = (value: string) => {
    setDoubanDataSource(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('doubanDataSource', value);
    }
  };

  const handleDoubanImageProxyTypeChange = (value: string) => {
    setDoubanImageProxyType(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('doubanImageProxyType', value);
    }
  };

  const handleDoubanImageProxyUrlChange = (value: string) => {
    setDoubanImageProxyUrl(value);
    if (typeof window !== 'undefined') {
      localStorage.setItem('doubanImageProxyUrl', value);
    }
  };

  // 获取感谢信息
  const getThanksInfo = (dataSource: string) => {
    switch (dataSource) {
      case 'cors-proxy-zwei':
        return {
          text: 'Thanks to @Zwei',
          url: 'https://github.com/bestzwei',
        };
      case 'cmliussss-cdn-tencent':
      case 'cmliussss-cdn-ali':
        return {
          text: 'Thanks to @CMLiussss',
          url: 'https://github.com/cmliu',
        };
      default:
        return null;
    }
  };

  const handleResetSettings = async () => {
    const defaultDoubanProxyType =
      (window as any).RUNTIME_CONFIG?.DOUBAN_PROXY_TYPE ||
      'cmliussss-cdn-tencent';
    const defaultDoubanProxy =
      (window as any).RUNTIME_CONFIG?.DOUBAN_PROXY || '';
    const defaultDoubanImageProxyType =
      (window as any).RUNTIME_CONFIG?.DOUBAN_IMAGE_PROXY_TYPE ||
      'cmliussss-cdn-tencent';
    const defaultDoubanImageProxyUrl =
      (window as any).RUNTIME_CONFIG?.DOUBAN_IMAGE_PROXY || '';
    const defaultFluidSearch =
      (window as any).RUNTIME_CONFIG?.FLUID_SEARCH !== false;

    setDefaultAggregateSearch(true);
    setEnableOptimization(true);
    setFluidSearch(defaultFluidSearch);
    setLiveDirectConnect(false);
    setDoubanProxyUrl(defaultDoubanProxy);
    setDoubanDataSource(defaultDoubanProxyType);
    setDoubanImageProxyType(defaultDoubanImageProxyType);
    setDoubanImageProxyUrl(defaultDoubanImageProxyUrl);
    setPlayerBufferMode('standard');

    // 保存流式搜索到 Tauri 后端
    try {
      await invoke('set_fluid_search', { enabled: defaultFluidSearch });
    } catch (error) {
      console.error('重置流式搜索设置失败:', error);
    }

    if (typeof window !== 'undefined') {
      localStorage.setItem('defaultAggregateSearch', JSON.stringify(true));
      localStorage.setItem('enableOptimization', JSON.stringify(true));
      localStorage.setItem('fluidSearch', JSON.stringify(defaultFluidSearch));
      localStorage.setItem('liveDirectConnect', JSON.stringify(false));
      localStorage.setItem('doubanProxyUrl', defaultDoubanProxy);
      localStorage.setItem('doubanDataSource', defaultDoubanProxyType);
      localStorage.setItem('doubanImageProxyType', defaultDoubanImageProxyType);
      localStorage.setItem('doubanImageProxyUrl', defaultDoubanImageProxyUrl);
      localStorage.setItem('playerBufferMode', 'standard');
    }
  };

  // 菜单面板内容
  const menuPanel = (
    <>
      {/* 背景遮罩 - 普通菜单无需模糊 */}
      <div
        className='fixed inset-0 bg-transparent z-1000'
        onClick={handleCloseMenu}
      />

      {/* 菜单面板 - 固定到视口右上角，使位置稳定且美观 */}
      <div className='fixed top-16 right-4 w-56 bg-white dark:bg-gray-900 rounded-lg shadow-2xl z-1001 border border-slate-200 dark:border-gray-700/50 overflow-hidden select-none'>
        {/* 菜单项 */}
        <div className='py-2'>
          {/* 设置按钮 */}
          <button onClick={() => {
            handleSettings();
            handleCloseMenu();
          }} className='w-full px-3 py-2 text-left flex items-center gap-2.5 text-slate-700 dark:text-gray-300 hover:bg-slate-100 dark:hover:bg-gray-800 transition-colors text-sm'
          >
            <Settings className='w-4 h-4 text-slate-500 dark:text-gray-400' />
            <span className='font-medium'>设置</span>
          </button>

          {/* 管理面板按钮 */}
          <button onClick={() => {
            handleAdminPanel();
            handleCloseMenu();
          }} className='w-full px-3 py-2 text-left flex items-center gap-2.5 text-slate-700 dark:text-gray-300 hover:bg-slate-100 dark:hover:bg-gray-800 transition-colors text-sm'
          >
            <Shield className='w-4 h-4 text-slate-500 dark:text-gray-400' />
            <span className='font-medium'>管理面板</span>
          </button>

          {/* 分割线 */}
          <div className='my-1 border-t border-slate-200 dark:border-gray-700'></div>

          {/* 版本信息 */}
          <button
            onClick={() => {
              setIsVersionPanelOpen(true);
              handleCloseMenu();
            }}
            className='w-full px-3 py-2 text-center flex items-center justify-center text-slate-600 dark:text-gray-400 hover:bg-slate-50 dark:hover:bg-gray-800/50 transition-colors text-xs'
          >
            <div className='flex items-center gap-1'>
              <span className='font-mono'>v{currentVersion}</span>
              {!isChecking &&
                updateStatus &&
                updateStatus.status !== UpdateStatus.FETCH_FAILED && (
                  <div
                    className={`w-2 h-2 rounded-full -translate-y-2 ${
                      updateStatus.status === UpdateStatus.HAS_UPDATE
                        ? 'bg-yellow-500'
                        : updateStatus.status === UpdateStatus.NO_UPDATE
                          ? 'bg-green-400'
                          : ''
                    }`}
                  ></div>
                )}
            </div>
          </button>
        </div>
      </div>
    </>
  );

  // 设置面板内容
  const settingsPanel = (
    <>
      {/* 背景遮罩 */}
      <div
        className='fixed inset-0 bg-black/50 backdrop-blur-sm z-1000'
        onClick={handleCloseSettings}
        onTouchMove={(e) => {
          // 只阻止滚动，允许其他触摸事件
          e.preventDefault();
        }}
        onWheel={(e) => {
          // 阻止滚轮滚动
          e.preventDefault();
        }}
        style={{
          touchAction: 'none',
        }}
      />

      {/* 设置面板 */}
      <div className='fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-xl max-h-[90vh] bg-white dark:bg-gray-900 rounded-xl shadow-xl z-1001 flex flex-col'>
        {/* 内容容器 - 独立的滚动区域 */}
        <div
          className='flex-1 p-6 overflow-y-auto'
          data-panel-content
          style={{
            touchAction: 'pan-y', // 只允许垂直滚动
            overscrollBehavior: 'contain', // 防止滚动冒泡
          }}
        >
          {/* 标题栏 */}
          <div className='flex items-center justify-between mb-6'>
            <div className='flex items-center gap-3'>
              <h3 className='text-xl font-bold text-gray-800 dark:text-gray-200'>
                本地设置
              </h3>
              <button
                onClick={handleResetSettings}
                className='px-2 py-1 text-xs text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 border border-red-200 hover:border-red-300 dark:border-red-800 dark:hover:border-red-700 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors'
                title='重置为默认设置'
              >
                恢复默认
              </button>
            </div>
            <button
              onClick={handleCloseSettings}
              className='w-8 h-8 p-1 rounded-full flex items-center justify-center text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors'
              aria-label='Close'
            >
              <X className='w-full h-full' />
            </button>
          </div>

          {/* 设置项 */}
          <div className='space-y-6'>
            {/* 豆瓣数据源选择 */}
            <div className='space-y-3'>
              <div>
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  豆瓣数据代理
                </h4>
                <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                  选择获取豆瓣数据的方式
                </p>
              </div>
              <div className='relative' data-dropdown='douban-datasource'>
                {/* 自定义下拉选择框 */}
                <button
                  type='button'
                  onClick={() => setIsDoubanDropdownOpen(!isDoubanDropdownOpen)}
                  className='w-full px-3 py-2.5 pr-10 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 shadow-sm hover:border-gray-400 dark:hover:border-gray-500 text-left'
                >
                  {
                    doubanDataSourceOptions.find(
                      (option) => option.value === doubanDataSource,
                    )?.label
                  }
                </button>

                {/* 下拉箭头 */}
                <div className='absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none'>
                  <ChevronDown
                    className={`w-4 h-4 text-gray-400 dark:text-gray-500 transition-transform duration-200 ${
                      isDoubanDropdownOpen ? 'rotate-180' : ''
                    }`}
                  />
                </div>

                {/* 下拉选项列表 */}
                {isDoubanDropdownOpen && (
                  <div className='absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg max-h-60 overflow-auto'>
                    {doubanDataSourceOptions.map((option) => (
                      <button
                        key={option.value}
                        type='button'
                        onClick={() => {
                          handleDoubanDataSourceChange(option.value);
                          setIsDoubanDropdownOpen(false);
                        }}
                        className={`w-full px-3 py-2.5 text-left text-sm transition-colors duration-150 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-700 ${
                          doubanDataSource === option.value
                            ? 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400'
                            : 'text-gray-900 dark:text-gray-100'
                        }`}
                      >
                        <span className='truncate'>{option.label}</span>
                        {doubanDataSource === option.value && (
                          <Check className='w-4 h-4 text-green-600 dark:text-green-400 shrink-0 ml-2' />
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* 感谢信息 */}
              {getThanksInfo(doubanDataSource) && (
                <div className='mt-3'>
                  <button
                    type='button'
                    onClick={() =>
                      window.open(
                        getThanksInfo(doubanDataSource)!.url,
                        '_blank',
                      )
                    }
                    className='flex items-center justify-center gap-1.5 w-full px-3 text-xs text-gray-500 dark:text-gray-400 cursor-pointer'
                  >
                    <span className='font-medium'>
                      {getThanksInfo(doubanDataSource)!.text}
                    </span>
                    <ExternalLink className='w-3.5 opacity-70' />
                  </button>
                </div>
              )}
            </div>

            {/* 豆瓣代理地址设置 - 仅在选择自定义代理时显示 */}
            {doubanDataSource === 'custom' && (
              <div className='space-y-3'>
                <div>
                  <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                    豆瓣代理地址
                  </h4>
                  <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                    自定义代理服务器地址
                  </p>
                </div>
                <input
                  type='text'
                  className='w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 shadow-sm hover:border-gray-400 dark:hover:border-gray-500'
                  placeholder='例如: https://proxy.example.com/fetch?url='
                  value={doubanProxyUrl}
                  onChange={(e) => handleDoubanProxyUrlChange(e.target.value)}
                />
              </div>
            )}

            {/* 分割线 */}
            <div className='border-t border-gray-200 dark:border-gray-700'></div>

            {/* 豆瓣图片代理设置 */}
            <div className='space-y-3'>
              <div>
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  豆瓣图片代理
                </h4>
                <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                  选择获取豆瓣图片的方式
                </p>
              </div>
              <div className='relative' data-dropdown='douban-image-proxy'>
                {/* 自定义下拉选择框 */}
                <button
                  type='button'
                  onClick={() =>
                    setIsDoubanImageProxyDropdownOpen(
                      !isDoubanImageProxyDropdownOpen,
                    )
                  }
                  className='w-full px-3 py-2.5 pr-10 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 shadow-sm hover:border-gray-400 dark:hover:border-gray-500 text-left'
                >
                  {
                    doubanImageProxyTypeOptions.find(
                      (option) => option.value === doubanImageProxyType,
                    )?.label
                  }
                </button>

                {/* 下拉箭头 */}
                <div className='absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none'>
                  <ChevronDown
                    className={`w-4 h-4 text-gray-400 dark:text-gray-500 transition-transform duration-200 ${
                      isDoubanDropdownOpen ? 'rotate-180' : ''
                    }`}
                  />
                </div>

                {/* 下拉选项列表 */}
                {isDoubanImageProxyDropdownOpen && (
                  <div className='absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg max-h-60 overflow-auto'>
                    {doubanImageProxyTypeOptions.map((option) => (
                      <button
                        key={option.value}
                        type='button'
                        onClick={() => {
                          handleDoubanImageProxyTypeChange(option.value);
                          setIsDoubanImageProxyDropdownOpen(false);
                        }}
                        className={`w-full px-3 py-2.5 text-left text-sm transition-colors duration-150 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-700 ${
                          doubanImageProxyType === option.value
                            ? 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400'
                            : 'text-gray-900 dark:text-gray-100'
                        }`}
                      >
                        <span className='truncate'>{option.label}</span>
                        {doubanImageProxyType === option.value && (
                          <Check className='w-4 h-4 text-green-600 dark:text-green-400 shrink-0 ml-2' />
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* 感谢信息 */}
              {getThanksInfo(doubanImageProxyType) && (
                <div className='mt-3'>
                  <button
                    type='button'
                    onClick={() =>
                      window.open(
                        getThanksInfo(doubanImageProxyType)!.url,
                        '_blank',
                      )
                    }
                    className='flex items-center justify-center gap-1.5 w-full px-3 text-xs text-gray-500 dark:text-gray-400 cursor-pointer'
                  >
                    <span className='font-medium'>
                      {getThanksInfo(doubanImageProxyType)!.text}
                    </span>
                    <ExternalLink className='w-3.5 opacity-70' />
                  </button>
                </div>
              )}
            </div>

            {/* 豆瓣图片代理地址设置 - 仅在选择自定义代理时显示 */}
            {doubanImageProxyType === 'custom' && (
              <div className='space-y-3'>
                <div>
                  <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                    豆瓣图片代理地址
                  </h4>
                  <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                    自定义图片代理服务器地址
                  </p>
                </div>
                <input
                  type='text'
                  className='w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500 transition-all duration-200 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 shadow-sm hover:border-gray-400 dark:hover:border-gray-500'
                  placeholder='例如: https://proxy.example.com/fetch?url='
                  value={doubanImageProxyUrl}
                  onChange={(e) =>
                    handleDoubanImageProxyUrlChange(e.target.value)
                  }
                />
              </div>
            )}

            {/* 分割线 */}
            <div className='border-t border-gray-200 dark:border-gray-700'></div>

            {/* 默认聚合搜索结果 */}
            <div className='flex items-center justify-between'>
              <div>
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  默认聚合搜索结果
                </h4>
                <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                  搜索时默认按标题和年份聚合显示结果
                </p>
              </div>
              <label className='flex items-center cursor-pointer'>
                <div className='relative'>
                  <input
                    type='checkbox'
                    className='sr-only peer'
                    checked={defaultAggregateSearch}
                    onChange={(e) => handleAggregateToggle(e.target.checked)}
                  />
                  <div className='w-11 h-6 bg-gray-300 rounded-full peer-checked:bg-green-500 transition-colors dark:bg-gray-600'></div>
                  <div className='absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform peer-checked:translate-x-5'></div>
                </div>
              </label>
            </div>

            {/* 优选和测速 */}
            <div className='flex items-center justify-between'>
              <div>
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  优选和测速
                </h4>
                <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                  如出现播放器劫持问题可关闭
                </p>
              </div>
              <label className='flex items-center cursor-pointer'>
                <div className='relative'>
                  <input
                    type='checkbox'
                    className='sr-only peer'
                    checked={enableOptimization}
                    onChange={(e) => handleOptimizationToggle(e.target.checked)}
                  />
                  <div className='w-11 h-6 bg-gray-300 rounded-full peer-checked:bg-green-500 transition-colors dark:bg-gray-600'></div>
                  <div className='absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform peer-checked:translate-x-5'></div>
                </div>
              </label>
            </div>

            {/* 流式搜索 */}
            <div className='flex items-center justify-between'>
              <div>
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  流式搜索输出
                </h4>
                <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
                  启用搜索结果实时流式输出，关闭后使用传统一次性搜索
                </p>
              </div>
              <label className='flex items-center cursor-pointer'>
                <div className='relative'>
                  <input
                    type='checkbox'
                    className='sr-only peer'
                    checked={fluidSearch}
                    onChange={(e) => handleFluidSearchToggle(e.target.checked)}
                  />
                  <div className='w-11 h-6 bg-gray-300 rounded-full peer-checked:bg-green-500 transition-colors dark:bg-gray-600'></div>
                  <div className='absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform peer-checked:translate-x-5'></div>
                </div>
              </label>
            </div>

            {/* 分割线 */}
            <div className='border-t border-gray-200 dark:border-gray-700'></div>

            {/* 缓存管理 */}
            <div className='space-y-3'>
              <div className='flex items-center gap-2'>
                <Database className='w-4 h-4 text-gray-500 dark:text-gray-400' />
                <h4 className='text-sm font-medium text-gray-700 dark:text-gray-300'>
                  缓存管理
                </h4>
              </div>

              {cacheStats && (
                <div className='grid grid-cols-3 gap-2 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg'>
                  <div className='text-center'>
                    <div className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
                      {cacheStats.total}
                    </div>
                    <div className='text-xs text-gray-500 dark:text-gray-400'>
                      总缓存
                    </div>
                  </div>
                  <div className='text-center'>
                    <div className='text-lg font-semibold text-green-600 dark:text-green-400'>
                      {cacheStats.valid}
                    </div>
                    <div className='text-xs text-gray-500 dark:text-gray-400'>
                      有效
                    </div>
                  </div>
                  <div className='text-center'>
                    <div className='text-lg font-semibold text-orange-600 dark:text-orange-400'>
                      {cacheStats.expired}
                    </div>
                    <div className='text-xs text-gray-500 dark:text-gray-400'>
                      过期
                    </div>
                  </div>
                </div>
              )}

              <div className='flex gap-2'>
                <button
                  onClick={handleCleanupExpiredCache}
                  disabled={isClearingCache}
                  className='flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-sm text-orange-600 dark:text-orange-400 border border-orange-200 dark:border-orange-800 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
                >
                  <Trash2 className='w-4 h-4' />
                  清理过期
                </button>
                <button
                  onClick={handleClearAllCache}
                  disabled={isClearingCache}
                  className='flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-sm text-red-600 dark:text-red-400 border border-red-200 dark:border-red-800 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
                >
                  <Trash2 className='w-4 h-4' />
                  清空所有
                </button>
              </div>

              <p className='text-xs text-gray-500 dark:text-gray-400'>
                缓存有效期 6 小时，包含首页、电影、剧集、动漫、综艺等页面数据
              </p>
            </div>
          </div>
        </div>
      </div>
    </>
  );

  return (
    <>
      <div className='relative'>
        <button
          onClick={handleMenuClick}
          className='w-10 h-10 p-2 rounded-full flex items-center justify-center text-gray-600 hover:bg-gray-200/50 dark:text-gray-300 dark:hover:bg-gray-700/50 transition-colors'
          aria-label='User Menu'
        >
          <User className='w-full h-full' />
        </button>
        {/* 版本状态光点指示器 */}
        {!isChecking && updateStatus && (
          <span className='absolute top-0 right-0 flex h-2.5 w-2.5'>
            {updateStatus.status === UpdateStatus.HAS_UPDATE && (
              <>
                <span className='animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75'></span>
                <span className='relative inline-flex rounded-full h-2.5 w-2.5 bg-orange-500'></span>
              </>
            )}
            {updateStatus.status === UpdateStatus.NO_UPDATE && (
              <>
                <span className='animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75'></span>
                <span className='relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500'></span>
              </>
            )}
          </span>
        )}
      </div>

      {/* 使用 Portal 将菜单面板渲染到 document.body */}
      {isOpen && mounted && createPortal(menuPanel, document.body)}

      {/* 使用 Portal 将设置面板渲染到 document.body */}
      {isSettingsOpen && mounted && createPortal(settingsPanel, document.body)}

      {/* 版本面板 */}
      <VersionPanel
        isOpen={isVersionPanelOpen}
        onClose={() => setIsVersionPanelOpen(false)}
      />
    </>
  );
};
