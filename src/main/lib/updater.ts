import { app } from 'electron'
import { autoUpdater } from 'electron-updater'
import { tt } from './i18n'
import { showNotification } from './notifications'

/**
 * GitHub Releases + electron-updater.
 * Only runs when packaged; needs latest*.yml on the Release assets.
 */
export function initUpdater(): void {
  if (!app.isPackaged) {
    console.log('[neko] updater skipped (dev / unpackaged)')
    return
  }

  autoUpdater.autoDownload = process.platform !== 'win32'
  autoUpdater.autoInstallOnAppQuit = true

  autoUpdater.on('error', (error) => {
    console.warn('[neko] updater error', error.message)
  })

  autoUpdater.on('update-available', (info) => {
    // Windows: no auto-download — nudge the user.
    // macOS / Linux: download starts; still surface an early heads-up.
    showNotification(
      tt('notify.updateAvailable'),
      tt('notify.updateAvailableBody', { version: info.version })
    )
  })

  autoUpdater.on('update-downloaded', () => {
    showNotification(tt('notify.updateReady'), tt('notify.updateReadyBody'))
  })

  void autoUpdater.checkForUpdatesAndNotify().catch((error: unknown) => {
    console.warn('[neko] checkForUpdates failed', error)
  })
}
