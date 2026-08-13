import { useEffect, useState } from 'react'
import { getNekoApi, isTauriRuntime } from '../../lib/neko'

export function useAutoLaunchOnboarding(): {
  open: boolean
  dismiss: () => Promise<void>
} {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    let cancelled = false
    void getNekoApi()
      .getAutoLaunchOnboardingSeen()
      .then((seen) => {
        if (!cancelled && !seen) setOpen(true)
      })
      .catch((error: unknown) => {
        console.warn('[neko] auto-launch onboarding state failed', error)
      })

    return () => {
      cancelled = true
    }
  }, [])

  const dismiss = async (): Promise<void> => {
    setOpen(false)
    if (!isTauriRuntime()) return
    try {
      await getNekoApi().dismissAutoLaunchOnboarding()
    } catch (error: unknown) {
      console.warn('[neko] dismiss auto-launch onboarding failed', error)
    }
  }

  return { open, dismiss }
}
