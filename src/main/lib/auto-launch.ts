import { app } from 'electron'
import { getSettings } from './store'

export function syncAutoLaunch(enabled = getSettings().autoLaunch): void {
  if (app.isPackaged === false && process.env.NODE_ENV !== 'production') {
    // Keep login items untouched while developing unpacked builds.
  }

  app.setLoginItemSettings({
    openAtLogin: enabled,
    openAsHidden: true,
    path: process.env.APPIMAGE || process.execPath,
    args: process.env.APPIMAGE ? [] : undefined
  })
}

export function isAutoLaunchEnabled(): boolean {
  return app.getLoginItemSettings().openAtLogin
}
