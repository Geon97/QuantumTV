'use client'

import { type ReactNode, type HTMLAttributes, forwardRef, useState, useRef, useEffect, useCallback } from 'react'
import { cn } from '@/lib/utils'
import {
  Play,
  Pause,
  Volume2,
  VolumeX,
  Maximize,
  Minimize,
  SkipBack,
  SkipForward,
  Settings,
  Subtitles,
  PictureInPicture2,
  Loader2,
} from 'lucide-react'

// Format time helper
function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)

  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }
  return `${m}:${s.toString().padStart(2, '0')}`
}

// Player Button
export interface PlayerButtonProps extends HTMLAttributes<HTMLButtonElement> {
  icon: ReactNode
  size?: 'sm' | 'md' | 'lg'
  active?: boolean
  disabled?: boolean
}

export const PlayerButton = forwardRef<HTMLButtonElement, PlayerButtonProps>(
  ({ icon, size = 'md', active = false, disabled = false, className, ...props }, ref) => {
    const sizeClasses = {
      sm: 'w-8 h-8',
      md: 'w-10 h-10',
      lg: 'w-14 h-14',
    }

    return (
      <button
        ref={ref}
        className={cn(
          'player-btn',
          sizeClasses[size],
          active && 'bg-white/20',
          disabled && 'opacity-50 cursor-not-allowed',
          className
        )}
        disabled={disabled}
        {...props}
      >
        {icon}
      </button>
    )
  }
)

PlayerButton.displayName = 'PlayerButton'

// Progress Bar
export interface PlayerProgressProps {
  currentTime: number
  duration: number
  buffered?: number
  onSeek?: (time: number) => void
  className?: string
}

export function PlayerProgress({
  currentTime,
  duration,
  buffered = 0,
  onSeek,
  className,
}: PlayerProgressProps) {
  const progressRef = useRef<HTMLDivElement>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [hoverTime, setHoverTime] = useState<number | null>(null)
  const [hoverPosition, setHoverPosition] = useState(0)

  const getTimeFromPosition = useCallback(
    (clientX: number) => {
      if (!progressRef.current || duration === 0) return 0
      const rect = progressRef.current.getBoundingClientRect()
      const position = (clientX - rect.left) / rect.width
      return Math.max(0, Math.min(duration, position * duration))
    },
    [duration]
  )

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!progressRef.current) return
      const rect = progressRef.current.getBoundingClientRect()
      const position = (e.clientX - rect.left) / rect.width
      setHoverPosition(position * 100)
      setHoverTime(getTimeFromPosition(e.clientX))
    },
    [getTimeFromPosition]
  )

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      const time = getTimeFromPosition(e.clientX)
      onSeek?.(time)
    },
    [getTimeFromPosition, onSeek]
  )

  const playedPercentage = duration > 0 ? (currentTime / duration) * 100 : 0
  const bufferedPercentage = duration > 0 ? (buffered / duration) * 100 : 0

  return (
    <div
      ref={progressRef}
      className={cn('player-progress', className)}
      onClick={handleClick}
      onMouseMove={handleMouseMove}
      onMouseLeave={() => setHoverTime(null)}
    >
      {/* Buffered */}
      <div
        className="player-progress-buffered"
        style={{ width: `${bufferedPercentage}%` }}
      />
      {/* Played */}
      <div
        className="player-progress-played"
        style={{ width: `${playedPercentage}%` }}
      />
      {/* Thumb */}
      <div
        className="player-progress-thumb"
        style={{ left: `${playedPercentage}%` }}
      />
      {/* Hover Time Tooltip */}
      {hoverTime !== null && (
        <div
          className="absolute -top-8 transform -translate-x-1/2 px-2 py-1 bg-black/90 rounded text-xs text-white whitespace-nowrap"
          style={{ left: `${hoverPosition}%` }}
        >
          {formatTime(hoverTime)}
        </div>
      )}
    </div>
  )
}

