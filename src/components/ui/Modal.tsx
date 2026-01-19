'use client'

import {
  type ReactNode,
  type HTMLAttributes,
  useEffect,
  useCallback,
  createContext,
  useContext,
} from 'react'
import { createPortal } from 'react-dom'
import { cn } from '@/lib/utils'
import { X } from 'lucide-react'
import { IconButton } from './Button'

// Modal Context
const ModalContext = createContext<{ onClose: () => void } | null>(null)

// Modal Root
export interface ModalProps {
  isOpen: boolean
  onClose: () => void
  children: ReactNode
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  closeOnOverlayClick?: boolean
  closeOnEscape?: boolean
  className?: string
}

export function Modal({
  isOpen,
  onClose,
  children,
  size = 'md',
  closeOnOverlayClick = true,
  closeOnEscape = true,
  className,
}: ModalProps) {
  // Handle escape key
  const handleEscape = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape' && closeOnEscape) {
        onClose()
      }
    },
    [closeOnEscape, onClose]
  )

  useEffect(() => {
    if (isOpen) {
      document.addEventListener('keydown', handleEscape)
      document.body.style.overflow = 'hidden'
      return () => {
        document.removeEventListener('keydown', handleEscape)
        document.body.style.overflow = ''
      }
    }
  }, [isOpen, handleEscape])

  if (!isOpen) return null

  const sizeClasses = {
    sm: 'modal-sm',
    md: '',
    lg: 'modal-lg',
    xl: 'modal-xl',
    full: 'modal-full',
  }

  const modalContent = (
    <ModalContext.Provider value={{ onClose }}>
      {/* Backdrop */}
      <div
        className="modal-backdrop"
        onClick={closeOnOverlayClick ? onClose : undefined}
        aria-hidden="true"
      />
      {/* Modal */}
      <div
        role="dialog"
        aria-modal="true"
        className={cn('modal', sizeClasses[size], className)}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </ModalContext.Provider>
  )

  // Use portal for modal
  if (typeof window !== 'undefined') {
    return createPortal(modalContent, document.body)
  }

  return null
}

// Modal Header
export interface ModalHeaderProps extends HTMLAttributes<HTMLDivElement> {
  title?: string
  showClose?: boolean
}

export function ModalHeader({
  title,
  showClose = true,
  className,
  children,
  ...props
}: ModalHeaderProps) {
  const context = useContext(ModalContext)

  return (
    <div className={cn('modal-header', className)} {...props}>
      {children || (
        <>
          <h2 className="text-lg font-semibold text-foreground">{title}</h2>
          {showClose && context && (
            <IconButton
              icon={<X size={20} />}
              variant="ghost"
              size="sm"
              onClick={context.onClose}
              aria-label="关闭"
            />
          )}
        </>
      )}
    </div>
  )
}

// Modal Body
export function ModalBody({
  className,
  children,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('modal-body', className)} {...props}>
      {children}
    </div>
  )
}

// Modal Footer
export function ModalFooter({
  className,
  children,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('modal-footer', className)} {...props}>
      {children}
    </div>
  )
}

// Bottom Sheet (Mobile-friendly modal)
export interface BottomSheetProps {
  isOpen: boolean
  onClose: () => void
  children: ReactNode
  showHandle?: boolean
  className?: string
}

export function BottomSheet({
  isOpen,
  onClose,
  children,
  showHandle = true,
  className,
}: BottomSheetProps) {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden'
      return () => {
        document.body.style.overflow = ''
      }
    }
  }, [isOpen])

  if (!isOpen) return null

  const sheetContent = (
    <ModalContext.Provider value={{ onClose }}>
      {/* Backdrop */}
      <div className="modal-backdrop" onClick={onClose} aria-hidden="true" />
      {/* Sheet */}
      <div
        role="dialog"
        aria-modal="true"
        className={cn('bottom-sheet', className)}
        onClick={(e) => e.stopPropagation()}
      >
        {showHandle && <div className="bottom-sheet-handle" />}
        {children}
      </div>
    </ModalContext.Provider>
  )

  if (typeof window !== 'undefined') {
    return createPortal(sheetContent, document.body)
  }

  return null
}

// Dropdown Menu
export interface DropdownProps {
  trigger: ReactNode
  children: ReactNode
  align?: 'left' | 'right'
  className?: string
}

export function Dropdown({ trigger, children, align = 'left', className }: DropdownProps) {
  return (
    <div className="relative inline-block">
      {trigger}
      <div
        className={cn(
          'dropdown',
          align === 'right' ? 'right-0' : 'left-0',
          'top-full mt-1',
          className
        )}
      >
        {children}
      </div>
    </div>
  )
}

// Dropdown Item
export interface DropdownItemProps extends HTMLAttributes<HTMLDivElement> {
  icon?: ReactNode
  danger?: boolean
  disabled?: boolean
}

export function DropdownItem({
  icon,
  danger = false,
  disabled = false,
  className,
  children,
  onClick,
  ...props
}: DropdownItemProps) {
  return (
    <div
      className={cn(
        'dropdown-item',
        danger && 'dropdown-item-danger',
        disabled && 'opacity-50 cursor-not-allowed',
        className
      )}
      onClick={disabled ? undefined : onClick}
      role="menuitem"
      tabIndex={disabled ? -1 : 0}
      {...props}
    >
      {icon && <span className="shrink-0">{icon}</span>}
      {children}
    </div>
  )
}

// Dropdown Divider
export function DropdownDivider() {
  return <div className="dropdown-divider" role="separator" />
}

export default Modal
