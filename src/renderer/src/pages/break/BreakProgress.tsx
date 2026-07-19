import { useEffect, useRef, useState } from 'react'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import { BreakPopupStyle, type Settings } from '@shared/settings'
import { formatDuration } from '@shared/time'
import { useI18n } from '../../i18n/use-i18n'

interface Props {
  settings: Settings
  isPrimary: boolean
  sharedEndTime: number | null
  closing: boolean
  onReady: () => Promise<void>
  onFinished: (elapsedMs: number) => Promise<void>
  onCancel: () => Promise<void>
}

export function BreakProgress({
  settings,
  isPrimary,
  sharedEndTime,
  closing,
  onReady,
  onFinished,
  onCancel
}: Props): React.JSX.Element {
  const { t } = useI18n()
  const [localEndTime, setLocalEndTime] = useState<number | null>(null)
  const [now, setNow] = useState(0)
  const [videoSrc, setVideoSrc] = useState<string | null>(null)
  const startedAt = useRef(0)
  const finished = useRef(false)
  const readyOnce = useRef(false)
  const videoRef = useRef<HTMLVideoElement | null>(null)

  const endTime = sharedEndTime ?? localEndTime
  const isVideo = settings.breakPopupStyle === BreakPopupStyle.Video

  useEffect(() => {
    if (!isVideo) return
    let cancelled = false
    void window.neko.getBreakVideoSrc().then((src) => {
      if (!cancelled) setVideoSrc(src)
    })
    return () => {
      cancelled = true
    }
  }, [isVideo, settings.breakVideoPath, settings.breakVideoSource])

  useEffect(() => {
    if (readyOnce.current) return
    readyOnce.current = true
    startedAt.current = performance.now() + performance.timeOrigin

    void (async () => {
      await onReady()
      const useCueSound = !isVideo || settings.breakVideoMuted
      if (isPrimary && useCueSound) {
        await window.neko.playStartSound(settings.soundType, settings.breakSoundVolume)
      }
      const lengthMs = settings.breakLengthSeconds * 1000
      setLocalEndTime(performance.now() + performance.timeOrigin + lengthMs)
      setNow(performance.now() + performance.timeOrigin)
    })()
  }, [isPrimary, isVideo, onReady, settings])

  useEffect(() => {
    const id = window.setInterval(() => {
      setNow(performance.now() + performance.timeOrigin)
    }, 50)
    return () => window.clearInterval(id)
  }, [])

  useEffect(() => {
    if (!endTime || finished.current || closing || !now) return
    if (now >= endTime) {
      finished.current = true
      void (async () => {
        const useCueSound = !isVideo || settings.breakVideoMuted
        if (isPrimary && useCueSound) {
          await window.neko.playEndSound(settings.soundType, settings.breakSoundVolume)
        }
        await onFinished(now - startedAt.current)
      })()
    }
  }, [closing, endTime, isPrimary, isVideo, now, onFinished, settings])

  useEffect(() => {
    const el = videoRef.current
    if (!el || !videoSrc) return
    const muted = settings.breakVideoMuted || !isPrimary
    el.muted = muted
    el.volume = settings.breakSoundVolume
    void el.play().catch(() => {
      // Autoplay with sound can be blocked — fall back to muted.
      el.muted = true
      void el.play().catch(() => undefined)
    })
  }, [videoSrc, settings.breakVideoMuted, settings.breakSoundVolume, isPrimary])

  const lengthMs = settings.breakLengthSeconds * 1000
  const remainingMs = endTime && now ? Math.max(0, endTime - now) : lengthMs
  const progress = endTime && now ? 1 - remainingMs / lengthMs : 0
  const canEnd = settings.endBreakEnabled
  const showCardBackdrop = !isVideo && settings.showBackdrop
  const breakTitle = resolveBreakTitle(settings.breakTitle, t)
  const breakMessage = resolveBreakMessage(settings.breakMessage, t)

  return (
    <div
      className={`break-stage ${closing ? 'is-closing' : ''} ${isVideo ? 'is-video' : 'is-card'}`}
    >
      {isVideo ? (
        <>
          <div className="break-video-wash" />
          {videoSrc && (
            <video
              ref={videoRef}
              className="break-video"
              src={videoSrc}
              autoPlay
              loop
              playsInline
              muted={settings.breakVideoMuted || !isPrimary}
              preload="auto"
            />
          )}
          <div className="break-video-shade" />
          <div className="break-video-chrome">
            {canEnd && (
              <button type="button" className="break-end-quiet" onClick={() => void onCancel()}>
                {progress < 0.5 ? t('break.cancel') : t('break.end')}
              </button>
            )}
            <div className="break-video-copy">
              <p className="break-kicker">{t('app.name')}</p>
              <h1>{breakTitle}</h1>
            </div>
            <div className="break-meter">
              <div
                className="break-meter-fill"
                style={{ width: `${Math.min(100, progress * 100)}%` }}
              />
            </div>
            <div className="break-meta">
              <span>{formatDuration(remainingMs / 1000)}</span>
              <span>{Math.round(Math.min(100, progress * 100))}%</span>
            </div>
          </div>
        </>
      ) : (
        <>
          {showCardBackdrop && (
            <div
              className="break-backdrop"
              style={{ background: `rgba(8, 12, 10, ${settings.backdropOpacity})` }}
            />
          )}
          <div className="break-card">
            {canEnd && (
              <button type="button" className="break-end-quiet" onClick={() => void onCancel()}>
                {progress < 0.5 ? t('break.cancel') : t('break.end')}
              </button>
            )}
            <p className="break-kicker">{t('app.name')}</p>
            <h1>{breakTitle}</h1>
            <p className="break-message">{breakMessage}</p>
            <div className="break-meter">
              <div
                className="break-meter-fill"
                style={{ width: `${Math.min(100, progress * 100)}%` }}
              />
            </div>
            <div className="break-meta">
              <span>{formatDuration(remainingMs / 1000)}</span>
              <span>{Math.round(Math.min(100, progress * 100))}%</span>
            </div>
          </div>
        </>
      )}
    </div>
  )
}
