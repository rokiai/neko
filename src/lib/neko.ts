import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { NekoApi, PostponeAction } from '@shared/ipc'
import { DEFAULT_SETTINGS, type Settings } from '@shared/settings'
import type { RuntimeStatus } from '@shared/stats'

type TauriPlatform = Extract<NodeJS.Platform, 'darwin' | 'linux' | 'win32'>

function tauriPlatform(): TauriPlatform {
  const platform = navigator.platform
  if (/win/i.test(platform)) return 'win32'
  if (/linux/i.test(platform)) return 'linux'
  return 'darwin'
}

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ != null
}

/**
 * Human-readable text for a failed `invoke`: Tauri command errors reject with
 * plain strings, JS-side failures with `Error`s.
 */
export function invokeErrorText(error: unknown, fallback: string): string {
  if (typeof error === 'string' && error.length > 0) return `${fallback}: ${error}`
  if (error instanceof Error && error.message) return `${fallback}: ${error.message}`
  return fallback
}

function subscribe<T>(event: string, listener: (payload: T) => void): () => void {
  let active = true
  let unlisten: UnlistenFn | null = null

  void listen<T>(event, ({ payload }) => {
    if (active) listener(payload)
  })
    .then((fn) => {
      unlisten = fn
      if (!active) unlisten()
    })
    .catch((error: unknown) => {
      console.warn(`[neko] failed to listen for ${event}`, error)
    })

  return () => {
    active = false
    unlisten?.()
  }
}

const tauriApi: NekoApi = {
  getSettings: () => invoke<Settings>('settings_get'),
  setSettings: (settings) => invoke<void>('settings_set', { settings }),
  getAllowPostpone: () => invoke<boolean>('break_allow_postpone_get'),
  postponeBreak: (action: PostponeAction) => invoke<void>('break_postpone', { action }),
  startBreak: () => invoke<number>('break_start'),
  endBreak: () => invoke<void>('break_end'),
  getActiveBreakEndTime: () => invoke<number | null>('break_active_end_time_get'),
  showBreakWindow: () => invoke<void>('break_window_ready'),
  getBreakLength: () => invoke<number>('break_length_get'),
  resizeBreakWindow: () => invoke<void>('break_window_resize'),
  completeBreakTracking: (breakDurationMs) =>
    invoke<void>('break_tracking_complete', { breakDurationMs }),
  getTimeSinceLastBreak: () => invoke<number | null>('time_since_last_break_get'),
  wasStartedFromTray: () => invoke<boolean>('break_started_from_tray_get'),
  getRuntimeStatus: () => invoke<RuntimeStatus>('runtime_status_get'),
  getAppVersion: () => invoke<string>('app_version_get'),
  playStartSound: (type, volume?: number) =>
    invoke<void>('sound_start_play', { soundType: type, volume }),
  playEndSound: (type, volume?: number) =>
    invoke<void>('sound_end_play', { soundType: type, volume }),
  previewBreak: (settings) => invoke<void>('break_preview', { settings }),
  getAutoLaunchOnboardingSeen: () => invoke<boolean>('autolaunch_onboarding_seen_get'),
  dismissAutoLaunchOnboarding: () => invoke<void>('autolaunch_onboarding_dismiss'),
  onBreakStart: (listener) => subscribe<number>('neko://break/start', listener),
  onBreakEnd: (listener) => subscribe<void>('neko://break/end', listener),
  onRuntimeStatus: (listener) => subscribe<RuntimeStatus>('neko://runtime/status', listener),
  platform: tauriPlatform()
}

let mockSettings: Settings = structuredClone(DEFAULT_SETTINGS)

const browserRuntimeStatus: RuntimeStatus = {
  breaksEnabled: false,
  havingBreak: false,
  idle: false,
  outsideWorkingHours: false,
  secondsToNextBreak: null,
  today: { dayKey: '', workSeconds: 0, restSeconds: 0, completedBreaks: 0 },
  dailyGoal: 8,
  focusStars: 0,
  progressPercent: 0
}

/** True when the Vite renderer is opened outside a desktop runtime. */
export function isBrowserPreview(): boolean {
  return !isTauriRuntime()
}

const browserApi: NekoApi = {
  getSettings: async () => mockSettings,
  setSettings: async (settings) => {
    mockSettings = structuredClone(settings)
  },
  getAllowPostpone: async () => true,
  postponeBreak: async () => undefined,
  startBreak: async () => Date.now() + mockSettings.breakLengthSeconds * 1_000,
  endBreak: async () => undefined,
  getActiveBreakEndTime: async () => null,
  showBreakWindow: async () => undefined,
  getBreakLength: async () => mockSettings.breakLengthSeconds,
  resizeBreakWindow: async () => undefined,
  completeBreakTracking: async () => undefined,
  getTimeSinceLastBreak: async () => null,
  wasStartedFromTray: async () => false,
  getRuntimeStatus: async () => browserRuntimeStatus,
  getAppVersion: async () => 'preview',
  playStartSound: async () => undefined,
  playEndSound: async () => undefined,
  previewBreak: async () => undefined,
  getAutoLaunchOnboardingSeen: async () => true,
  dismissAutoLaunchOnboarding: async () => undefined,
  onBreakStart: () => () => undefined,
  onBreakEnd: () => () => undefined,
  onRuntimeStatus: () => () => undefined,
  platform: tauriPlatform()
}

/** Returns the Tauri command adapter, or a browser-only UI preview adapter. */
export function getNekoApi(): NekoApi {
  if (isTauriRuntime()) return tauriApi
  return browserApi
}
