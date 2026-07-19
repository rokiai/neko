import { useEffect, useMemo, useState } from 'react'
import { BreakPopupStyle, type Settings } from '@shared/settings'
import { BreakProgress } from './BreakProgress'
import './break.css'

function windowId(): number {
  const raw = new URLSearchParams(window.location.search).get('windowId')
  return raw ? Number(raw) : 0
}

export function BreakPage(): React.JSX.Element {
  const [settings, setSettings] = useState<Settings | null>(null)
  const [phase, setPhase] = useState<'boot' | 'progress' | 'closing'>('boot')
  const [sharedEndTime, setSharedEndTime] = useState<number | null>(null)
  const isPrimary = useMemo(() => windowId() === 0, [])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void (async () => {
        const nextSettings = await window.neko.getSettings()
        setSettings(nextSettings)
        // Skip readiness toast — open the break overlay as soon as time is up.
        setPhase('progress')
      })()
    }, 400)

    const offStart = window.neko.onBreakStart((breakEndTime) => {
      setSharedEndTime(breakEndTime)
      setPhase('progress')
    })

    const offEnd = window.neko.onBreakEnd(() => {
      setPhase('closing')
      window.setTimeout(() => window.close(), 420)
    })

    return () => {
      window.clearTimeout(timer)
      offStart()
      offEnd()
    }
  }, [])

  if (!settings || phase === 'boot') return <div className="break-root" />

  const fullscreen = settings.breakPopupStyle === BreakPopupStyle.Video || settings.showBackdrop

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
      data-style={settings.breakPopupStyle === BreakPopupStyle.Video ? 'video' : 'card'}
    >
      {(phase === 'progress' || phase === 'closing') && (
        <BreakProgress
          settings={settings}
          isPrimary={isPrimary}
          sharedEndTime={sharedEndTime}
          closing={phase === 'closing'}
          onReady={async () => {
            await window.neko.resizeBreakWindow()
            if (isPrimary) await window.neko.startBreak()
          }}
          onFinished={async (elapsedMs) => {
            if (isPrimary) {
              await window.neko.completeBreakTracking(elapsedMs)
              await window.neko.endBreak()
            }
          }}
          onCancel={async () => {
            if (isPrimary) await window.neko.endBreak()
          }}
        />
      )}
    </div>
  )
}
