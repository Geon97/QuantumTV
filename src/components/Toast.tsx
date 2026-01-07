'use client';

import { CheckCircle, Info, XCircle } from 'lucide-react';
import { useEffect, useState } from 'react';

export interface ToastProps {
  message: string;
  type?: 'success' | 'error' | 'info';
  duration?: number;
  onClose?: () => void;
}

export default function Toast({
  message,
  type = 'info',
  duration = 3000,
  onClose,
}: ToastProps) {
  const [isVisible, setIsVisible] = useState(true);
  const [isLeaving, setIsLeaving] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsLeaving(true);
      setTimeout(() => {
        setIsVisible(false);
        onClose?.();
      }, 300);
    }, duration);

    return () => clearTimeout(timer);
  }, [duration, onClose]);

  if (!isVisible) return null;

  const icons = {
    success: <CheckCircle className='w-5 h-5' />,
    error: <XCircle className='w-5 h-5' />,
    info: <Info className='w-5 h-5' />,
  };

  const colors = {
    success: 'bg-green-500',
    error: 'bg-red-500',
    info: 'bg-blue-500',
  };

  return (
    <div
      className={`fixed bottom-20 right-5 z-10000 transform transition-all duration-300 ${
        isLeaving ? 'translate-x-[200%] opacity-0' : 'translate-x-0 opacity-100'
      }`}
    >
      <div
        className={`${colors[type]} text-white px-5 py-4 rounded-xl shadow-2xl flex items-center space-x-3 min-w-70 max-w-100`}
      >
        <div className='shrink-0'>{icons[type]}</div>
        <div className='flex-1 text-sm font-medium whitespace-pre-line'>
          {message}
        </div>
      </div>
    </div>
  );
}
