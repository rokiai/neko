import { useEffect, useState } from 'react'
import type { RuntimeStatus } from '@shared/stats'
import { getNekoApi } from '../../lib/neko'

export const EMPTY_RUNTIME_STATUS: RuntimeStatus = {
  breaksEnabled: false,
  havingBreak: false,
  idle: false,
  outsideWorkingHours: false,
  secondsToNextBreak: null,
  today: { dayKey: '', workSeconds: 0, restSeconds: 0, completedBreaks: 0 },
  dailyGoal: 8,
  focusStars: 0,
  progressPercent: 0
}

/**
 * Subscribes to the status the backend pushes each tick.
 *
 * Deliberately not a poll: closing Settings only hides the window, so a timer
 * here would keep issuing IPC calls and re-rendering a tree nobody can see.
 * The backend pushes only while the window is visible, and pushes once more
 * when it is shown again.
 */
export function useRuntimeStatus(): RuntimeStatus {
  const [status, setStatus] = useState<RuntimeStatus>(EMPTY_RUNTIME_STATUS)

  useEffect(() => {
    let cancelled = false
    const neko = getNekoApi()

    // The first push is up to a tick away; fetch once so the panel is never
    // blank on mount.
    void neko
      .getRuntimeStatus()
      .then((next) => {
        if (!cancelled) setStatus(next)
      })
      .catch((error: unknown) => {
        console.warn('[neko] getRuntimeStatus failed', error)
      })

    const unsubscribe = neko.onRuntimeStatus((next) => {
      if (!cancelled) setStatus(next)
    })

    return () => {
      cancelled = true
      unsubscribe()
    }
  }, [])

  return status
}
