import { useEffect, useRef, useState } from 'react'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import type { PostponeAction } from '@shared/ipc'
import type { Settings } from '@shared/settings'
import { formatDuration } from '@shared/time'
import { useI18n } from '../../i18n/use-i18n'
import { getNekoApi } from '../../lib/neko'

interface Props {
  settings: Settings
  isPrimary: boolean
  sharedEndTime: number | null
  closing: boolean
  onReady: () => Promise<number | null>
  onFinished: (elapsedMs: number) => Promise<void>
  onCancel: () => Promise<void>
  onPostpone: (action: PostponeAction) => Promise<void>
}

export function BreakProgress({
  settings,
  isPrimary,
  sharedEndTime,
  closing,
  onReady,
  onFinished,
  onCancel,
  onPostpone
}: Props): React.JSX.Element {
  const neko = getNekoApi()
  const { t } = useI18n()
  const [localEndTime, setLocalEndTime] = useState<number | null>(null)
  const [now, setNow] = useState(0)
  const [actionPending, setActionPending] = useState(false)
  const startedAt = useRef(0)
  const finished = useRef(false)
  const readyOnce = useRef(false)

  const endTime = sharedEndTime ?? localEndTime

  useEffect(() => {
    if (readyOnce.current) return
    readyOnce.current = true
    startedAt.current = performance.now() + performance.timeOrigin

    void (async () => {
      const readyEndTime = await onReady()
      if (isPrimary) {
        await neko.playStartSound(settings.soundType, settings.breakSoundVolume)
      }
      if (finished.current) return
      if (readyEndTime != null) setLocalEndTime(readyEndTime)
      setNow(performance.now() + performance.timeOrigin)
      await new Promise<void>((resolve) => {
        window.requestAnimationFrame(() => {
          window.requestAnimationFrame(() => resolve())
        })
      })
      try {
        await neko.showBreakWindow()
      } catch (error) {
        console.warn('[neko] failed to show Break window', error)
      }
    })()
  }, [isPrimary, neko, onReady, settings])

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
        if (isPrimary) {
          await neko.playEndSound(settings.soundType, settings.breakSoundVolume)
        }
        await onFinished(now - startedAt.current)
      })()
    }
  }, [closing, endTime, isPrimary, neko, now, onFinished, settings])

  const lengthMs = settings.breakLengthSeconds * 1000
  const remainingMs = endTime && now ? Math.max(0, endTime - now) : lengthMs
  const progress = endTime && now ? 1 - remainingMs / lengthMs : 0
  const canEnd = isPrimary && settings.endBreakEnabled
  const canSkip = isPrimary && settings.skipBreakEnabled
  const showCardBackdrop = settings.showBackdrop
  const breakTitle = resolveBreakTitle(settings.breakTitle, t)
  const breakMessage = resolveBreakMessage(settings.breakMessage, t)

  const handleSkip = (): void => {
    if (!isPrimary || actionPending) return
    setActionPending(true)
    void onPostpone('skipped').catch(() => setActionPending(false))
  }

  return (
    <div className={`break-stage ${closing ? 'is-closing' : ''} is-card`}>
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
        {canSkip && (
          <div className="break-actions">
            <button
              type="button"
              className="break-action"
              disabled={actionPending}
              onClick={handleSkip}
            >
              {t('break.skip')}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
