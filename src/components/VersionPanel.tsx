/* eslint-disable no-console */

'use client';

import { invoke } from '@tauri-apps/api/core';
import {
  CheckCircle,
  Download,
  Loader2,
  RefreshCw,
  X,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

import { UpdateStatus } from '@/lib/types';
interface VersionCheckResult {
  status: UpdateStatus;
  local_timestamp?: string;
  remote_timestamp?: string;
  formatted_local_time?: string;
  formatted_remote_time?: string;
  error?: string;
}

interface RemoteVersionInfo {
  version: string;
  timestamp: string;
  build_time: string;
  release_notes: string[];
  download_url: string;
}

// 获取当前版本
async function getCurrentVersion(): Promise<string> {
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
    return await invoke('check_for_updates');
  } catch (error) {
    console.warn('版本检查失败:', error);
    return {
      status: UpdateStatus.FETCH_FAILED,
      error: error instanceof Error ? error.message : '未知错误',
    };
  }
}
// 获取版本详细信息
async function getVersionForUpdate(): Promise<RemoteVersionInfo | null> {
  try {
    return await invoke('version_for_updates');
  } catch (error) {
    console.warn('获取版本更新信息失败:', error);
    return null;
  }
}

// 版本面板组件
interface VersionPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const VersionPanel: React.FC<VersionPanelProps> = ({
  isOpen,
  onClose,
}) => {
  const [mounted, setMounted] = useState(false);
  const [hasUpdate, setIsHasUpdate] = useState(false);
  const [currentVersion, setCurrentVersion] = useState<string>('0.0.0');
  const [latestVersion, setLatestVersion] = useState<string>('');
  const [versionCheckResult, setVersionCheckResult] = useState<VersionCheckResult | null>(null);
  const [remoteVersionInfo, setRemoteVersionInfo] = useState<RemoteVersionInfo | null>(null);
  const [isCheckingVersion, setIsCheckingVersion] = useState(false);
  const UPDATE_REPO = process.env.NEXT_PUBLIC_UPDATE_REPO || 'Geon97/QuantumTV';
  const REPO_URL = process.env.NEXT_PUBLIC_REPO_URL || `https://github.com/${UPDATE_REPO}`;

  // 确保组件已挂载
  useEffect(() => {
    setMounted(true);
    return () => setMounted(false);
  }, []);

  // 获取当前版本
  useEffect(() => {
    const fetchCurrentVersion = async () => {
      try {
        const version = await getCurrentVersion();
        setCurrentVersion(version);
      } catch (err) {
        console.warn('获取当前版本失败:', err);
      }
    };
    fetchCurrentVersion();
  }, []);

  // Body 滚动锁定 - 使用 overflow 方式避免布局问题
  useEffect(() => {
    if (isOpen) {
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
  }, [isOpen]);
  useEffect(() => {
    if (isOpen) {
      doVersionCheck();
    }
  }, [isOpen]);

  // 执行版本检测
  const doVersionCheck = async () => {
     console.log('🔍 开始执行 doVersionCheck');
  setIsCheckingVersion(true);
  try {
    const result = await checkForUpdates();
    console.log('✅ checkForUpdates 成功返回:', result);
    
    setVersionCheckResult(result);
    
    const hasUpdate = result.status === UpdateStatus.HAS_UPDATE;
    setIsHasUpdate(hasUpdate);
    
    // 如果有更新，获取详细版本信息
    if (hasUpdate) {
      const versionInfo = await getVersionForUpdate();
      setRemoteVersionInfo(versionInfo);
      if (versionInfo?.version) {
        setLatestVersion(versionInfo.version);
      }
    } else {
      console.log('👍 已是最新版本或无更新');
    }
  } catch (error) {
    console.error('错误详情:', error instanceof Error ? error.stack : error);
  } finally {
    setIsCheckingVersion(false);
  }
};

  // 版本面板内容
  const versionPanelContent = (
    <>
      {/* 背景遮罩 */}
      <div
        className='fixed inset-0 bg-black/50 backdrop-blur-sm z-1000'
        onClick={onClose}
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

      {/* 版本面板 */}
      <div
        className='fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-xl max-h-[90vh] bg-white dark:bg-gray-900 rounded-xl shadow-xl z-1001 overflow-hidden'
        onTouchMove={(e) => {
          // 允许版本面板内部滚动，阻止事件冒泡到外层
          e.stopPropagation();
        }}
        style={{
          touchAction: 'auto', // 允许面板内的正常触摸操作
        }}
      >
        {/* 标题栏 */}
        <div className='flex items-center justify-between p-3 sm:p-6 border-b border-gray-200 dark:border-gray-700'>
          <div className='flex items-center gap-2 sm:gap-3'>
            <h3 className='text-lg sm:text-xl font-bold text-gray-800 dark:text-gray-200'>
              版本信息
            </h3>
            <div className='flex flex-wrap items-center gap-1 sm:gap-2'>
              <span className='px-2 sm:px-3 py-1 text-xs sm:text-sm font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 rounded-full'>
                v{currentVersion}
              </span>
              {hasUpdate && (
                <span className='px-2 sm:px-3 py-1 text-xs sm:text-sm font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300 rounded-full flex items-center gap-1'>
                  <Download className='w-3 h-3 sm:w-4 sm:h-4' />
                  <span className='hidden sm:inline'>有新版本可用</span>
                  <span className='sm:hidden'>可更新</span>
                </span>
              )}
            </div>
          </div>
          <button
            onClick={onClose}
            className='w-6 h-6 sm:w-8 sm:h-8 p-1 rounded-full flex items-center justify-center text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors'
            aria-label='关闭'
          >
            <X className='w-full h-full' />
          </button>
        </div>

        {/* 内容区域 */}
        <div className='p-3 sm:p-6 overflow-y-auto max-h-[calc(95vh-140px)] sm:max-h-[calc(90vh-120px)]'>
          <div className='space-y-3 sm:space-y-6'>
            {/* 版本检测状态 - 检测中 */}
            {isCheckingVersion && (
              <div className='bg-linear-to-r from-gray-50 to-slate-50 dark:from-gray-900/20 dark:to-slate-900/20 border border-gray-200 dark:border-gray-700 rounded-lg p-3 sm:p-4'>
                <div className='flex items-center gap-3'>
                  <div className='w-8 h-8 sm:w-10 sm:h-10 bg-gray-100 dark:bg-gray-800/40 rounded-full flex items-center justify-center shrink-0'>
                    <Loader2 className='w-4 h-4 sm:w-5 sm:h-5 text-gray-500 dark:text-gray-400 animate-spin' />
                  </div>
                  <div className='min-w-0 flex-1'>
                    <h4 className='text-sm sm:text-base font-semibold text-gray-700 dark:text-gray-300'>
                      正在检测版本...
                    </h4>
                  </div>
                </div>
              </div>
            )}

            {/* 远程更新信息 - 有新版本 */}
            {!isCheckingVersion && hasUpdate && remoteVersionInfo && (
              <div className='bg-linear-to-r from-yellow-50 to-amber-50 dark:from-yellow-900/20 dark:to-amber-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3 sm:p-4'>
                <div className='flex flex-col gap-3'>
                  <div className='flex items-center gap-2 sm:gap-3'>
                    <div className='relative w-8 h-8 sm:w-10 sm:h-10 bg-yellow-100 dark:bg-yellow-800/40 rounded-full flex items-center justify-center shrink-0'>
                      <Download className='w-4 h-4 sm:w-5 sm:h-5 text-yellow-600 dark:text-yellow-400' />
                      {/* 脉冲光点 */}
                      <span className='absolute -top-0.5 -right-0.5 flex h-3 w-3'>
                        <span className='animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75'></span>
                        <span className='relative inline-flex rounded-full h-3 w-3 bg-orange-500'></span>
                      </span>
                    </div>
                    <div className='min-w-0 flex-1'>
                      <h4 className='text-sm sm:text-base font-semibold text-yellow-800 dark:text-yellow-200'>
                        发现新版本
                      </h4>
                      <p className='text-xs sm:text-sm text-yellow-700 dark:text-yellow-300 break-all'>
                        v{currentVersion} → v{remoteVersionInfo.version}
                      </p>
                      {remoteVersionInfo.build_time && (
                        <p className='text-xs text-yellow-600 dark:text-yellow-400 mt-1'>
                          发布时间: {remoteVersionInfo.build_time}
                        </p>
                      )}
                    </div>
                  </div>
                  <a
                    href={remoteVersionInfo.download_url || REPO_URL}
                    target='_blank'
                    rel='noopener noreferrer'
                    className='inline-flex items-center justify-center gap-2 px-3 py-2 bg-yellow-600 hover:bg-yellow-700 text-white text-xs sm:text-sm rounded-lg transition-colors shadow-sm w-full'
                  >
                    <Download className='w-3 h-3 sm:w-4 sm:h-4' />
                    前往更新
                  </a>
                </div>
              </div>
            )}

            {/* 当前为最新版本信息 */}
            {!isCheckingVersion &&
              !hasUpdate &&
              versionCheckResult?.status === UpdateStatus.NO_UPDATE && (
                <div className='bg-linear-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3 sm:p-4'>
                  <div className='flex flex-col gap-3'>
                    <div className='flex items-center gap-2 sm:gap-3'>
                      <div className='relative w-8 h-8 sm:w-10 sm:h-10 bg-green-100 dark:bg-green-800/40 rounded-full flex items-center justify-center shrink-0'>
                        <CheckCircle className='w-4 h-4 sm:w-5 sm:h-5 text-green-600 dark:text-green-400' />
                        {/* 绿色光点 */}
                        <span className='absolute -top-0.5 -right-0.5 flex h-3 w-3'>
                          <span className='animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75'></span>
                          <span className='relative inline-flex rounded-full h-3 w-3 bg-emerald-500'></span>
                        </span>
                      </div>
                      <div className='min-w-0 flex-1'>
                        <h4 className='text-sm sm:text-base font-semibold text-green-800 dark:text-green-200'>
                          当前为最新版本
                        </h4>
                        <p className='text-xs sm:text-sm text-green-700 dark:text-green-300 break-all'>
                          已是最新版本 v{currentVersion}
                        </p>
                        {versionCheckResult?.formatted_local_time && (
                          <p className='text-xs text-green-600 dark:text-green-400 mt-1'>
                            构建时间: {versionCheckResult.formatted_local_time}
                          </p>
                        )}
                      </div>
                    </div>
                    <a
                      href={REPO_URL || '#'}
                      target='_blank'
                      rel='noopener noreferrer'
                      className='inline-flex items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 text-white text-xs sm:text-sm rounded-lg transition-colors shadow-sm w-full'
                    >
                      <CheckCircle className='w-3 h-3 sm:w-4 sm:h-4' />
                      前往仓库
                    </a>
                  </div>
                </div>
              )}

            {/* 检测失败 */}
            {!isCheckingVersion &&
              versionCheckResult?.status === UpdateStatus.FETCH_FAILED && (
                <div className='bg-linear-to-r from-red-50 to-rose-50 dark:from-red-900/20 dark:to-rose-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3 sm:p-4'>
                  <div className='flex flex-col gap-3'>
                    <div className='flex items-center gap-2 sm:gap-3'>
                      <div className='relative w-8 h-8 sm:w-10 sm:h-10 bg-red-100 dark:bg-red-800/40 rounded-full flex items-center justify-center shrink-0'>
                        <X className='w-4 h-4 sm:w-5 sm:h-5 text-red-600 dark:text-red-400' />
                      </div>
                      <div className='min-w-0 flex-1'>
                        <h4 className='text-sm sm:text-base font-semibold text-red-800 dark:text-red-200'>
                          版本检测失败
                        </h4>
                        <p className='text-xs sm:text-sm text-red-700 dark:text-red-300 break-all'>
                          {versionCheckResult?.error || '无法连接到更新服务器'}
                        </p>
                      </div>
                    </div>
                    <button
                      onClick={doVersionCheck}
                      className='inline-flex items-center justify-center gap-2 px-3 py-2 bg-red-600 hover:bg-red-700 text-white text-xs sm:text-sm rounded-lg transition-colors shadow-sm w-full'
                    >
                      <RefreshCw className='w-3 h-3 sm:w-4 sm:h-4' />
                      重试检测
                    </button>
                  </div>
                </div>
              )}
          </div>
        </div>
      </div>
    </>
  );

  // 使用 Portal 渲染到 document.body
  if (!mounted || !isOpen) return null;

  return createPortal(versionPanelContent, document.body);
};
