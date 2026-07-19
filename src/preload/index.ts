import { contextBridge, ipcRenderer } from 'electron'
import {
  IpcChannel,
  type BreakVideoSrcQuery,
  type NekoApi,
  type PostponeAction
} from '../shared/ipc'
import type { Settings, SoundType } from '../shared/settings'

function subscribe<T extends unknown[]>(
  channel: string,
  listener: (...args: T) => void
): () => void {
  const handler = (_event: Electron.IpcRendererEvent, ...args: T): void => listener(...args)
  ipcRenderer.on(channel, handler)
  return () => {
    ipcRenderer.removeListener(channel, handler)
  }
}

const neko: NekoApi = {
  getSettings: () => ipcRenderer.invoke(IpcChannel.SettingsGet),
  setSettings: (settings: Settings) => ipcRenderer.invoke(IpcChannel.SettingsSet, settings),
  getAllowPostpone: () => ipcRenderer.invoke(IpcChannel.AllowPostponeGet),
  postponeBreak: (action: PostponeAction) => ipcRenderer.invoke(IpcChannel.BreakPostpone, action),
  startBreak: () => ipcRenderer.invoke(IpcChannel.BreakStart),
  endBreak: () => ipcRenderer.invoke(IpcChannel.BreakEnd),
  getBreakLength: () => ipcRenderer.invoke(IpcChannel.BreakLengthGet),
  resizeBreakWindow: () => ipcRenderer.invoke(IpcChannel.BreakWindowResize),
  completeBreakTracking: (breakDurationMs: number) =>
    ipcRenderer.invoke(IpcChannel.BreakTrackingComplete, breakDurationMs),
  getTimeSinceLastBreak: () => ipcRenderer.invoke(IpcChannel.TimeSinceLastBreakGet),
  wasStartedFromTray: () => ipcRenderer.invoke(IpcChannel.WasStartedFromTrayGet),
  getRuntimeStatus: () => ipcRenderer.invoke(IpcChannel.RuntimeStatusGet),
  playStartSound: (type: SoundType, volume?: number) =>
    ipcRenderer.invoke(IpcChannel.SoundStartPlay, type, volume),
  playEndSound: (type: SoundType, volume?: number) =>
    ipcRenderer.invoke(IpcChannel.SoundEndPlay, type, volume),
  getBreakVideoSrc: (query?: BreakVideoSrcQuery) =>
    ipcRenderer.invoke(IpcChannel.BreakVideoSrcGet, query),
  pickBreakVideo: () => ipcRenderer.invoke(IpcChannel.BreakVideoPick),
  importBreakVideo: () => ipcRenderer.invoke(IpcChannel.BreakVideoImport),
  previewBreak: (settings: Settings) => ipcRenderer.invoke(IpcChannel.BreakPreview, settings),
  onBreakStart: (listener) => subscribe(IpcChannel.BreakStart, listener),
  onBreakEnd: (listener) => subscribe(IpcChannel.BreakEnd, listener),
  onSoundStart: (listener) => subscribe(IpcChannel.SoundStartPlay, listener),
  onSoundEnd: (listener) => subscribe(IpcChannel.SoundEndPlay, listener),
  platform: process.platform
}

contextBridge.exposeInMainWorld('neko', neko)
