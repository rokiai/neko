import { BrowserWindow } from 'electron'
import type { IpcChannelName } from '@shared/ipc'

const listeners = new Set<() => BrowserWindow[]>()

export function registerWindowProvider(provider: () => BrowserWindow[]): void {
  listeners.add(provider)
}

export function broadcast(channel: IpcChannelName, ...args: unknown[]): void {
  const windows = [...listeners].flatMap((provider) => provider())
  for (const win of windows) {
    if (!win.isDestroyed()) {
      win.webContents.send(channel, ...args)
    }
  }
}
