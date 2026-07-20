import { app, BrowserWindow, ipcMain, type IpcMainInvokeEvent } from 'electron'
import { existsSync } from 'fs'
import { IpcChannel, type BreakVideoSrcQuery, type PostponeAction } from '@shared/ipc'
import { BUILTIN_BREAK_VIDEO, BreakVideoSource, type Settings } from '@shared/settings'
import { syncAutoLaunch } from './auto-launch'
import { broadcast } from './broadcast'
import {
  beginBreakFromRenderer,
  completeBreakTracking,
  getAllowPostpone,
  getRuntimeStatus,
  getTimeSinceLastBreakSeconds,
  initBreaks,
  postponeBreak,
  previewBreak,
  signalBreakEnd,
  wasBreakStartedFromTray
} from './scheduler'
import { getSettings, setSettings as persistSettings } from './store'
import { syncLocaleFromSettings } from './i18n'
import { buildTray } from './tray'
import { toNekoVideoUrl } from './video-protocol'
import { importBreakVideo } from './break-video-import'
import { closeBreakWindows, resizeBreakWindow } from './windows'

function resolveBreakVideoSrc(query?: BreakVideoSrcQuery): string {
  const settings = getSettings()
  const source = query?.breakVideoSource ?? settings.breakVideoSource
  const filePath = query?.breakVideoPath ?? settings.breakVideoPath
  if (source === BreakVideoSource.Custom && filePath && existsSync(filePath)) {
    return toNekoVideoUrl(filePath)
  }
  return BUILTIN_BREAK_VIDEO
}

export function registerIpc(): void {
  ipcMain.handle(IpcChannel.SettingsGet, () => getSettings())

  ipcMain.handle(IpcChannel.SettingsSet, (_event, settings: Settings) => {
    const prev = getSettings()
    persistSettings(settings)
    syncLocaleFromSettings()

    if (prev.autoLaunch !== settings.autoLaunch) {
      syncAutoLaunch(settings.autoLaunch)
    }

    initBreaks()
    buildTray()
  })

  ipcMain.handle(IpcChannel.AllowPostponeGet, () => getAllowPostpone())

  ipcMain.handle(IpcChannel.BreakPostpone, (_event, action: PostponeAction = 'snoozed') => {
    postponeBreak(action)
    closeBreakWindows()
  })

  ipcMain.handle(IpcChannel.BreakStart, () => {
    beginBreakFromRenderer()
  })

  ipcMain.handle(IpcChannel.BreakEnd, () => {
    signalBreakEnd()
  })

  ipcMain.handle(IpcChannel.BreakLengthGet, () => getSettings().breakLengthSeconds)

  ipcMain.handle(IpcChannel.BreakWindowResize, (event: IpcMainInvokeEvent) => {
    const win = BrowserWindow.fromWebContents(event.sender)
    if (win) resizeBreakWindow(win)
  })

  ipcMain.handle(IpcChannel.BreakTrackingComplete, (_event, durationMs: number) => {
    completeBreakTracking(durationMs)
  })

  ipcMain.handle(IpcChannel.TimeSinceLastBreakGet, () => getTimeSinceLastBreakSeconds())

  ipcMain.handle(IpcChannel.WasStartedFromTrayGet, () => wasBreakStartedFromTray())

  ipcMain.handle(IpcChannel.RuntimeStatusGet, () => getRuntimeStatus())

  ipcMain.handle(IpcChannel.AppVersionGet, () => app.getVersion())

  ipcMain.handle(IpcChannel.SoundStartPlay, (_event, type, volume = 1) => {
    broadcast(IpcChannel.SoundStartPlay, type, volume)
  })

  ipcMain.handle(IpcChannel.SoundEndPlay, (_event, type, volume = 1) => {
    broadcast(IpcChannel.SoundEndPlay, type, volume)
  })

  ipcMain.handle(IpcChannel.BreakVideoSrcGet, (_event, query?: BreakVideoSrcQuery) =>
    resolveBreakVideoSrc(query)
  )

  ipcMain.handle(IpcChannel.BreakVideoPick, async () => importBreakVideo())

  ipcMain.handle(IpcChannel.BreakVideoImport, async () => importBreakVideo())

  ipcMain.handle(IpcChannel.BreakPreview, (_event, settings: Settings) => {
    previewBreak(settings)
  })
}
