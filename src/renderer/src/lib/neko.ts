import type { NekoApi } from '@shared/ipc'

export function hasNekoBridge(): boolean {
  return typeof window !== 'undefined' && typeof window.neko !== 'undefined'
}

/** True when the Vite renderer is opened outside Electron (e.g. Chrome). */
export function isBrowserPreview(): boolean {
  return (
    !hasNekoBridge() && typeof navigator !== 'undefined' && !/Electron/i.test(navigator.userAgent)
  )
}

/** Safe accessor — avoids crashing if preload failed to inject. */
export function getNekoApi(): NekoApi {
  const api = window.neko
  if (!api) {
    if (isBrowserPreview()) {
      throw new Error(
        'Neko must run inside the Electron app. Do not open http://localhost:5173 in a browser — use `pnpm dev` and the Neko window/tray.'
      )
    }
    throw new Error('Neko preload bridge is unavailable (window.neko is missing)')
  }
  return api
}
