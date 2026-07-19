import { Notification, nativeImage, type Tray } from 'electron'
import { APP_NAME } from '@shared/constants'
import { resolveResource } from './paths'

let trayIconPath: string | null = null

export function setTrayIconPath(path: string): void {
  trayIconPath = path
}

function resolveIcon(): Electron.NativeImage | undefined {
  if (process.platform === 'darwin') return undefined
  if (!trayIconPath) return undefined
  return nativeImage.createFromPath(trayIconPath)
}

export function showNotification(title: string, body: string, silent = false): void {
  if (!Notification.isSupported()) return

  const notification = new Notification({
    title: title || APP_NAME,
    body,
    silent: process.platform === 'win32' ? false : silent,
    icon: resolveIcon()
  })

  notification.show()

  if (process.platform !== 'darwin') {
    setTimeout(() => notification.close(), 5000)
  }
}

export function stripHtml(value: string): string {
  return value.replace(/<br\s*\/?>/gi, '\n').replace(/<[^>]+>/g, '')
}

export function getResourcesIconPath(): string {
  return resolveResource('icon.png')
}

export type TrayRef = Tray | null
