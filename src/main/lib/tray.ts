import { Menu, Tray, app, nativeImage } from 'electron'
import { existsSync } from 'fs'
import { APP_NAME } from '@shared/constants'
import { TrayTextMode } from '@shared/settings'
import { isWithinWorkingHours, formatDuration } from '@shared/time'
import { appDisplayName, tt } from './i18n'
import { setTrayIconPath } from './notifications'
import { resolveResource } from './paths'
import {
  getBreakTime,
  getLastCompletedBreakTime,
  getTimeSinceLastBreakSeconds,
  initBreaks,
  isHavingBreak,
  isIdleNow,
  startBreakNow
} from './scheduler'
import { getDisableEndTime, getSettings, setDisableEndTime, setSettings } from './store'
import { createSettingsWindow } from './windows'

let tray: Tray | null = null
let disableTimer: NodeJS.Timeout | null = null

/**
 * Embedded 16×16 template (black/alpha cup silhouette) — survives bad resource paths.
 * Generated into resources/trayTemplate.png as well.
 */
const EMBEDDED_TRAY_TEMPLATE_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAA3ElEQVR42qXTu0pDQRAG4C8Xok0630QQTGWXVCEg2PkMFioIYhAfwDIBA6lCQCtbBa3EFxF8ASvBWzMHlpBDNseBYWb+3X9mdpjln9JYcX6ND7xVLfCLs6rkB9yHX5qkWYK/Yhc1DPFepfU+euGvlOMS/BIXC9g5uimwF1UOMwp18IKnZS2fhM3VxzTBXZIoR4tnj+sRtMLerDHobcyKYAfzyJrTwS0Gy+aQo9NF4tGaw0vnoBlbV8gktq6NTdTxGR9qA6dxb1QQagl5HwfYwg++olIryN94xlXa/x/hMFkg3tjDewAAAABJRU5ErkJggg==',
  'base64'
)

type NekoGlobal = typeof globalThis & { __nekoTray?: Tray }

function asTemplate(image: Electron.NativeImage): Electron.NativeImage {
  if (process.platform === 'darwin') {
    image.setTemplateImage(true)
  }
  return image
}

function trayImage(): Electron.NativeImage {
  if (process.platform === 'darwin') {
    const file1x = resolveResource('trayTemplate.png')
    const file2x = resolveResource('ivan.p@example.net')
    console.log('[neko] tray path', file1x, 'exists=', existsSync(file1x))

    if (existsSync(file1x)) {
      const image = nativeImage.createFromPath(file1x)
      if (!image.isEmpty()) {
        // Prefer crisp @2x representation when present.
        if (existsSync(file2x)) {
          const hi = nativeImage.createFromPath(file2x)
          if (!hi.isEmpty()) {
            image.addRepresentation({
              scaleFactor: 2.0,
              width: 32,
              height: 32,
              buffer: hi.toPNG()
            })
          }
        }
        console.log('[neko] tray from file', image.getSize())
        return asTemplate(image)
      }
    }

    const embedded = nativeImage.createFromBuffer(EMBEDDED_TRAY_TEMPLATE_PNG)
    console.log('[neko] tray from embedded', embedded.getSize(), 'empty=', embedded.isEmpty())
    return asTemplate(embedded)
  }

  const file = resolveResource('icon.png')
  const image = nativeImage.createFromPath(file)
  if (image.isEmpty()) {
    return asTemplate(nativeImage.createFromBuffer(EMBEDDED_TRAY_TEMPLATE_PNG))
  }
  return image.resize({ width: 16, height: 16 })
}

function statusLabel(): string {
  const settings = getSettings()
  if (!settings.breaksEnabled) {
    const until = getDisableEndTime()
    if (until) {
      const remaining = Math.max(0, Math.floor((until - Date.now()) / 1000))
      return tt('tray.disabledLeft', { time: formatDuration(remaining) })
    }
    return tt('tray.disabled')
  }
  if (!isWithinWorkingHours(settings)) return tt('tray.outsideHours')
  if (isIdleNow()) return tt('tray.idle')
  if (isHavingBreak()) return tt('tray.onBreak')
  const next = getBreakTime()
  if (!next) return tt('tray.scheduling')
  const seconds = Math.max(0, Math.floor((next.getTime() - Date.now()) / 1000))
  return tt('tray.nextIn', { time: formatDuration(seconds) })
}

