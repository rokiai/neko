import { BrowserWindow, app, screen } from 'electron'
import { existsSync } from 'fs'
import { join } from 'path'
import { is } from '@electron-toolkit/utils'
import { usesFullscreenBreakWindow } from '@shared/settings'
import { getSettings } from './store'
import { registerWindowProvider } from './broadcast'

let settingsWindow: BrowserWindow | null = null
let soundsWindow: BrowserWindow | null = null
let breakWindows: BrowserWindow[] = []
let onBreakWindowsClosed: (() => void) | null = null
let isQuitting = false

export function setAppQuitting(value: boolean): void {
  isQuitting = value
}

function preloadPath(): string {
  const path = join(__dirname, '../preload/index.js')
  if (!existsSync(path)) {
    console.error('[neko] preload script missing:', path)
  }
  return path
}

function webPreferences(): Electron.WebPreferences {
  return {
    preload: preloadPath(),
    sandbox: false,
    contextIsolation: true,
    nodeIntegration: false
  }
}

export function setOnBreakWindowsClosed(fn: () => void): void {
  onBreakWindowsClosed = fn
}

function pageUrl(page: 'settings' | 'break' | 'sounds', windowId?: number): string {
  const params = new URLSearchParams({ page })
  if (typeof windowId === 'number') params.set('windowId', String(windowId))

  if (is.dev && process.env.ELECTRON_RENDERER_URL) {
    return `${process.env.ELECTRON_RENDERER_URL}?${params.toString()}`
  }

  return `${join(__dirname, '../renderer/index.html')}?${params.toString()}`
}

function attachDiagnostics(win: BrowserWindow, label: string): void {
  win.webContents.on('preload-error', (_event, path, error) => {
    console.error(`[neko] preload-error (${label})`, path, error)
  })
  win.webContents.on('did-fail-load', (_event, code, desc, url) => {
    console.error(`[neko] did-fail-load (${label})`, code, desc, url)
  })
}

function loadPage(
  win: BrowserWindow,
  page: 'settings' | 'break' | 'sounds',
  windowId?: number
): void {
  const target = pageUrl(page, windowId)
  console.log(`[neko] load ${page}:`, target)
  if (is.dev && process.env.ELECTRON_RENDERER_URL) {
    void win.loadURL(target)
  } else {
    const [file, query] = target.split('?')
    void win.loadFile(file, { search: query })
  }
}

export function getAllAppWindows(): BrowserWindow[] {
  return [settingsWindow, soundsWindow, ...breakWindows].filter(
    (win): win is BrowserWindow => !!win && !win.isDestroyed()
  )
}

registerWindowProvider(getAllAppWindows)

export function createSoundsWindow(): void {
  if (soundsWindow && !soundsWindow.isDestroyed()) return

  soundsWindow = new BrowserWindow({
    width: 100,
    height: 100,
    show: false,
    skipTaskbar: true,
    webPreferences: webPreferences()
  })
  attachDiagnostics(soundsWindow, 'sounds')

  loadPage(soundsWindow, 'sounds')
  soundsWindow.on('closed', () => {
    soundsWindow = null
  })
}

export function createSettingsWindow(): void {
  if (settingsWindow && !settingsWindow.isDestroyed()) {
    settingsWindow.show()
    settingsWindow.focus()
    return
  }

  settingsWindow = new BrowserWindow({
    width: 1240,
    height: 780,
    minWidth: 1040,
    minHeight: 700,
    show: false,
    autoHideMenuBar: true,
    title: 'Neko',
    backgroundColor: '#EEF6F1',
    webPreferences: webPreferences()
  })
  attachDiagnostics(settingsWindow, 'settings')

  loadPage(settingsWindow, 'settings')

  settingsWindow.on('ready-to-show', () => {
    settingsWindow?.show()
  })

  // Close button hides to the menu-bar tray instead of quitting.
  settingsWindow.on('close', (event) => {
    if (isQuitting) return
    event.preventDefault()
    settingsWindow?.hide()
  })

  settingsWindow.on('closed', () => {
    settingsWindow = null
  })
}

export function createBreakWindows(): void {
  if (breakWindows.length) return

  const settings = getSettings()
  const displays = screen.getAllDisplays()

  displays.forEach((display, index) => {
    const bounds = usesFullscreenBreakWindow(settings)
      ? display.bounds
      : {
          width: 520,
          height: 320,
          x: Math.round(display.bounds.x + (display.bounds.width - 520) / 2),
          y: Math.round(display.bounds.y + (display.bounds.height - 320) / 2)
        }

    const win = new BrowserWindow({
      ...bounds,
      frame: false,
      transparent: true,
      resizable: false,
      movable: false,
      focusable: false,
      hasShadow: false,
      skipTaskbar: true,
      show: false,
      backgroundColor: '#00000000',
      webPreferences: webPreferences()
    })

    win.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true })
    win.setAlwaysOnTop(true, 'screen-saver')
    win.setFullScreenable(false)

    if (process.platform === 'darwin') {
      app.dock?.hide()
    }

    win.on('ready-to-show', () => {
      win.showInactive()
      win.moveTop()
    })

    win.on('closed', () => {
      if (process.platform === 'darwin') {
        app.hide()
      }

      const remaining = breakWindows.filter((item) => item !== win && !item.isDestroyed())
      for (const other of remaining) other.close()
      breakWindows = []
      onBreakWindowsClosed?.()
    })

    loadPage(win, 'break', index)
    breakWindows.push(win)
  })
}

export function closeBreakWindows(): void {
  if (!breakWindows.length) return
  breakWindows[0]?.close()
}

export function resizeBreakWindow(win: BrowserWindow): void {
  const settings = getSettings()
  const display = screen.getDisplayNearestPoint(win.getBounds())

  if (usesFullscreenBreakWindow(settings)) {
    win.setBounds(display.bounds)
    return
  }

  const width = 520
  const height = 320
  win.setBounds({
    width,
    height,
    x: Math.round(display.bounds.x + (display.bounds.width - width) / 2),
    y: Math.round(display.bounds.y + (display.bounds.height - height) / 2)
  })
}
