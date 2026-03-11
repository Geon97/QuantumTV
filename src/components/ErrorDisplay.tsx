'use client';

import {
  AlertCircle,
  Database,
  FileX,
  Search,
  Settings,
  WifiOff,
  XCircle,
} from 'lucide-react';
import { useEffect, useState } from 'react';

export interface PlayerInitError {
  type:
    | 'network_timeout'
    | 'source_unreachable'
    | 'parse_failed'
    | 'no_available_sources'
    | 'search_failed'
    | 'database_error'
    | 'config_error';
  source?: string;
  timeout_ms?: number;
  stage?: string;
  reason?: string;
  http_status?: number;
  error?: string;
  content_preview?: string;
  attempted_sources?: string[];
  last_error?: string;
  query?: string;
  operation?: string;
  field?: string;
}

interface ErrorDisplayProps {
  error: PlayerInitError | string | null;
  onRetry?: () => void;
  onDismiss?: () => void;
  autoHide?: boolean;
  autoHideDelay?: number;
}

export default function ErrorDisplay({
  error,
  onRetry,
  onDismiss,
  autoHide = false,
  autoHideDelay = 5000,
}: ErrorDisplayProps) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    if (autoHide && error) {
      const timer = setTimeout(() => {
        setVisible(false);
        onDismiss?.();
      }, autoHideDelay);
      return () => clearTimeout(timer);
    }
  }, [error, autoHide, autoHideDelay, onDismiss]);

  useEffect(() => {
    setVisible(true);
  }, [error]);

  if (!error || !visible) return null;

  // 处理字符串错误
  if (typeof error === 'string') {
    return (
      <div className='fixed top-4 right-4 z-50 max-w-md animate-slide-in-right'>
        <div className='bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 shadow-lg'>
          <div className='flex items-start gap-3'>
            <AlertCircle className='w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5' />
            <div className='flex-1'>
              <h3 className='text-sm font-semibold text-red-900 dark:text-red-100 mb-1'>
                错误
              </h3>
              <p className='text-sm text-red-800 dark:text-red-200'>{error}</p>
            </div>
            {onDismiss && (
              <button
                onClick={() => {
                  setVisible(false);
                  onDismiss();
                }}
                className='text-red-400 hover:text-red-600 dark:text-red-500 dark:hover:text-red-300'
              >
                <XCircle className='w-5 h-5' />
              </button>
            )}
          </div>
          {onRetry && (
            <button
              onClick={onRetry}
              className='mt-3 w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium'
            >
              重试
            </button>
          )}
        </div>
      </div>
    );
  }

  // 处理结构化错误
  const getErrorConfig = (error: PlayerInitError) => {
    switch (error.type) {
      case 'network_timeout':
        return {
          icon: WifiOff,
          color: 'orange',
          title: '网络超时',
          message: `源 ${error.source} 在 ${error.stage} 阶段超时（${error.timeout_ms}ms），请检查网络连接`,
          suggestion: '请检查网络连接或稍后重试',
          canRetry: true,
        };
      case 'source_unreachable':
        return {
          icon: WifiOff,
          color: 'red',
          title: '源不可达',
          message: error.http_status
            ? `源 ${error.source} 不可达（HTTP ${error.http_status}）：${error.reason}`
            : `源 ${error.source} 不可达：${error.reason}`,
          suggestion: '该视频源可能暂时不可用，请尝试切换其他源',
          canRetry: true,
        };
      case 'parse_failed':
        return {
          icon: FileX,
          color: 'yellow',
          title: '数据解析失败',
          message: `源 ${error.source} 数据解析失败：${error.error}`,
          suggestion: '视频源返回的数据格式异常，请尝试其他源',
          canRetry: false,
        };
      case 'no_available_sources':
        return {
          icon: AlertCircle,
          color: 'red',
          title: '无可用源',
          message: `所有源均不可用（尝试了：${error.attempted_sources?.join(', ')}）`,
          suggestion: error.last_error || '请稍后重试或搜索其他内容',
          canRetry: true,
        };
      case 'search_failed':
        return {
          icon: Search,
          color: 'blue',
          title: '搜索失败',
          message: `搜索 "${error.query}" 失败：${error.reason}`,
          suggestion: '请检查搜索关键词或稍后重试',
          canRetry: true,
        };
      case 'database_error':
        return {
          icon: Database,
          color: 'purple',
          title: '数据库错误',
          message: `数据库操作 ${error.operation} 失败：${error.error}`,
          suggestion: '数据库操作失败，请重启应用或联系支持',
          canRetry: false,
        };
      case 'config_error':
        return {
          icon: Settings,
          color: 'gray',
          title: '配置错误',
          message: `配置项 ${error.field} 错误：${error.error}`,
          suggestion: '请检查应用配置或重置为默认设置',
          canRetry: false,
        };
      default:
        return {
          icon: AlertCircle,
          color: 'red',
          title: '未知错误',
          message: '发生了未知错误',
          suggestion: '请重试或联系支持',
          canRetry: true,
        };
    }
  };

  const config = getErrorConfig(error);
  const Icon = config.icon;

  const colorClasses = {
    red: {
      bg: 'bg-red-50 dark:bg-red-900/20',
      border: 'border-red-200 dark:border-red-800',
      icon: 'text-red-600 dark:text-red-400',
      title: 'text-red-900 dark:text-red-100',
      message: 'text-red-800 dark:text-red-200',
      suggestion: 'text-red-700 dark:text-red-300',
      button: 'bg-red-600 hover:bg-red-700',
    },
    orange: {
      bg: 'bg-orange-50 dark:bg-orange-900/20',
      border: 'border-orange-200 dark:border-orange-800',
      icon: 'text-orange-600 dark:text-orange-400',
      title: 'text-orange-900 dark:text-orange-100',
      message: 'text-orange-800 dark:text-orange-200',
      suggestion: 'text-orange-700 dark:text-orange-300',
      button: 'bg-orange-600 hover:bg-orange-700',
    },
    yellow: {
      bg: 'bg-yellow-50 dark:bg-yellow-900/20',
      border: 'border-yellow-200 dark:border-yellow-800',
      icon: 'text-yellow-600 dark:text-yellow-400',
      title: 'text-yellow-900 dark:text-yellow-100',
      message: 'text-yellow-800 dark:text-yellow-200',
      suggestion: 'text-yellow-700 dark:text-yellow-300',
      button: 'bg-yellow-600 hover:bg-yellow-700',
    },
    blue: {
      bg: 'bg-blue-50 dark:bg-blue-900/20',
      border: 'border-blue-200 dark:border-blue-800',
      icon: 'text-blue-600 dark:text-blue-400',
      title: 'text-blue-900 dark:text-blue-100',
      message: 'text-blue-800 dark:text-blue-200',
      suggestion: 'text-blue-700 dark:text-blue-300',
      button: 'bg-blue-600 hover:bg-blue-700',
    },
    purple: {
      bg: 'bg-purple-50 dark:bg-purple-900/20',
      border: 'border-purple-200 dark:border-purple-800',
      icon: 'text-purple-600 dark:text-purple-400',
      title: 'text-purple-900 dark:text-purple-100',
      message: 'text-purple-800 dark:text-purple-200',
      suggestion: 'text-purple-700 dark:text-purple-300',
      button: 'bg-purple-600 hover:bg-purple-700',
    },
    gray: {
      bg: 'bg-gray-50 dark:bg-gray-900/20',
      border: 'border-gray-200 dark:border-gray-800',
      icon: 'text-gray-600 dark:text-gray-400',
      title: 'text-gray-900 dark:text-gray-100',
      message: 'text-gray-800 dark:text-gray-200',
      suggestion: 'text-gray-700 dark:text-gray-300',
      button: 'bg-gray-600 hover:bg-gray-700',
    },
  };

  const colors = colorClasses[config.color as keyof typeof colorClasses];

  return (
    <div className='fixed top-4 right-4 z-50 max-w-md animate-slide-in-right'>
      <div
        className={`${colors.bg} border ${colors.border} rounded-lg p-4 shadow-lg`}
      >
        <div className='flex items-start gap-3'>
          <Icon className={`w-5 h-5 ${colors.icon} flex-shrink-0 mt-0.5`} />
          <div className='flex-1'>
            <h3 className={`text-sm font-semibold ${colors.title} mb-1`}>
              {config.title}
            </h3>
            <p className={`text-sm ${colors.message} mb-2`}>{config.message}</p>
            <p className={`text-xs ${colors.suggestion}`}>
              💡 {config.suggestion}
            </p>
          </div>
          {onDismiss && (
            <button
              onClick={() => {
                setVisible(false);
                onDismiss();
              }}
              className={`${colors.icon} hover:opacity-70`}
            >
              <XCircle className='w-5 h-5' />
            </button>
          )}
        </div>
        {onRetry && config.canRetry && (
          <button
            onClick={onRetry}
            className={`mt-3 w-full px-4 py-2 ${colors.button} text-white rounded-lg transition-colors text-sm font-medium`}
          >
            重试
          </button>
        )}
      </div>
    </div>
  );
}

// 添加动画样式到全局 CSS
// @keyframes slide-in-right {
//   from {
//     transform: translateX(100%);
//     opacity: 0;
//   }
//   to {
//     transform: translateX(0);
//     opacity: 1;
//   }
// }
// .animate-slide-in-right {
//   animation: slide-in-right 0.3s ease-out;
// }
