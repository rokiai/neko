import { useEffect, useMemo, useState } from 'react'
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
    const offStart = neko.onBreakStart((breakEndTime) => {
      setSharedEndTime(breakEndTime)
      setPhase('progress')
    })

    const offEnd = neko.onBreakEnd(() => {
      setPhase('closing')
      window.setTimeout(() => window.close(), 420)
    })

    let active = true
    const timer = window.setTimeout(() => {
      void (async () => {
        const nextSettings = await neko.getSettings()
        let activeEndTime: number | null = null
        try {
          activeEndTime = await neko.getActiveBreakEndTime()
        } catch (error) {
          console.warn('[neko] failed to restore active Break state', error)
        }
        if (!active) return
        setSettings(nextSettings)
        if (activeEndTime != null) setSharedEndTime(activeEndTime)
        // Give the webview a beat to paint before the native show path runs.
        setPhase('progress')
      })()
    }, 250)

    return () => {
      active = false
      window.clearTimeout(timer)
      offStart()
      offEnd()
    }
  }, [neko])

  useEffect(() => {
    if (isPrimary || phase !== 'progress' || sharedEndTime != null) return

    let active = true
    const syncActiveEndTime = async (): Promise<void> => {
      try {
        const activeEndTime = await neko.getActiveBreakEndTime()
        if (active && activeEndTime != null) setSharedEndTime(activeEndTime)
      } catch (error) {
        console.warn('[neko] failed to restore active Break state', error)
      }
    }

    void syncActiveEndTime()
    const poll = window.setInterval(() => void syncActiveEndTime(), 150)

    return () => {
      active = false
      window.clearInterval(poll)
    }
  }, [isPrimary, neko, phase, sharedEndTime])

  if (!settings || phase === 'boot') return <div className="break-root" />

  const fullscreen = settings.showBackdrop

  const style = {
    ['--break-bg' as string]: settings.backgroundColor,
    ['--break-fg' as string]: settings.textColor,
    ['--break-backdrop-alpha' as string]: String(settings.backdropOpacity)
  }

  return (
    <div
      className={`break-root phase-${phase}`}
      style={style}
      data-backdrop={fullscreen && phase === 'progress' ? 'on' : 'off'}
      data-style="card"
    >
      {(phase === 'progress' || phase === 'closing') && (isPrimary || sharedEndTime != null) && (
        <BreakProgress
          settings={settings}
          isPrimary={isPrimary}
          sharedEndTime={sharedEndTime}
          closing={phase === 'closing'}
          onReady={async () => {
            try {
              await neko.resizeBreakWindow()
            } catch (error) {
              console.warn('[neko] failed to resize Break window', error)
            }
            if (isPrimary) return neko.startBreak()
            return sharedEndTime ?? neko.getActiveBreakEndTime()
          }}
          onFinished={async (elapsedMs) => {
            if (isPrimary) {
              await neko.completeBreakTracking(elapsedMs)
              await neko.endBreak()
            }
          }}
          onCancel={async () => {
            if (isPrimary) await neko.endBreak()
          }}
          onPostpone={async (action: PostponeAction) => {
            if (isPrimary) await neko.postponeBreak(action)
          }}
        />
      )}
    </div>
  )
}
