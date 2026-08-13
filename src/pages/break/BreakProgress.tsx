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
  const meterRef = useRef<HTMLDivElement>(null)
  const startedAt = useRef(0)
  const finished = useRef(false)
  const readyOnce = useRef(false)

  const endTime = sharedEndTime ?? localEndTime
  const lengthMs = settings.breakLengthSeconds * 1000

  useEffect(() => {
    if (readyOnce.current) return
    readyOnce.current = true
    startedAt.current = performance.now() + performance.timeOrigin

    void (async () => {
      const readyEndTime = await onReady()
      if (isPrimary) {
        // Sound is best-effort: a missing audio device must not stall the
        // ready handshake that makes the Break window visible.
        try {
          await neko.playStartSound(settings.soundType, settings.breakSoundVolume)
        } catch (error) {
          console.warn('[neko] failed to play Break start sound', error)
        }
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

  // The UI only shows whole seconds, so it wakes once a second — but the last
  // wake is scheduled exactly on `endTime`, because the effect below ends the
  // break when `now` reaches it. A plain 1s interval would overrun the break
  // by up to a second, which is 20% of the shortest configurable break (5s).
  // Reading the wall clock on every wake also keeps the countdown honest
  // across system sleep, which a single end-of-break timeout would not.
  useEffect(() => {
    let cancelled = false
    let timer = 0

    const wake = (): void => {
      if (cancelled) return
      const current = performance.now() + performance.timeOrigin
      setNow(current)
      const untilEnd = endTime ? endTime - current : Number.POSITIVE_INFINITY
      timer = window.setTimeout(wake, Math.max(0, Math.min(1000, untilEnd)))
    }

    wake()
    return () => {
      cancelled = true
      window.clearTimeout(timer)
    }
  }, [endTime])

  // Written straight to the DOM once per end time: the bar is owned by the
  // compositor, not by React, so the countdown re-render neither restarts nor
  // advances it. The negative delay skips the part of the break that already
  // elapsed, which matters for a window opened mid-break.
  useEffect(() => {
    const meter = meterRef.current
    if (!meter) return
    if (!endTime) {
      meter.style.animation = ''
      return
    }
    const elapsedMs = Math.min(
      lengthMs,
      Math.max(0, lengthMs - (endTime - (performance.now() + performance.timeOrigin)))
    )
    meter.style.animation = `break-meter-fill ${lengthMs}ms linear ${-elapsedMs}ms forwards`
  }, [endTime, lengthMs])

  useEffect(() => {
    if (!endTime || finished.current || closing || !now) return
    if (now >= endTime) {
      finished.current = true
      void (async () => {
        if (isPrimary) {
          // Sound is best-effort: a failure must not block onFinished, which
          // reports the Break end and lets every Break window close.
          try {
            await neko.playEndSound(settings.soundType, settings.breakSoundVolume)
          } catch (error) {
            console.warn('[neko] failed to play Break end sound', error)
          }
        }
        await onFinished(now - startedAt.current)
      })()
    }
  }, [closing, endTime, isPrimary, neko, now, onFinished, settings])

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
    <div className="break-stage">
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
          <div ref={meterRef} className="break-meter-fill" />
        </div>
        <div className="break-meta">
          {/* Round up: the clock is sampled once a second and each wake lands a
              few ms late, so flooring would drop a whole label (2m 00s jumping
              straight to 1m 58s). Ceiling also reads "1s" until the break is
              actually over. */}
          <span>{formatDuration(Math.ceil(remainingMs / 1000))}</span>
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
