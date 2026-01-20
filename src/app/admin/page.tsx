/* eslint-disable @typescript-eslint/no-explicit-any, no-console */

'use client';

import {
  closestCenter,
  DndContext,
  PointerSensor,
  TouchSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  restrictToParentElement,
  restrictToVerticalAxis,
} from '@dnd-kit/modifiers';
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { invoke } from '@tauri-apps/api/core';
import {
  AlertCircle,
  AlertTriangle,
  Check,
  CheckCircle,
  ChevronDown,
  ChevronUp,
  Cloud,
  Database,
  FolderOpen,
  GripVertical,
  Plus,
  RefreshCw,
  Settings,
  Trash2,
  Video,
  X,
} from 'lucide-react';
import { Suspense, useCallback, useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

import { AdminConfig } from '@/lib/admin.types';

import DatabaseImportExport from '@/components/DatabaseImportExport';
import PageLayout from '@/components/PageLayout';

// 配置导入/导出弹窗组件
interface ConfigImportExportModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: AdminConfig | null;
  onImport: (config: AdminConfig) => void;
}

const ConfigImportExportModal = ({
  isOpen,
  onClose,
  config,
  // onImport,
}: ConfigImportExportModalProps) => {
  const [importText, setImportText] = useState('');
  const [activeTab, setActiveTab] = useState<'import' | 'export'>('export');

  if (!isOpen) return null;

  const handleExport = () => {
    if (!config) return;
    const dataStr = JSON.stringify(config, null, 2);
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `quantumtv-config-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    try {
      const parsed = JSON.parse(importText);

      // 智能识别和转换配置格式
      let finalConfig: AdminConfig;

      // 情况1: 完整的 AdminConfig 格式
      if (parsed.SourceConfig !== undefined || parsed.ConfigSubscribtion !== undefined || parsed.SiteConfig !== undefined) {
        // 合并到默认配置，确保所有字段都存在
        const defaultConfig = getDefaultConfig();
        finalConfig = {
          ...defaultConfig,
          ...parsed,
          ConfigSubscribtion: { ...defaultConfig.ConfigSubscribtion, ...parsed.ConfigSubscribtion },
          SiteConfig: { ...defaultConfig.SiteConfig, ...parsed.SiteConfig },
          UserConfig: parsed.UserConfig || defaultConfig.UserConfig,
          SourceConfig: Array.isArray(parsed.SourceConfig)
            ? parsed.SourceConfig.map((s: any) => ({ ...s, from: s.from || 'custom' }))
            : defaultConfig.SourceConfig,
          CustomCategories: Array.isArray(parsed.CustomCategories)
            ? parsed.CustomCategories.map((c: any) => ({ ...c, from: c.from || 'custom' }))
            : defaultConfig.CustomCategories,
        };

        // 如果 SourceConfig 为空但 ConfigFile 有内容，尝试从 ConfigFile 中解析
        if ((!finalConfig.SourceConfig || finalConfig.SourceConfig.length === 0) && parsed.ConfigFile) {
          try {
            const configFileContent = JSON.parse(parsed.ConfigFile);
            // 处理 api_site 对象格式
            if (configFileContent.api_site && typeof configFileContent.api_site === 'object') {
              finalConfig.SourceConfig = Object.entries(configFileContent.api_site).map(([key, value]: [string, any]) => ({
                key: key,
                name: value.name || key,
                api: value.api,
                detail: value.detail || '',
                from: 'config' as const,
                disabled: value.disabled || false,
                is_adult: value.is_adult || false,
              }));
            }
            // 处理 sites 数组格式
            else if (configFileContent.sites && Array.isArray(configFileContent.sites)) {
              finalConfig.SourceConfig = configFileContent.sites.map((s: any) => ({
                key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
                name: s.name || '未命名源',
                api: s.api,
                detail: s.detail || '',
                from: 'config' as const,
                disabled: s.disabled || false,
                is_adult: s.is_adult || false,
              }));
            }
            // 处理 SourceConfig 数组格式
            else if (configFileContent.SourceConfig && Array.isArray(configFileContent.SourceConfig)) {
              finalConfig.SourceConfig = configFileContent.SourceConfig.map((s: any) => ({
                ...s,
                from: s.from || 'config',
              }));
            }
          } catch {
            console.warn('无法解析 ConfigFile 内容');
          }
        }
      }
      // 情况2: 纯数组格式 - 作为 SourceConfig 处理
      else if (Array.isArray(parsed)) {
        const defaultConfig = getDefaultConfig();
        // 检查是否为视频源数组 (包含 api 字段)
        if (parsed.length > 0 && parsed[0].api) {
          finalConfig = {
            ...defaultConfig,
            SourceConfig: parsed.map((s: any) => ({
              key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
              name: s.name || '未命名源',
              api: s.api,
              detail: s.detail || '',
              from: 'custom' as const,
              disabled: s.disabled || false,
              is_adult: s.is_adult || false,
            })),
          };
        }
      }
      // 情况3: 包含 sites 字段的格式 (常见的订阅配置格式)
      else if (parsed.sites && Array.isArray(parsed.sites)) {
        const defaultConfig = getDefaultConfig();
        finalConfig = {
          ...defaultConfig,
          SourceConfig: parsed.sites.map((s: any) => ({
            key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
            name: s.name || '未命名源',
            api: s.api,
            detail: s.detail || '',
            from: 'config' as const,
            disabled: s.disabled || false,
            is_adult: s.is_adult || false,
          })),
        };
      }
      // 情况4: 包含 api_site 对象格式 (键值对形式)
      else if (parsed.api_site && typeof parsed.api_site === 'object') {
        const defaultConfig = getDefaultConfig();
        finalConfig = {
          ...defaultConfig,
          SourceConfig: Object.entries(parsed.api_site).map(([key, value]: [string, any]) => ({
            key: key,
            name: value.name || key,
            api: value.api,
            detail: value.detail || '',
            from: 'config' as const,
            disabled: value.disabled || false,
            is_adult: value.is_adult || false,
          })),
        };
      }
      // 情况5: 无法识别的格式
      else {
        alert('无法识别的配置格式，请确保包含 SourceConfig、sites 或 api_site 字段');
        return;
      }

      setImportText('');
      onClose();
    } catch {
      alert('配置格式错误，请检查 JSON 格式');
    }
  };

  const handleFileImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const text = event.target?.result as string;
      setImportText(text);
    };
    reader.readAsText(file);
  };

  return createPortal(
    <div className='fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4'>
      <div className='bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full p-6'>
        <div className='flex justify-between items-center mb-4'>
          <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
            配置导入/导出
          </h3>
          <button onClick={onClose}>
            <X className='w-5 h-5 text-gray-500' />
          </button>
        </div>

        {/* 标签切换 */}
        <div className='flex gap-2 mb-4'>
          <button
            onClick={() => setActiveTab('export')}
            className={`flex-1 py-2 px-4 rounded-lg transition-colors ${
              activeTab === 'export'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
            }`}
          >
            导出配置
          </button>
          <button
            onClick={() => setActiveTab('import')}
            className={`flex-1 py-2 px-4 rounded-lg transition-colors ${
              activeTab === 'import'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
            }`}
          >
            导入配置
          </button>
        </div>

        {activeTab === 'export' ? (
          <div className='space-y-4'>
            <p className='text-sm text-gray-600 dark:text-gray-400'>
              将当前配置导出为 JSON 文件，可用于备份或迁移到其他设备。
            </p>
            <button
              onClick={handleExport}
              className='w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors'
            >
              下载配置文件
            </button>
          </div>
        ) : (
          <div className='space-y-4'>
            <p className='text-sm text-gray-600 dark:text-gray-400'>
              从 JSON 文件导入配置。导入后将覆盖当前所有配置。
            </p>
            <div>
              <input
                type='file'
                accept='.json'
                onChange={handleFileImport}
                className='w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100 dark:file:bg-blue-900/30 dark:file:text-blue-400'
              />
            </div>
            <textarea
              value={importText}
              onChange={(e) => setImportText(e.target.value)}
              placeholder='或者直接粘贴 JSON 配置内容...'
              className='w-full h-40 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm resize-none'
            />
            <button
              onClick={handleImport}
              disabled={!importText.trim()}
              className='w-full py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white rounded-lg transition-colors'
            >
              导入配置
            </button>
          </div>
        )}
      </div>
    </div>,
    document.body
  );
};

// localStorage 配置键
const LOCAL_CONFIG_KEY = 'quantumtv_admin_config';

// 获取默认配置
function getDefaultConfig(): AdminConfig {
  return {
    ConfigFile: '',
    ConfigSubscribtion: {
      URL: '',
      AutoUpdate: false,
      LastCheck: '',
    },
    SiteConfig: {
      SiteName: 'QuantumTV',
      Announcement: '本应用仅提供影视信息搜索服务，所有内容均来自第三方网站。',
      SearchDownstreamMaxPage: 5,
      SiteInterfaceCacheTime: 7200,
      DoubanProxyType: 'cmliussss-cdn-tencent',
      DoubanProxy: '',
      DoubanImageProxyType: 'cmliussss-cdn-tencent',
      DoubanImageProxy: '',
      DisableYellowFilter: false,
      FluidSearch: true,
    },
    UserConfig: {
      Users: [{ username: 'admin', role: 'owner', banned: false }],
    },
    SourceConfig: [],
    CustomCategories: [],
  };
}

// 通用弹窗组件
export interface AlertModalProps {
  isOpen: boolean;
  onClose: () => void;
  type: 'success' | 'error' | 'warning';
  title: string;
  message?: string;
  timer?: number;
}

export const AlertModal = ({
  isOpen,
  onClose,
  type,
  title,
  message,
  timer,
}: AlertModalProps) => {
  const [isVisible, setIsVisible] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (isOpen) {
      setIsVisible(true);
      if (timer) {
        const timeoutId = setTimeout(onClose, timer);
        return () => clearTimeout(timeoutId);
      }
    } else {
      setIsVisible(false);
    }
  }, [isOpen, timer, onClose]);

  if (!mounted || !isOpen) return null;

  const getIcon = () => {
    switch (type) {
      case 'success':
        return <CheckCircle className='w-8 h-8 text-green-500' />;
      case 'error':
        return <AlertCircle className='w-8 h-8 text-red-500' />;
      case 'warning':
        return <AlertTriangle className='w-8 h-8 text-yellow-500' />;
    }
  };

  const getBgColor = () => {
    switch (type) {
      case 'success':
        return 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800';
      case 'error':
        return 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800';
      case 'warning':
        return 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800';
    }
  };

  return createPortal(
    <div
      className={`fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4 transition-opacity duration-200 ${isVisible ? 'opacity-100' : 'opacity-0'}`}
      onClick={onClose}
    >
      <div
        className={`bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-sm w-full border ${getBgColor()} transition-all duration-200 ${isVisible ? 'scale-100' : 'scale-95'}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className='p-6 text-center'>
          <div className='flex justify-center mb-4'>{getIcon()}</div>
          <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2'>
            {title}
          </h3>
          {message && (
            <p className='text-sm text-gray-600 dark:text-gray-400'>{message}</p>
          )}
          <button
            onClick={onClose}
            className='mt-4 px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors'
          >
            确定
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
};

// 可折叠标签组件
interface CollapsibleTabProps {
  title: string;
  icon: React.ReactNode;
  isExpanded: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

const CollapsibleTab = ({
  title,
  icon,
  isExpanded,
  onToggle,
  children,
}: CollapsibleTabProps) => (
  <div className='bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden'>
    <button
      onClick={onToggle}
      className='w-full flex items-center justify-between p-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors'
    >
      <div className='flex items-center gap-3'>
        {icon}
        <span className='font-semibold text-base text-gray-900 dark:text-gray-100'>{title}</span>
      </div>
      {isExpanded ? (
        <ChevronUp className='w-5 h-5 text-gray-500' />
      ) : (
        <ChevronDown className='w-5 h-5 text-gray-500' />
      )}
    </button>
    {isExpanded && (
      <div className='p-4 border-t border-gray-200 dark:border-gray-700'>
        {children}
      </div>
    )}
  </div>
);

// 可拖拽的源项组件
interface SortableSourceItemProps {
  source: any;
  onToggle: () => void;
  onDelete: () => void;
  onEdit: () => void;
}

const SortableSourceItem = ({
  source,
  onToggle,
  onDelete,
  onEdit,
}: SortableSourceItemProps) => {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: source.key });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg ${isDragging ? 'z-50' : ''}`}
    >
      <button
        {...attributes}
        {...listeners}
        className='cursor-grab active:cursor-grabbing text-gray-400 hover:text-gray-600 dark:hover:text-gray-300'
      >
        <GripVertical className='w-5 h-5' />
      </button>

      <div className='flex-1 min-w-0'>
        <div className='flex items-center gap-2'>
          <span className='text-sm font-medium text-gray-900 dark:text-gray-100 truncate'>
            {source.name}
          </span>
          {source.is_adult && (
            <span className='px-1.5 py-0.5 text-xs bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400 rounded'>
              18+
            </span>
          )}
        </div>
        <p className='text-xs text-gray-500 dark:text-gray-400 truncate'>
          {source.api}
        </p>
      </div>

      <div className='flex items-center gap-2'>
        <button
          onClick={onEdit}
          className='p-1.5 text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 transition-colors'
          title='编辑'
        >
          <Settings className='w-4 h-4' />
        </button>
        <button
          onClick={onToggle}
          className={`relative w-10 h-5 rounded-full transition-colors ${source.disabled ? 'bg-gray-300 dark:bg-gray-600' : 'bg-green-500'}`}
          title={source.disabled ? '启用' : '禁用'}
        >
          <span
            className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform ${source.disabled ? 'left-0.5' : 'left-5'}`}
          />
        </button>
        <button
          onClick={onDelete}
          className='p-1.5 text-gray-500 hover:text-red-600 dark:hover:text-red-400 transition-colors'
          title='删除'
        >
          <Trash2 className='w-4 h-4' />
        </button>
      </div>
    </div>
  );
};

