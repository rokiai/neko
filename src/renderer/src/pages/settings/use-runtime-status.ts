import { useEffect, useState } from 'react'
import type { RuntimeStatus } from '@shared/stats'
import { getNekoApi, hasNekoBridge } from '../../lib/neko'

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

export function useRuntimeStatus(): RuntimeStatus {
  const [status, setStatus] = useState<RuntimeStatus>(EMPTY_RUNTIME_STATUS)

  useEffect(() => {
    if (!hasNekoBridge()) return
    let cancelled = false

    const pull = async (): Promise<void> => {
      try {
        const next = await getNekoApi().getRuntimeStatus()
        if (!cancelled) setStatus(next)
      } catch (error) {
        console.warn('[neko] getRuntimeStatus failed', error)
      }
    }

    void pull()
    const id = window.setInterval(() => void pull(), 1000)
    return () => {
      cancelled = true
      window.clearInterval(id)
    }
  }, [])

  return status
}