// Volume Control
export interface VolumeControlProps {
  volume: number
  muted: boolean
  onVolumeChange?: (volume: number) => void
  onMuteToggle?: () => void
  className?: string
}

export function VolumeControl({
  volume,
  muted,
  onVolumeChange,
  onMuteToggle,
  className,
}: VolumeControlProps) {
  const [showSlider, setShowSlider] = useState(false)

  return (
    <div
      className={cn('flex items-center gap-2', className)}
      onMouseEnter={() => setShowSlider(true)}
      onMouseLeave={() => setShowSlider(false)}
    >
      <PlayerButton
        icon={muted || volume === 0 ? <VolumeX size={20} /> : <Volume2 size={20} />}
        size="sm"
        onClick={onMuteToggle}
        aria-label={muted ? '取消静音' : '静音'}
      />
      {showSlider && (
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={muted ? 0 : volume}
          onChange={(e) => onVolumeChange?.(parseFloat(e.target.value))}
          className="volume-slider"
          aria-label="音量"
        />
      )}
    </div>
  )
}

// Player Time Display
export interface PlayerTimeProps {
  currentTime: number
  duration: number
  className?: string
}

export function PlayerTime({ currentTime, duration, className }: PlayerTimeProps) {
  return (
    <span className={cn('player-time', className)}>
      {formatTime(currentTime)} / {formatTime(duration)}
    </span>
  )
}

// Episode Selector
export interface Episode {
  index: number
  title?: string
  watched?: boolean
  current?: boolean
}

export interface EpisodeSelectorProps {
  episodes: Episode[]
  currentIndex: number
  onSelect: (index: number) => void
  className?: string
}

export function EpisodeSelector({
  episodes,
  currentIndex,
  onSelect,
  className,
}: EpisodeSelectorProps) {
  const currentRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    currentRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }, [currentIndex])

  return (
    <div className={cn('episode-grid', className)}>
      {episodes.map((episode) => (
        <button
          key={episode.index}
          ref={episode.index === currentIndex ? currentRef : undefined}
          className={cn(
            'episode-btn',
            episode.index === currentIndex && 'active',
            episode.watched && 'watched'
          )}
          onClick={() => onSelect(episode.index)}
        >
          {episode.index + 1}
        </button>
      ))}
    </div>
  )
}

// Quality Selector
export interface QualityOption {
  label: string
  value: string
  bitrate?: number
}

export interface QualitySelectorProps {
  options: QualityOption[]
  currentQuality: string
  onSelect: (quality: string) => void
  className?: string
}

export function QualitySelector({
  options,
  currentQuality,
  onSelect,
  className,
}: QualitySelectorProps) {
  return (
    <div className={cn('flex flex-col gap-1', className)}>
      {options.map((option) => (
        <button
          key={option.value}
          className={cn(
            'px-4 py-2 text-sm text-left rounded-lg transition-colors',
            'hover:bg-white/10',
            currentQuality === option.value && 'bg-white/20 text-primary-400'
          )}
          onClick={() => onSelect(option.value)}
        >
          {option.label}
          {option.bitrate && (
            <span className="ml-2 text-xs text-white/60">
              {Math.round(option.bitrate / 1000)}kbps
            </span>
          )}
        </button>
      ))}
    </div>
  )
}

// Player Loading Overlay
export interface PlayerLoadingProps {
  isLoading: boolean
  message?: string
  className?: string
}

export function PlayerLoading({ isLoading, message, className }: PlayerLoadingProps) {
  if (!isLoading) return null

  return (
    <div
      className={cn(
        'absolute inset-0 flex flex-col items-center justify-center',
        'bg-black/50 backdrop-blur-sm',
        className
      )}
    >
      <Loader2 className="w-12 h-12 text-white animate-spin" />
      {message && (
        <p className="mt-3 text-sm text-white/80">{message}</p>
      )}
    </div>
  )
}

// Player Error Overlay
export interface PlayerErrorProps {
  error: string
  onRetry?: () => void
  className?: string
}

