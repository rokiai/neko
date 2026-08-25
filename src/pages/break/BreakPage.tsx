import { useCallback, useEffect, useMemo, useState } from 'react'
import type { PostponeAction } from '@shared/ipc'
import type { Settings } from '@shared/settings'
import { getNekoApi } from '../../lib/neko'
import { BreakProgress } from './BreakProgress'
import './break.css'

function windowId(): number {
  const raw = new URLSearchParams(window.location.search).get('windowId')
  return raw ? Number(raw) : 0
}

export function BreakPage(): React.JSX.Element {
  const neko = getNekoApi()
  const [settings, setSettings] = useState<Settings | null>(null)
  const [phase, setPhase] = useState<'boot' | 'progress' | 'closing'>('boot')
  const [sharedEndTime, setSharedEndTime] = useState<number | null>(null)
  const isPrimary = useMemo(() => windowId() === 0, [])

  useEffect(() => {
    let cancelled = false
    let closeTimer = 0
    const offStart = neko.onBreakStart((breakEndTime) => {
      setSharedEndTime(breakEndTime)
      setPhase('progress')
    })

    const offEnd = neko.onBreakEnd(() => {
      if (cancelled) return
      setPhase('closing')
      closeTimer = window.setTimeout(() => window.close(), 420)
    })

    const timer = window.setTimeout(() => {
      void (async () => {
        const nextSettings = await neko.getSettings()
        let activeEndTime: number | null = null
        try {
          activeEndTime = await neko.getActiveBreakEndTime()
        } catch (error) {
          console.warn('[neko] failed to restore active Break state', error)
        }
        if (cancelled) return
        setSettings(nextSettings)
        if (activeEndTime != null) setSharedEndTime(activeEndTime)
        // Give the webview a beat to paint before the native show path runs.
        setPhase('progress')
      })()
    }, 250)

    return () => {
      cancelled = true
      offStart()
      offEnd()
      window.clearTimeout(timer)
      window.clearTimeout(closeTimer)
    }
  }, [neko])

  // Secondary displays often miss the first start event (it used to fire
  // before any WebView existed). One refetch after boot is enough; the
  // start listener covers a late primary handshake.
  useEffect(() => {
    if (isPrimary || phase !== 'progress' || sharedEndTime != null) return

    let active = true
    void neko
      .getActiveBreakEndTime()
      .then((activeEndTime) => {
        if (active && activeEndTime != null) setSharedEndTime(activeEndTime)
      })
      .catch((error: unknown) => {
        console.warn('[neko] failed to restore active Break state', error)
      })

    return () => {
      active = false
    }
  }, [isPrimary, neko, phase, sharedEndTime])

  const onReady = useCallback(async (): Promise<number | null> => {
    try {
      await neko.resizeBreakWindow()
    } catch (error) {
      console.warn('[neko] failed to resize Break window', error)
    }
    if (isPrimary) return neko.startBreak()
    return sharedEndTime ?? neko.getActiveBreakEndTime()
  }, [isPrimary, neko, sharedEndTime])

  const onFinished = useCallback(
    async (elapsedMs: number): Promise<void> => {
      if (isPrimary) {
        await neko.completeBreakTracking(elapsedMs)
        await neko.endBreak()
      }
    },
    [isPrimary, neko]
  )

  const onCancel = useCallback(async (): Promise<void> => {
    if (isPrimary) await neko.endBreak()
  }, [isPrimary, neko])

  const onPostpone = useCallback(
    async (action: PostponeAction): Promise<void> => {
      if (isPrimary) await neko.postponeBreak(action)
    },
    [isPrimary, neko]
  )

  if (!settings || phase === 'boot') return <div className="break-root" />

  const style = {
    ['--break-bg' as string]: settings.backgroundColor,
    ['--break-fg' as string]: settings.textColor,
    ['--break-backdrop-alpha' as string]: String(settings.backdropOpacity)
  }

  return (
    <div className={`break-root phase-${phase}`} style={style}>
      {(phase === 'progress' || phase === 'closing') && (isPrimary || sharedEndTime != null) && (
        <BreakProgress
          settings={settings}
          isPrimary={isPrimary}
          sharedEndTime={sharedEndTime}
          closing={phase === 'closing'}
          onReady={onReady}
          onFinished={onFinished}
          onCancel={onCancel}
          onPostpone={onPostpone}
        />
      )}
    </div>
  )
}
