import { app, BrowserWindow } from 'electron'
import { electronApp, optimizer } from '@electron-toolkit/utils'
import { APP_ID } from '@shared/constants'
import { syncAutoLaunch } from './lib/auto-launch'
import { syncLocaleFromSettings } from './lib/i18n'
import { registerIpc } from './lib/ipc'
import {
  endPopupBreak,
  initBreaks,
  setBreakWindowsCloser,
  setBreakWindowsOpener,
  setTrayRebuild
} from './lib/scheduler'
import { buildTray, initTray } from './lib/tray'
import { initUpdater } from './lib/updater'
import { registerVideoProtocolHandler, registerVideoScheme } from './lib/video-protocol'
import {
  createBreakWindows,
  closeBreakWindows,
  createSettingsWindow,
  createSoundsWindow,
  setAppQuitting,
  setOnBreakWindowsClosed
} from './lib/windows'

registerVideoScheme()

const gotLock = app.requestSingleInstanceLock()

if (!gotLock) {
  app.exit()
} else {
  app.on('second-instance', () => {
    createSettingsWindow()
  })

  app.whenReady().then(() => {
    electronApp.setAppUserModelId(APP_ID)
    registerVideoProtocolHandler()

    if (process.platform === 'darwin') {
      app.dock?.hide()
    }

    app.on('browser-window-created', (_, window) => {
      optimizer.watchWindowShortcuts(window)
    })

    setTrayRebuild(buildTray)
    setBreakWindowsOpener(createBreakWindows)
    setBreakWindowsCloser(closeBreakWindows)
    setOnBreakWindowsClosed(endPopupBreak)

    registerIpc()
    syncLocaleFromSettings()
    createSoundsWindow()
    initTray()
    syncAutoLaunch()
    initBreaks()
    initUpdater()
    createSettingsWindow()

    app.on('activate', () => {
      createSettingsWindow()
    })
  })

  // Tray apps stay alive with zero windows.
  app.on('window-all-closed', () => {
    // no-op
  })

  app.on('before-quit', () => {
    setAppQuitting(true)
    for (const win of BrowserWindow.getAllWindows()) {
      win.destroy()
    }
  })
}
