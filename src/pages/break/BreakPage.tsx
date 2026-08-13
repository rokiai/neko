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
    const timer = window.setTimeout(() => {
      void (async () => {
        const nextSettings = await neko.getSettings()
        setSettings(nextSettings)
        // Skip readiness toast — open the break overlay as soon as time is up.
        setPhase('progress')
      })()
    }, 400)

    const offStart = neko.onBreakStart((breakEndTime) => {
      setSharedEndTime(breakEndTime)
      setPhase('progress')
    })

    const offEnd = neko.onBreakEnd(() => {
      setPhase('closing')
      window.setTimeout(() => window.close(), 420)
    })

    return () => {
      window.clearTimeout(timer)
      offStart()
      offEnd()
    }
  }, [neko])

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
            await neko.resizeBreakWindow()
            return neko.startBreak()
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
