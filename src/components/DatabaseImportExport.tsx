import { invoke } from '@tauri-apps/api/core';
import {
  AlertTriangle,
  Download,
  Trash2,
  Upload,
} from 'lucide-react';
import React, { useRef, useState } from 'react';
import { createPortal } from 'react-dom';
interface Props {
  showAlert: (type: 'success' | 'error' | 'warning', title: string, message?: string) => void;
}

// 确认弹窗组件
interface ConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'danger' | 'warning';
}

const ConfirmModal = ({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = '确认',
  cancelText = '取消',
  type = 'danger',
}: ConfirmModalProps) => {
  const [isVisible, setIsVisible] = useState(false);

  React.useEffect(() => {
    if (isOpen) {
      setIsVisible(true);
    } else {
      setIsVisible(false);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const getColors = () => {
    switch (type) {
      case 'danger':
        return {
          bg: 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800',
          button: 'bg-red-600 hover:bg-red-700 text-white',
          icon: 'text-red-500',
        };
      case 'warning':
        return {
          bg: 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800',
          button: 'bg-yellow-600 hover:bg-yellow-700 text-white',
          icon: 'text-yellow-500',
        };
    }
  };

  const colors = getColors();

  return createPortal(
    <div
      className={`fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4 transition-opacity duration-200 ${isVisible ? 'opacity-100' : 'opacity-0'}`}
      onClick={onClose}
    >
      <div
        className={`bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-sm w-full border ${colors.bg} transition-all duration-200 ${isVisible ? 'scale-100' : 'scale-95'}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className='p-6'>
          <div className='flex items-center gap-3 mb-4'>
            <AlertTriangle className={`w-6 h-6 ${colors.icon}`} />
            <h3 className='text-lg font-semibold text-gray-900 dark:text-gray-100'>
              {title}
            </h3>
          </div>
          <p className='text-sm text-gray-600 dark:text-gray-400 mb-6'>
            {message}
          </p>
          <div className='flex justify-end gap-3'>
            <button
              onClick={onClose}
              className='px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors'
            >
              {cancelText}
            </button>
            <button
              onClick={() => {
                onConfirm();
                onClose();
              }}
              className={`px-4 py-2 rounded-lg transition-colors ${colors.button}`}
            >
              {confirmText}
            </button>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
};

export default function DatabaseImportExport({ showAlert }: Props) {
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [confirmModal, setConfirmModal] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
    type?: 'danger' | 'warning';
  }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
    type: 'danger',
  });

  const fileInputRef = useRef<HTMLInputElement>(null);
// 文件导入逻辑
  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files?.length) return;
    const file = event.target.files[0];
    const reader = new FileReader();

    reader.onload = async (e) => {
      const content = e.target?.result;
      if (typeof content !== 'string') {
        showAlert('error', '文件读取失败: 内容格式不正确');
        return;
      }

      // 解析数字序列字符串成字节数组
      const byteStrings = content.split(',').map(s => s.trim());
      const bytes = new Uint8Array(byteStrings.map(n => Number(n)));

      // 把字节数组转成utf-8字符串
      const decoder = new TextDecoder('utf-8');
      const jsonString = decoder.decode(bytes);

      setImporting(true);
      try {
        await invoke('import_json', { data: jsonString });
        showAlert('success', '导入成功！');
      } catch (error) {
        console.error(error);
        showAlert('error', '导入失败，请检查文件格式或联系管理员');
      } finally {
        setImporting(false);
        if (fileInputRef.current) fileInputRef.current.value = '';
      }
    };

    reader.onerror = () => {
      showAlert('error', '文件读取失败');
      setImporting(false);
    };

    reader.readAsText(file);
  };
// 文件导出逻辑
  const handleExport = async () => {
    setExporting(true);
    try {
      const data: Uint8Array = await invoke('export_json');
      const blob = new Blob([data as unknown as ArrayBuffer], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const date = new Date().toISOString().slice(0, 10);
      a.download = `quantumtv_${date}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
      showAlert('success', '导出成功', `数据库已导出为 quantumtv_${date}.json`);
    } catch (error) {
      console.error('导出失败:', error);
      showAlert('error', '导出失败', '请重试或联系管理员');
    } finally {
      setExporting(false);
    }
  };

  const handleClearCache = async () => {
    setConfirmModal({
      isOpen: true,
      title: '清空数据库',
      message: '确认要清空数据库吗？此操作将删除所有数据且不可撤销！',
      onConfirm: async () => {
        setClearing(true);
        try {
          await invoke('clear_cache');
          showAlert('success', '数据库已清空', '所有数据已被清除');
        } catch (error) {
          console.error('清空失败:', error);
          showAlert('error', '清空失败', '请重试或联系管理员');
        } finally {
          setClearing(false);
        }
      },
      type: 'danger',
    });
  };

  const handleImportClick = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  const closeConfirmModal = () => {
    setConfirmModal({ ...confirmModal, isOpen: false });
  };

  return (
    <div className='space-y-6'>
      <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
        {/* 导入按钮 */}
        <div className='bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700 p-4'>
          <div className='flex items-center gap-3 mb-3'>
            <div className='p-2 bg-green-100 dark:bg-green-900/30 rounded-lg'>
              <Upload className='w-5 h-5 text-green-600 dark:text-green-400' />
            </div>
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100'>导入数据库</h4>
              <p className='text-xs text-gray-500 dark:text-gray-400'>从 JSON 文件恢复数据</p>
            </div>
          </div>
          <div className='space-y-3'>
            <input
              type='file'
              accept='.json,application/json'
              onChange={handleFileChange}
              ref={fileInputRef}
              disabled={importing || exporting || clearing}
              className='hidden'
            />
            <button
              onClick={handleImportClick}
              disabled={importing || exporting || clearing}
              className={`w-full px-4 py-2.5 rounded-lg transition-all flex items-center justify-center gap-2 ${
                importing || exporting || clearing
                  ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 cursor-not-allowed'
                  : 'bg-green-600 hover:bg-green-700 text-white'
              }`}
            >
              {importing ? (
                <>
                  <div className='w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin' />
                  导入中...
                </>
              ) : (
                <>
                  <Upload className='w-4 h-4' />
                  选择文件导入
                </>
              )}
            </button>
            <p className='text-xs text-gray-500 dark:text-gray-400 text-center'>
              支持 QuantumTV 导出的 JSON 文件
            </p>
          </div>
        </div>

        {/* 导出按钮 */}
        <div className='bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700 p-4'>
          <div className='flex items-center gap-3 mb-3'>
            <div className='p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg'>
              <Download className='w-5 h-5 text-blue-600 dark:text-blue-400' />
            </div>
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100'>导出数据库</h4>
              <p className='text-xs text-gray-500 dark:text-gray-400'>备份当前所有数据</p>
            </div>
          </div>
          <div className='space-y-3'>
            <button
              onClick={handleExport}
              disabled={importing || exporting || clearing}
              className={`w-full px-4 py-2.5 rounded-lg transition-all flex items-center justify-center gap-2 ${
                importing || exporting || clearing
                  ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 cursor-not-allowed'
                  : 'bg-blue-600 hover:bg-blue-700 text-white'
              }`}
            >
              {exporting ? (
                <>
                  <div className='w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin' />
                  导出中...
                </>
              ) : (
                <>
                  <Download className='w-4 h-4' />
                  导出数据库
                </>
              )}
            </button>
            <p className='text-xs text-gray-500 dark:text-gray-400 text-center'>
              导出为 JSON 格式，包含日期时间戳
            </p>
          </div>
        </div>

        {/* 清空按钮 */}
        <div className='bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700 p-4'>
          <div className='flex items-center gap-3 mb-3'>
            <div className='p-2 bg-red-100 dark:bg-red-900/30 rounded-lg'>
              <Trash2 className='w-5 h-5 text-red-600 dark:text-red-400' />
            </div>
            <div>
              <h4 className='font-medium text-gray-900 dark:text-gray-100'>清空数据库</h4>
              <p className='text-xs text-gray-500 dark:text-gray-400'>删除所有数据</p>
            </div>
          </div>
          <div className='space-y-3'>
            <button
              onClick={handleClearCache}
              disabled={importing || exporting || clearing}
              className={`w-full px-4 py-2.5 rounded-lg transition-all flex items-center justify-center gap-2 ${
                importing || exporting || clearing
                  ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 cursor-not-allowed'
                  : 'bg-red-600 hover:bg-red-700 text-white'
              }`}
            >
              {clearing ? (
                <>
                  <div className='w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin' />
                  清空中...
                </>
              ) : (
                <>
                  <Trash2 className='w-4 h-4' />
                  清空数据库
                </>
              )}
            </button>
            <p className='text-xs text-red-500 dark:text-red-400 text-center'>
              警告：此操作不可撤销！
            </p>
          </div>
        </div>
      </div>

      {/* 状态指示器 */}
      {(importing || exporting || clearing) && (
        <div className='mt-6'>
          <div className='flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400'>
            <div className='w-2 h-2 bg-blue-500 rounded-full animate-pulse' />
            {importing && '正在导入数据库，请稍候...'}
            {exporting && '正在导出数据库，请稍候...'}
            {clearing && '正在清空数据库，请稍候...'}
          </div>
        </div>
      )}

      {/* 确认弹窗 */}
      <ConfirmModal
        isOpen={confirmModal.isOpen}
        onClose={closeConfirmModal}
        onConfirm={confirmModal.onConfirm}
        title={confirmModal.title}
        message={confirmModal.message}
        confirmText='确认清空'
        cancelText='取消'
        type={confirmModal.type}
      />
    </div>
  );
}