export function PlayerError({ error, onRetry, className }: PlayerErrorProps) {
  return (
    <div
      className={cn(
        'absolute inset-0 flex flex-col items-center justify-center',
        'bg-black/80 text-white',
        className
      )}
    >
      <div className="text-center max-w-sm px-4">
        <p className="text-lg font-medium mb-2">播放出错</p>
        <p className="text-sm text-white/70 mb-4">{error}</p>
        {onRetry && (
          <button
            onClick={onRetry}
            className="btn btn-primary"
          >
            重试
          </button>
        )}
      </div>
    </div>
  )
}

// Full Player Controls (Composite)
export interface PlayerControlsProps {
  isPlaying: boolean
  currentTime: number
  duration: number
  buffered: number
  volume: number
  muted: boolean
  isFullscreen: boolean
  isPiP?: boolean
  hasSubtitles?: boolean
  showSettings?: boolean
  onPlayPause: () => void
  onSeek: (time: number) => void
  onVolumeChange: (volume: number) => void
  onMuteToggle: () => void
  onFullscreenToggle: () => void
  onPiPToggle?: () => void
  onSettingsToggle?: () => void
  onSubtitlesToggle?: () => void
  onSkipBack?: () => void
  onSkipForward?: () => void
  className?: string
}

export function PlayerControls({
  isPlaying,
  currentTime,
  duration,
  buffered,
  volume,
  muted,
  isFullscreen,
  isPiP,
  hasSubtitles,
  showSettings,
  onPlayPause,
  onSeek,
  onVolumeChange,
  onMuteToggle,
  onFullscreenToggle,
  onPiPToggle,
  onSettingsToggle,
  onSubtitlesToggle,
  onSkipBack,
  onSkipForward,
  className,
}: PlayerControlsProps) {
  return (
    <div className={cn('player-controls visible', className)}>
      {/* Progress Bar */}
      <div className="mb-3">
        <PlayerProgress
          currentTime={currentTime}
          duration={duration}
          buffered={buffered}
          onSeek={onSeek}
        />
      </div>

      {/* Controls Row */}
      <div className="flex items-center justify-between">
        {/* Left Controls */}
        <div className="flex items-center gap-1">
          {onSkipBack && (
            <PlayerButton
              icon={<SkipBack size={20} />}
              size="sm"
              onClick={onSkipBack}
              aria-label="后退10秒"
            />
          )}
          <PlayerButton
            icon={isPlaying ? <Pause size={24} /> : <Play size={24} />}
            size="md"
            onClick={onPlayPause}
            aria-label={isPlaying ? '暂停' : '播放'}
          />
          {onSkipForward && (
            <PlayerButton
              icon={<SkipForward size={20} />}
              size="sm"
              onClick={onSkipForward}
              aria-label="快进10秒"
            />
          )}
          <VolumeControl
            volume={volume}
            muted={muted}
            onVolumeChange={onVolumeChange}
            onMuteToggle={onMuteToggle}
          />
          <PlayerTime currentTime={currentTime} duration={duration} />
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-1">
          {hasSubtitles && onSubtitlesToggle && (
            <PlayerButton
              icon={<Subtitles size={20} />}
              size="sm"
              onClick={onSubtitlesToggle}
              aria-label="字幕"
            />
          )}
          {onPiPToggle && (
            <PlayerButton
              icon={<PictureInPicture2 size={20} />}
              size="sm"
              active={isPiP}
              onClick={onPiPToggle}
              aria-label="画中画"
              className="hide-mobile"
            />
          )}
          {onSettingsToggle && (
            <PlayerButton
              icon={<Settings size={20} />}
              size="sm"
              active={showSettings}
              onClick={onSettingsToggle}
              aria-label="设置"
            />
          )}
          <PlayerButton
            icon={isFullscreen ? <Minimize size={20} /> : <Maximize size={20} />}
            size="sm"
            onClick={onFullscreenToggle}
            aria-label={isFullscreen ? '退出全屏' : '全屏'}
          />
        </div>
      </div>
    </div>
  )
}

export default PlayerControls
