import { BrowserWindow, app, dialog, shell } from 'electron'
import { autoUpdater } from 'electron-updater'
import { RELEASES_LATEST_URL } from '@shared/constants'
import { tt } from './i18n'
import { showNotification } from './notifications'

function promptManualDownload(version: string): void {
  const parent = BrowserWindow.getFocusedWindow() ?? BrowserWindow.getAllWindows()[0]
  const options = {
    type: 'info' as const,
    title: tt('update.dialogTitle'),
    message: tt('update.dialogMessage', { version }),
    detail: tt('update.dialogDetail'),
    buttons: [tt('update.dialogDownload'), tt('update.dialogLater')],
    defaultId: 0,
    cancelId: 1,
    noLink: true
  }

  void (
    parent && !parent.isDestroyed()
      ? dialog.showMessageBox(parent, options)
      : dialog.showMessageBox(options)
  ).then((result) => {
    if (result.response === 0) {
      void shell.openExternal(RELEASES_LATEST_URL)
    }
  })
}

/**
 * GitHub Releases + electron-updater.
 * Only runs when packaged; needs latest*.yml on the Release assets.
 *
 * When auto-install cannot complete (e.g. unsigned macOS) or is disabled
 * (Windows), prompt once to open the latest Release download page.
 *
 * Dev preview: `NEKO_PREVIEW_UPDATE_DIALOG=1 pnpm dev`
 */
export function initUpdater(): void {
  if (!app.isPackaged) {
    if (process.env.NEKO_PREVIEW_UPDATE_DIALOG === '1') {
      console.log('[neko] updater dialog preview (dev)')
      setTimeout(() => promptManualDownload('0.1.2'), 800)
    } else {
      console.log('[neko] updater skipped (dev / unpackaged)')
    }
    return
  }

  const autoDownload = process.platform !== 'win32'
  autoUpdater.autoDownload = autoDownload
  autoUpdater.autoInstallOnAppQuit = true

  let pendingVersion: string | null = null
  let updateDownloaded = false
  let manualPromptShown = false

  const promptOnce = (version: string): void => {
    if (manualPromptShown) return
    manualPromptShown = true
    promptManualDownload(version)
  }

  autoUpdater.on('error', (error) => {
    console.warn('[neko] updater error', error.message)
    if (pendingVersion && !updateDownloaded) {
      promptOnce(pendingVersion)
    }
  })

  autoUpdater.on('update-available', (info) => {
    pendingVersion = info.version
    updateDownloaded = false

    showNotification(
      tt('notify.updateAvailable'),
      tt('notify.updateAvailableBody', { version: info.version })
    )

    // Windows: we never auto-download — send the user to Releases.
    if (!autoDownload) {
      promptOnce(info.version)
    }
  })

  autoUpdater.on('update-downloaded', () => {
    updateDownloaded = true
    showNotification(tt('notify.updateReady'), tt('notify.updateReadyBody'))
  })

  void autoUpdater.checkForUpdatesAndNotify().catch((error: unknown) => {
    console.warn('[neko] checkForUpdates failed', error)
    if (pendingVersion && !updateDownloaded) {
      promptOnce(pendingVersion)
    }
  })
}
