'use client'

import { type ReactNode, createContext, useContext, useState, useCallback } from 'react'
import { createPortal } from 'react-dom'
import { cn } from '@/lib/utils'
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-react'

// Toast types
type ToastType = 'success' | 'error' | 'warning' | 'info'

interface Toast {
  id: string
  type: ToastType
  title: string
  description?: string
  duration?: number
}

// Toast Context
interface ToastContextValue {
  toasts: Toast[]
  addToast: (toast: Omit<Toast, 'id'>) => void
  removeToast: (id: string) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

// Toast Provider
export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])

  const addToast = useCallback((toast: Omit<Toast, 'id'>) => {
    const id = Math.random().toString(36).substr(2, 9)
    const newToast = { ...toast, id }
    setToasts((prev) => [...prev, newToast])

    // Auto remove
    const duration = toast.duration ?? 5000
    if (duration > 0) {
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id))
      }, duration)
    }
  }, [])

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }, [])

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
      <ToastContainer />
    </ToastContext.Provider>
  )
}

// Hook to use toast
export function useToast() {
  const context = useContext(ToastContext)
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider')
  }

  const toast = useCallback(
    (options: Omit<Toast, 'id'>) => {
      context.addToast(options)
    },
    [context]
  )

  return {
    toast,
    success: (title: string, description?: string) =>
      toast({ type: 'success', title, description }),
    error: (title: string, description?: string) =>
      toast({ type: 'error', title, description }),
    warning: (title: string, description?: string) =>
      toast({ type: 'warning', title, description }),
    info: (title: string, description?: string) =>
      toast({ type: 'info', title, description }),
    dismiss: context.removeToast,
  }
}

// Toast Container
function ToastContainer() {
  const context = useContext(ToastContext)
  if (!context || typeof window === 'undefined') return null

  return createPortal(
    <div className="fixed top-4 right-4 z-[80] flex flex-col gap-3 max-w-sm w-full pointer-events-none">
      {context.toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onClose={() => context.removeToast(toast.id)} />
      ))}
    </div>,
    document.body
  )
}

// Toast Item
interface ToastItemProps {
  toast: Toast
  onClose: () => void
}

const toastIcons: Record<ToastType, ReactNode> = {
  success: <CheckCircle size={20} className="text-success" />,
  error: <AlertCircle size={20} className="text-error" />,
  warning: <AlertTriangle size={20} className="text-warning" />,
  info: <Info size={20} className="text-info" />,
}

const toastClasses: Record<ToastType, string> = {
  success: 'toast-success',
  error: 'toast-error',
  warning: 'toast-warning',
  info: 'toast-info',
}

function ToastItem({ toast, onClose }: ToastItemProps) {
  return (
    <div className={cn('toast pointer-events-auto', toastClasses[toast.type])}>
      <div className="shrink-0">{toastIcons[toast.type]}</div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground">{toast.title}</p>
        {toast.description && (
          <p className="text-sm text-foreground-muted mt-0.5">{toast.description}</p>
        )}
      </div>
      <button
        onClick={onClose}
        className="shrink-0 p-1 rounded-md hover:bg-background-muted transition-colors"
        aria-label="关闭"
      >
        <X size={16} className="text-foreground-muted" />
      </button>
    </div>
  )
}

// Standalone Toast Component (for direct use)
export interface ToastProps {
  type?: ToastType
  title: string
  description?: string
  onClose?: () => void
  className?: string
}

export function Toast({ type = 'info', title, description, onClose, className }: ToastProps) {
  return (
    <div className={cn('toast', toastClasses[type], className)}>
      <div className="shrink-0">{toastIcons[type]}</div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground">{title}</p>
        {description && (
          <p className="text-sm text-foreground-muted mt-0.5">{description}</p>
        )}
      </div>
      {onClose && (
        <button
          onClick={onClose}
          className="shrink-0 p-1 rounded-md hover:bg-background-muted transition-colors"
          aria-label="关闭"
        >
          <X size={16} className="text-foreground-muted" />
        </button>
      )}
    </div>
  )
}

export default Toast