function macTitle(): string {
  if (process.platform !== 'darwin') return ''

  const settings = getSettings()
  // Default: icon only. Optional menu-bar timer can be turned on in Settings.
  if (!settings.trayTextEnabled) return ''

  if (
    !settings.breaksEnabled ||
    !isWithinWorkingHours(settings) ||
    isIdleNow() ||
    isHavingBreak()
  ) {
    return ''
  }

  if (settings.trayTextMode === TrayTextMode.TimeSinceLastBreak) {
    const since = getTimeSinceLastBreakSeconds()
    if (since == null && !getLastCompletedBreakTime()) return ''
    return formatDuration(since ?? 0)
  }

  const next = getBreakTime()
  if (!next) return ''
  return formatDuration(Math.max(0, (next.getTime() - Date.now()) / 1000))
}

function disableFor(ms: number | null): void {
  const settings = getSettings()
  setSettings({ ...settings, breaksEnabled: false })
  setDisableEndTime(ms == null ? null : Date.now() + ms)
  initBreaks()
  buildTray()
}

function enableBreaks(): void {
  const settings = getSettings()
  setSettings({ ...settings, breaksEnabled: true })
  setDisableEndTime(null)
  initBreaks()
  buildTray()
}

function checkDisableTimeout(): void {
  const until = getDisableEndTime()
  if (!until) return
  if (Date.now() >= until) enableBreaks()
}

function endOfDayMs(): number {
  const end = new Date()
  end.setHours(23, 59, 59, 999)
  return Math.max(0, end.getTime() - Date.now())
}

export function buildTray(): void {
  if (!tray) return

  const settings = getSettings()
  const template: Electron.MenuItemConstructorOptions[] = [
    { label: statusLabel(), enabled: false },
    { type: 'separator' },
    {
      label: tt('tray.startNow'),
      click: () => startBreakNow(),
      enabled: settings.breaksEnabled
    },
    { type: 'separator' }
  ]

  if (settings.breaksEnabled) {
    template.push({
      label: tt('tray.disable'),
      submenu: [
        { label: tt('tray.disable.30m'), click: () => disableFor(30 * 60 * 1000) },
        { label: tt('tray.disable.1h'), click: () => disableFor(60 * 60 * 1000) },
        { label: tt('tray.disable.2h'), click: () => disableFor(2 * 60 * 60 * 1000) },
        { label: tt('tray.disable.4h'), click: () => disableFor(4 * 60 * 60 * 1000) },
        { label: tt('tray.disable.eod'), click: () => disableFor(endOfDayMs()) },
        { label: tt('tray.disable.indefinite'), click: () => disableFor(null) }
      ]
    })
  } else {
    template.push({ label: tt('tray.enable'), click: () => enableBreaks() })
  }

  template.push(
    { type: 'separator' },
    { label: tt('tray.settings'), click: () => createSettingsWindow() },
    {
      label: tt('tray.about', { name: appDisplayName() }),
      click: () => {
        createSettingsWindow()
      }
    },
    { type: 'separator' },
    { label: tt('tray.quit'), click: () => app.quit() }
  )

  tray.setContextMenu(Menu.buildFromTemplate(template))
  tray.setToolTip(`${APP_NAME} · ${statusLabel()}`)

  if (process.platform === 'darwin') {
    tray.setTitle(macTitle(), { fontType: 'monospacedDigit' })
  }
}

export function initTray(): void {
  if (tray) {
    try {
      tray.destroy()
    } catch {
      // ignore
    }
    tray = null
  }

  const image = trayImage()
  setTrayIconPath(resolveResource('icon.png'))
  // Stable UUID helps macOS remember menu-bar position after the user Cmd-drags it.
  tray = new Tray(image, 'a3f8c2e1-9b4d-4e6a-8f1c-2d7e5b9a0c3f')
  // Prevent GC from dropping the tray (Electron classic gotcha).
  ;(globalThis as NekoGlobal).__nekoTray = tray
  tray.setToolTip(APP_NAME)

  tray.on('click', () => {
    createSettingsWindow()
  })
  tray.on('double-click', () => {
    createSettingsWindow()
  })

  if (process.platform === 'win32') {
    tray.on('right-click', () => tray?.popUpContextMenu())
  }

  console.log('[neko] tray ready')
  buildTray()

  if (disableTimer) clearInterval(disableTimer)
  disableTimer = setInterval(() => {
    checkDisableTimeout()
    buildTray()
  }, 5000)
}