// 视频源配置组件
interface SourceConfigProps {
  config: AdminConfig | null;
  onSave: (config: AdminConfig) => void;
  showAlert: (type: 'success' | 'error' | 'warning', title: string, message?: string) => void;
}

const SourceConfig = ({ config, onSave, showAlert }: SourceConfigProps) => {
  const [sources, setSources] = useState<any[]>([]);
  const [editingSource, setEditingSource] = useState<any | null>(null);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newSource, setNewSource] = useState({ key: '', name: '', api: '', detail: '', is_adult: false });

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 5 } })
  );

  useEffect(() => {
    // 确保当 config 变化时始终同步 sources 状态
    const sourceConfig = config?.SourceConfig || [];
    setSources([...sourceConfig]);
  }, [config?.SourceConfig]);

  const handleDragEnd = (event: any) => {
    const { active, over } = event;
    if (active.id !== over?.id) {
      const oldIndex = sources.findIndex((s) => s.key === active.id);
      const newIndex = sources.findIndex((s) => s.key === over.id);
      const newSources = arrayMove(sources, oldIndex, newIndex);
      setSources(newSources);
      saveChanges(newSources);
    }
  };

  const saveChanges = (newSources: any[]) => {
    if (!config) return;
    const newConfig = { ...config, SourceConfig: newSources };
    onSave(newConfig);
  };

  const handleToggle = (key: string) => {
    const newSources = sources.map((s) =>
      s.key === key ? { ...s, disabled: !s.disabled } : s
    );
    setSources(newSources);
    saveChanges(newSources);
  };

  const handleDelete = (key: string) => {
    const newSources = sources.filter((s) => s.key !== key);
    setSources(newSources);
    saveChanges(newSources);
    showAlert('success', '删除成功');
  };

  const handleAdd = () => {
    if (!newSource.key || !newSource.name || !newSource.api) {
      showAlert('error', '请填写完整信息');
      return;
    }
    if (sources.some((s) => s.key === newSource.key)) {
      showAlert('error', '源标识已存在');
      return;
    }
    const newSources = [...sources, { ...newSource, from: 'custom', disabled: false }];
    setSources(newSources);
    saveChanges(newSources);
    setNewSource({ key: '', name: '', api: '', detail: '', is_adult: false });
    setIsAddModalOpen(false);
    showAlert('success', '添加成功');
  };

  const handleEdit = (source: any) => {
    setEditingSource({ ...source });
  };

  const handleSaveEdit = () => {
    if (!editingSource) return;
    const newSources = sources.map((s) =>
      s.key === editingSource.key ? editingSource : s
    );
    setSources(newSources);
    saveChanges(newSources);
    setEditingSource(null);
    showAlert('success', '保存成功');
  };

  return (
    <div className='space-y-4'>
      <div className='flex justify-between items-center'>
        <p className='text-sm text-gray-600 dark:text-gray-400'>
          共 {sources.length} 个视频源，{sources.filter((s) => !s.disabled).length} 个已启用
        </p>
        <button
          onClick={() => setIsAddModalOpen(true)}
          className='flex items-center gap-1.5 px-3 py-1.5 bg-green-600 hover:bg-green-700 text-white text-sm rounded-lg transition-colors'
        >
          <Plus className='w-4 h-4' />
          添加源
        </button>
      </div>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        modifiers={[restrictToVerticalAxis, restrictToParentElement]}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={sources.map((s) => s.key)} strategy={verticalListSortingStrategy}>
          <div className='space-y-2'>
            {sources.map((source) => (
              <SortableSourceItem
                key={source.key}
                source={source}
                onToggle={() => handleToggle(source.key)}
                onDelete={() => handleDelete(source.key)}
                onEdit={() => handleEdit(source)}
              />
            ))}
          </div>
        </SortableContext>
      </DndContext>

      {sources.length === 0 && (
        <div className='text-center py-8 text-gray-500 dark:text-gray-400'>
          <Database className='w-12 h-12 mx-auto mb-3 opacity-50' />
          <p>暂无视频源，点击上方按钮添加</p>
        </div>
      )}

      {/* 添加源弹窗 */}
      {isAddModalOpen && (
        <div className='fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4'>
          <div className='bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6'>
            <div className='flex justify-between items-center mb-4'>
              <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
                添加视频源
              </h3>
              <button onClick={() => setIsAddModalOpen(false)}>
                <X className='w-5 h-5 text-gray-500' />
              </button>
            </div>
            <div className='space-y-4'>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  源标识 (key)
                </label>
                <input
                  type='text'
                  value={newSource.key}
                  onChange={(e) => setNewSource({ ...newSource, key: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  placeholder='例如: source1'
                />
              </div>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  名称
                </label>
                <input
                  type='text'
                  value={newSource.name}
                  onChange={(e) => setNewSource({ ...newSource, name: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  placeholder='例如: 视频源1'
                />
              </div>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  API 地址
                </label>
                <input
                  type='text'
                  value={newSource.api}
                  onChange={(e) => setNewSource({ ...newSource, api: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  placeholder='https://example.com/api.php/provide/vod'
                />
              </div>
              <div className='flex items-center gap-2'>
                <input
                  type='checkbox'
                  id='is_adult'
                  checked={newSource.is_adult}
                  onChange={(e) => setNewSource({ ...newSource, is_adult: e.target.checked })}
                  className='w-4 h-4 rounded border-gray-300'
                />
                <label htmlFor='is_adult' className='text-sm text-gray-700 dark:text-gray-300'>
                  成人内容 (18+)
                </label>
              </div>
            </div>
            <div className='flex justify-end gap-2 mt-6'>
              <button
                onClick={() => setIsAddModalOpen(false)}
                className='px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors'
              >
                取消
              </button>
              <button
                onClick={handleAdd}
                className='px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors'
              >
                添加
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 编辑源弹窗 */}
      {editingSource && (
        <div className='fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4'>
          <div className='bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6'>
            <div className='flex justify-between items-center mb-4'>
              <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
                编辑视频源
              </h3>
              <button onClick={() => setEditingSource(null)}>
                <X className='w-5 h-5 text-gray-500' />
              </button>
            </div>
            <div className='space-y-4'>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  名称
                </label>
                <input
                  type='text'
                  value={editingSource.name}
                  onChange={(e) => setEditingSource({ ...editingSource, name: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                />
              </div>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  API 地址
                </label>
                <input
                  type='text'
                  value={editingSource.api}
                  onChange={(e) => setEditingSource({ ...editingSource, api: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                />
              </div>
              <div className='flex items-center gap-2'>
                <input
                  type='checkbox'
                  id='edit_is_adult'
                  checked={editingSource.is_adult}
                  onChange={(e) => setEditingSource({ ...editingSource, is_adult: e.target.checked })}
                  className='w-4 h-4 rounded border-gray-300'
                />
                <label htmlFor='edit_is_adult' className='text-sm text-gray-700 dark:text-gray-300'>
                  成人内容 (18+)
                </label>
              </div>
            </div>
            <div className='flex justify-end gap-2 mt-6'>
              <button
                onClick={() => setEditingSource(null)}
                className='px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors'
              >
                取消
              </button>
              <button
                onClick={handleSaveEdit}
                className='px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors'
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// 自定义分类配置组件
interface CategoryConfigProps {
  config: AdminConfig | null;
  onSave: (config: AdminConfig) => void;
  showAlert: (type: 'success' | 'error' | 'warning', title: string, message?: string) => void;
}

const CategoryConfig = ({ config, onSave, showAlert }: CategoryConfigProps) => {
  const [categories, setCategories] = useState<any[]>([]);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newCategory, setNewCategory] = useState({ name: '', query: '', type: 'movie' as 'movie' | 'tv' });

  useEffect(() => {
    // 确保当 config 变化时始终同步 categories 状态
    const customCategories = config?.CustomCategories || [];
    setCategories([...customCategories]);
  }, [config?.CustomCategories]);

  const saveChanges = (newCategories: any[]) => {
    if (!config) return;
    const newConfig = { ...config, CustomCategories: newCategories };
    onSave(newConfig);
  };

  const handleToggle = (index: number) => {
    const newCategories = categories.map((c, i) =>
      i === index ? { ...c, disabled: !c.disabled } : c
    );
    setCategories(newCategories);
    saveChanges(newCategories);
  };

  const handleDelete = (index: number) => {
    const newCategories = categories.filter((_, i) => i !== index);
    setCategories(newCategories);
    saveChanges(newCategories);
    showAlert('success', '删除成功');
  };

  const handleAdd = () => {
    if (!newCategory.name || !newCategory.query) {
      showAlert('error', '请填写完整信息');
      return;
    }
    const newCategories = [...categories, { ...newCategory, from: 'custom', disabled: false }];
    setCategories(newCategories);
    saveChanges(newCategories);
    setNewCategory({ name: '', query: '', type: 'movie' });
    setIsAddModalOpen(false);
    showAlert('success', '添加成功');
  };

  return (
    <div className='space-y-4'>
      <div className='flex justify-between items-center'>
        <p className='text-sm text-gray-600 dark:text-gray-400'>
          共 {categories.length} 个分类
        </p>
        <button
          onClick={() => setIsAddModalOpen(true)}
          className='flex items-center gap-1.5 px-3 py-1.5 bg-green-600 hover:bg-green-700 text-white text-sm rounded-lg transition-colors'
        >
          <Plus className='w-4 h-4' />
          添加分类
        </button>
      </div>

      <div className='space-y-2'>
        {categories.map((category, index) => (
          <div
            key={`${category.query}-${category.type}-${index}`}
            className='flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg'
          >
            <div className='flex-1 min-w-0'>
              <div className='flex items-center gap-2'>
                <span className='text-sm font-medium text-gray-900 dark:text-gray-100'>
                  {category.name || category.query}
                </span>
                <span className={`px-1.5 py-0.5 text-xs rounded ${category.type === 'movie' ? 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400' : 'bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400'}`}>
                  {category.type === 'movie' ? '电影' : '电视剧'}
                </span>
              </div>
              <p className='text-xs text-gray-500 dark:text-gray-400'>
                搜索: {category.query}
              </p>
            </div>
            <button
              onClick={() => handleToggle(index)}
              className={`relative w-10 h-5 rounded-full transition-colors ${category.disabled ? 'bg-gray-300 dark:bg-gray-600' : 'bg-green-500'}`}
            >
              <span
                className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform ${category.disabled ? 'left-0.5' : 'left-5'}`}
              />
            </button>
            <button
              onClick={() => handleDelete(index)}
              className='p-1.5 text-gray-500 hover:text-red-600 dark:hover:text-red-400 transition-colors'
            >
              <Trash2 className='w-4 h-4' />
            </button>
          </div>
        ))}
      </div>

      {categories.length === 0 && (
        <div className='text-center py-8 text-gray-500 dark:text-gray-400'>
          <FolderOpen className='w-12 h-12 mx-auto mb-3 opacity-50' />
          <p>暂无自定义分类</p>
        </div>
      )}

      {/* 添加分类弹窗 */}
      {isAddModalOpen && (
        <div className='fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4'>
          <div className='bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6'>
            <div className='flex justify-between items-center mb-4'>
              <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
                添加自定义分类
              </h3>
              <button onClick={() => setIsAddModalOpen(false)}>
                <X className='w-5 h-5 text-gray-500' />
              </button>
            </div>
            <div className='space-y-4'>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  分类名称
                </label>
                <input
                  type='text'
                  value={newCategory.name}
                  onChange={(e) => setNewCategory({ ...newCategory, name: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  placeholder='例如: 漫威电影'
                />
              </div>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  搜索关键词
                </label>
                <input
                  type='text'
                  value={newCategory.query}
                  onChange={(e) => setNewCategory({ ...newCategory, query: e.target.value })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                  placeholder='例如: 漫威'
                />
              </div>
              <div>
                <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1'>
                  类型
                </label>
                <select
                  value={newCategory.type}
                  onChange={(e) => setNewCategory({ ...newCategory, type: e.target.value as 'movie' | 'tv' })}
                  className='w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100'
                >
                  <option value='movie'>电影</option>
                  <option value='tv'>电视剧</option>
                </select>
              </div>
            </div>
            <div className='flex justify-end gap-2 mt-6'>
              <button
                onClick={() => setIsAddModalOpen(false)}
                className='px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors'
              >
                取消
              </button>
              <button
                onClick={handleAdd}
                className='px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors'
              >
                添加
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};


// 配置订阅组件
interface ConfigSubscriptionProps {
  config: AdminConfig | null;
  onSave: (config: AdminConfig) => void;
  showAlert: (type: 'success' | 'error' | 'warning', title: string, message?: string) => void;
}

const ConfigSubscription = ({ config, onSave, showAlert }: ConfigSubscriptionProps) => {
  const [subscriptionUrl, setSubscriptionUrl] = useState('');
  const [autoUpdate, setAutoUpdate] = useState(false);
  const [lastCheckTime, setLastCheckTime] = useState<string | null>(null);
  const [configContent, setConfigContent] = useState('');
  const [isFetching, setIsFetching] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (config?.ConfigSubscribtion) {
      setSubscriptionUrl(config.ConfigSubscribtion.URL || '');
      setAutoUpdate(config.ConfigSubscribtion.AutoUpdate || false);
      setLastCheckTime(config.ConfigSubscribtion.LastCheck || null);
    }
    if (config?.ConfigFile) {
      setConfigContent(config.ConfigFile);
    }
  }, [config]);

  const handleFetchConfig = async () => {
    if (!subscriptionUrl.trim()) {
      showAlert('error', '请输入订阅URL');
      return;
    }

    setIsFetching(true);
    try {
      const response = await fetch(subscriptionUrl);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const text = await response.text();

      // 验证是否为有效的 JSON
      try {
        JSON.parse(text);
      } catch {
        throw new Error('返回内容不是有效的 JSON 格式');
      }

      setConfigContent(text);
      const now = new Date().toISOString();
      setLastCheckTime(now);

      // 保存订阅信息
      if (config) {
        const newConfig = {
          ...config,
          ConfigSubscribtion: {
            ...config.ConfigSubscribtion,
            URL: subscriptionUrl,
            LastCheck: now,
          },
        };
        onSave(newConfig);
      }

      showAlert('success', '拉取成功', '配置已更新到编辑器中，请确认后保存');
    } catch (error) {
      console.error('拉取配置失败:', error);
      showAlert('error', '拉取失败', error instanceof Error ? error.message : '网络错误');
    } finally {
      setIsFetching(false);
    }
  };

  const handleSave = () => {
    if (!configContent.trim()) {
      showAlert('error', '配置内容不能为空');
      return;
    }

    setIsSaving(true);
    try {
      // 解析配置内容
      const parsedContent = JSON.parse(configContent);

      if (config) {
        // 合并配置
        const newConfig: AdminConfig = {
          ...config,
          ConfigFile: configContent,
          ConfigSubscribtion: {
            ...config.ConfigSubscribtion,
            URL: subscriptionUrl,
            AutoUpdate: autoUpdate,
          },
        };

        // 智能识别配置格式
        // 情况1: 标准 AdminConfig 格式 (包含 SourceConfig 字段)
        if (parsedContent.SourceConfig && Array.isArray(parsedContent.SourceConfig)) {
          newConfig.SourceConfig = parsedContent.SourceConfig.map((s: any) => ({
            key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
            name: s.name || '未命名源',
            api: s.api,
            detail: s.detail || '',
            from: 'config' as const,
            disabled: s.disabled || false,
            is_adult: s.is_adult || false,
          }));
        }
        // 情况2: 包含 sites 字段的格式 (常见的订阅配置格式)
        else if (parsedContent.sites && Array.isArray(parsedContent.sites)) {
          newConfig.SourceConfig = parsedContent.sites.map((s: any) => ({
            key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
            name: s.name || '未命名源',
            api: s.api,
            detail: s.detail || '',
            from: 'config' as const,
            disabled: s.disabled || false,
            is_adult: s.is_adult || false,
          }));
        }
        // 情况3: 纯数组格式 - 作为视频源处理
        else if (Array.isArray(parsedContent) && parsedContent.length > 0 && parsedContent[0].api) {
          newConfig.SourceConfig = parsedContent.map((s: any) => ({
            key: s.key || s.name?.toLowerCase().replace(/\s+/g, '_') || `source_${Date.now()}`,
            name: s.name || '未命名源',
            api: s.api,
            detail: s.detail || '',
            from: 'config' as const,
            disabled: s.disabled || false,
            is_adult: s.is_adult || false,
          }));
        }
        // 情况4: api_site 对象格式 (键值对形式)
        else if (parsedContent.api_site && typeof parsedContent.api_site === 'object') {
          newConfig.SourceConfig = Object.entries(parsedContent.api_site).map(([key, value]: [string, any]) => ({
            key: key,
            name: value.name || key,
            api: value.api,
            detail: value.detail || '',
            from: 'config' as const,
            disabled: value.disabled || false,
            is_adult: value.is_adult || false,
          }));
        }

        // 如果解析的内容包含 CustomCategories，合并进去
        if (parsedContent.CustomCategories && Array.isArray(parsedContent.CustomCategories)) {
          newConfig.CustomCategories = parsedContent.CustomCategories.map((c: any) => ({
            ...c,
            from: 'config' as const,
          }));
        }

        onSave(newConfig);
        showAlert('success', '保存成功', `已解析 ${newConfig.SourceConfig?.length || 0} 个视频源`);
      }
    } catch (error) {
      console.error('保存配置失败:', error);
      showAlert('error', '保存失败', '配置格式错误，请检查 JSON 格式');
    } finally {
      setIsSaving(false);
    }
  };

  const handleAutoUpdateChange = (enabled: boolean) => {
    setAutoUpdate(enabled);
    if (config) {
      const newConfig = {
        ...config,
        ConfigSubscribtion: {
          ...config.ConfigSubscribtion,
          AutoUpdate: enabled,
        },
      };
      onSave(newConfig);
    }
  };

  return (
    <div className='space-y-6'>
      {/* 订阅URL输入 */}
      <div>
        <div className='flex items-center justify-between mb-3'>
          <label className='block text-sm font-medium text-gray-700 dark:text-gray-300'>
            订阅URL
          </label>
          <div className='text-xs text-gray-500 dark:text-gray-400'>
            最后更新: {lastCheckTime ? new Date(lastCheckTime).toLocaleString('zh-CN') : '从未更新'}
          </div>
        </div>
        <div className='flex gap-2'>
          <input
            type='url'
            value={subscriptionUrl}
            onChange={(e) => setSubscriptionUrl(e.target.value)}
            placeholder='https://example.com/config.json'
            className='flex-1 px-4 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all'
          />
          <button
            onClick={handleFetchConfig}
            disabled={isFetching || !subscriptionUrl.trim()}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-lg font-medium transition-all ${
              isFetching || !subscriptionUrl.trim()
                ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 cursor-not-allowed'
                : 'bg-blue-600 hover:bg-blue-700 text-white'
            }`}
          >
            {isFetching ? (
              <>
                <RefreshCw className='w-4 h-4 animate-spin' />
                拉取中
              </>
            ) : (
              <>
                <RefreshCw className='w-4 h-4' />
                拉取
              </>
            )}
          </button>
        </div>
        <p className='mt-2 text-xs text-gray-500 dark:text-gray-400'>
          输入配置文件的订阅地址，要求 JSON 格式
        </p>
      </div>

      {/* 自动更新开关 */}
      <div className='flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg'>
        <div>
          <label className='text-sm font-medium text-gray-700 dark:text-gray-300'>
            自动更新
          </label>
          <p className='text-xs text-gray-500 dark:text-gray-400 mt-1'>
            启用后系统将定期自动拉取最新配置
          </p>
        </div>
        <button
          type='button'
          onClick={() => handleAutoUpdateChange(!autoUpdate)}
          className={`relative w-11 h-6 rounded-full transition-colors ${
            autoUpdate ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'
          }`}
        >
          <span
            className={`absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${
              autoUpdate ? 'left-5.5 translate-x-0' : 'left-0.5'
            }`}
            style={{ left: autoUpdate ? '22px' : '2px' }}
          />
        </button>
      </div>

      {/* 配置文件编辑区域 */}
      <div>
        <label className='block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3'>
          配置内容
        </label>
        <textarea
          value={configContent}
          onChange={(e) => setConfigContent(e.target.value)}
          rows={16}
          placeholder='请输入配置文件内容（JSON 格式）...'
          className='w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all'
          style={{
            fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace',
          }}
          spellCheck={false}
        />
        <div className='flex items-center justify-between mt-3'>
          <p className='text-xs text-gray-500 dark:text-gray-400'>
            支持 JSON 格式，用于配置视频源和自定义分类
          </p>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-all ${
              isSaving
                ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 cursor-not-allowed'
                : 'bg-green-600 hover:bg-green-700 text-white'
            }`}
          >
            {isSaving ? (
              <>
                <RefreshCw className='w-4 h-4 animate-spin' />
                保存中
              </>
            ) : (
              <>
                <Check className='w-4 h-4' />
                保存配置
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

// 主页面组件
function AdminPageContent() {
  const [config, setConfig] = useState<AdminConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [expandedTabs, setExpandedTabs] = useState({
    version: true,
    configSubscription: false,
    videoSource: false,
    categoryConfig: false,
    liveSource: false,
    databaseImportExport: false,
  });
  const [alertModal, setAlertModal] = useState({
    isOpen: false,
    type: 'success' as 'success' | 'error' | 'warning',
    title: '',
    message: '',
  });
  const [isImportExportModalOpen, setIsImportExportModalOpen] = useState(false);

  // 加载配置
  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setIsLoading(true);
    try {
      let configToUse: AdminConfig | null = null;

      // 尝试从 localStorage 读取
      const stored = localStorage.getItem(LOCAL_CONFIG_KEY);
      if (stored) {
        configToUse = JSON.parse(stored);
      }

      // 从 Tauri 后端加载
      try {
        const tauriData = await invoke<AdminConfig>('get_config');
        // 如果 Tauri 后端有配置且有 SourceConfig
        if (tauriData && tauriData.SourceConfig && tauriData.SourceConfig.length > 0) {
          configToUse = tauriData;
          // 同步到 localStorage
          localStorage.setItem(LOCAL_CONFIG_KEY, JSON.stringify(tauriData));
        } else if (configToUse) {
          // 如果 localStorage 有配置但 Tauri 后端没有，同步到 Tauri 后端
          await invoke('save_config', { config: configToUse });
          console.log('配置已从 localStorage 同步到 Tauri 后端');
        }
      } catch (tauriError) {
        console.warn('从 Tauri 后端加载配置失败:', tauriError);
      }

      if (configToUse) {
        setConfig(configToUse);
      } else {
        const defaultConfig = getDefaultConfig();
        setConfig(defaultConfig);
        localStorage.setItem(LOCAL_CONFIG_KEY, JSON.stringify(defaultConfig));
      }
    } catch (error) {
      console.error('加载配置失败:', error);
      const defaultConfig = getDefaultConfig();
      setConfig(defaultConfig);
    } finally {
      setIsLoading(false);
    }
  };

  const saveConfig = useCallback(async (newConfig: AdminConfig) => {
    try {
      // 保存到 localStorage
      localStorage.setItem(LOCAL_CONFIG_KEY, JSON.stringify(newConfig));
      setConfig(newConfig);

      // 同步到 Tauri 后端
      try {
        await invoke('save_config', { config: newConfig });
        console.log('配置已同步到 Tauri 后端');
      } catch (tauriError) {
        console.warn('同步到 Tauri 后端失败:', tauriError);
      }
    } catch (error) {
      console.error('保存配置失败:', error);
      showAlert('error', '保存失败', '请检查浏览器存储空间');
    }
  }, []);

  const showAlert = (type: 'success' | 'error' | 'warning', title: string, message?: string) => {
    setAlertModal({ isOpen: true, type, title, message: message || '' });
  };

  const hideAlert = () => {
    setAlertModal({ ...alertModal, isOpen: false });
  };

  const toggleTab = (tab: keyof typeof expandedTabs) => {
    setExpandedTabs((prev) => ({ ...prev, [tab]: !prev[tab] }));
  };

  if (isLoading) {
    return (
      <PageLayout activePath='/admin'>
        <div className='flex items-center justify-center min-h-screen'>
          <div className='text-center'>
            <div className='w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4' />
            <p className='text-gray-600 dark:text-gray-400'>加载中...</p>
          </div>
        </div>
      </PageLayout>
    );
  }

  return (
    <PageLayout activePath='/admin'>
      <div className='max-w-4xl mx-auto px-4 py-6 space-y-4'>
        {/* 页面标题 */}
        <div className='flex items-center justify-between'>
          <h1 className='text-2xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2'>
            <Settings className='w-6 h-6' />
            设置
          </h1>
          <button
            onClick={() => setIsImportExportModalOpen(true)}
            className='flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors'
          >
            <Database className='w-4 h-4' />
            导入/导出
          </button>
        </div>

        {/* 配置订阅 */}
        <CollapsibleTab
          title='配置订阅'
          icon={<Cloud className='w-5 h-5 text-cyan-500' />}
          isExpanded={expandedTabs.configSubscription}
          onToggle={() => toggleTab('configSubscription')}
        >
          <ConfigSubscription config={config} onSave={saveConfig} showAlert={showAlert} />
        </CollapsibleTab>

        {/* 视频源配置 */}
        <CollapsibleTab
          title='视频源配置'
          icon={<Video className='w-5 h-5 text-blue-500' />}
          isExpanded={expandedTabs.videoSource}
          onToggle={() => toggleTab('videoSource')}
        >
          <SourceConfig config={config} onSave={saveConfig} showAlert={showAlert} />
        </CollapsibleTab>

        {/* 自定义分类 */}
        <CollapsibleTab
          title='自定义分类'
          icon={<FolderOpen className='w-5 h-5 text-purple-500' />}
          isExpanded={expandedTabs.categoryConfig}
          onToggle={() => toggleTab('categoryConfig')}
        >
          <CategoryConfig config={config} onSave={saveConfig} showAlert={showAlert} />
        </CollapsibleTab>
        {/* 数据库导入导出 */}
        <CollapsibleTab
          title='数据操作'
          icon={<Database className='w-5 h-5 text-green-500' />}
          isExpanded={expandedTabs.databaseImportExport}
          onToggle={() => toggleTab('databaseImportExport')}
        >
          <DatabaseImportExport showAlert={showAlert} />
        </CollapsibleTab>
      </div>

      {/* 弹窗 */}
      <AlertModal
        isOpen={alertModal.isOpen}
        onClose={hideAlert}
        type={alertModal.type}
        title={alertModal.title}
        message={alertModal.message}
        timer={2000}
      />

      {/* 导入/导出弹窗 */}
      <ConfigImportExportModal
        isOpen={isImportExportModalOpen}
        onClose={() => setIsImportExportModalOpen(false)}
        config={config}
        onImport={(newConfig) => {
          saveConfig(newConfig);
          showAlert('success', '导入成功');
        }}
      />
    </PageLayout>
  );
}

export default function AdminPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <AdminPageContent />
    </Suspense>
  );
}
