/* eslint-disable react-refresh/only-export-components -- context module */
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import type { RuntimeStatus } from '@shared/stats'
import { getNekoApi } from '../../lib/neko'

const EMPTY_RUNTIME_STATUS: RuntimeStatus = {
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

const RuntimeStatusContext = createContext<RuntimeStatus | null>(null)

function useRuntimeStatusSource(): RuntimeStatus {
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

/**
 * One backend subscription for the whole Settings tree. Sidebar and Today
 * both read this instead of each opening a listen.
 *
 * Deliberately not a poll: closing Settings only hides the window, so a timer
 * here would keep issuing IPC calls and re-rendering a tree nobody can see.
 * The backend pushes only while the window is visible, and pushes once more
 * when it is shown again.
 */
export function RuntimeStatusProvider({ children }: { children: ReactNode }): React.JSX.Element {
  const status = useRuntimeStatusSource()
  return <RuntimeStatusContext.Provider value={status}>{children}</RuntimeStatusContext.Provider>
}

export function useRuntimeStatus(): RuntimeStatus {
  const status = useContext(RuntimeStatusContext)
  if (status == null) {
    throw new Error('useRuntimeStatus requires RuntimeStatusProvider')
  }
  return status
}
