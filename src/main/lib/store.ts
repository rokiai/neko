import ElectronStore from 'electron-store'
import { LEGACY_BREAK_MESSAGE, LEGACY_BREAK_TITLE } from '@shared/break-copy'
import { DEFAULT_SETTINGS, SoundType, type Settings } from '@shared/settings'
import { DEFAULT_DAILY_STATS, localDayKey, type DailyStats } from '@shared/stats'

/** Bump when a one-shot settings migration must run. */
const SETTINGS_VERSION = 4

export interface AppStoreSchema {
  settings: Settings
  settingsVersion: number
  disableEndTime: number | null
  dailyStats: DailyStats
}

type StoreInstance = ElectronStore<AppStoreSchema>
type StoreCtor = new (options?: { name?: string; defaults?: AppStoreSchema }) => StoreInstance

// electron-vite emits CJS for main; electron-store is ESM-only, so
// `require('electron-store')` yields `{ default: Store }` at runtime.
function resolveStoreCtor(): StoreCtor {
  const mod = ElectronStore as unknown as StoreCtor | { default: StoreCtor }
  if (typeof mod === 'function') return mod
  if (mod && typeof mod.default === 'function') return mod.default
  throw new Error('electron-store export is not a constructor')
}

const Store = resolveStoreCtor()

let store: StoreInstance | null = null

function migrateSettings(instance: StoreInstance): void {
  const version = instance.get('settingsVersion') ?? 1
  if (version >= SETTINGS_VERSION) return

  let current: Settings = {
    ...DEFAULT_SETTINGS,
    ...instance.get('settings')
  }

  // v2: product default is silent breaks — mute leftover Gong (old default).
  if (version < 2) {
    current = {
      ...current,
      soundType: SoundType.None
    }
  }

  // v3: clear legacy English break copy so locale defaults apply.
  if (version < 3) {
    current = {
      ...current,
      breakTitle: current.breakTitle === LEGACY_BREAK_TITLE ? '' : current.breakTitle,
      breakMessage: current.breakMessage === LEGACY_BREAK_MESSAGE ? '' : current.breakMessage
    }
  }

  // v4: menu bar shows icon only by default (no countdown title).
  if (version < 4) {
    current = {
      ...current,
      trayTextEnabled: false
    }
  }

  instance.set('settings', current)
  instance.set('settingsVersion', SETTINGS_VERSION)
}

function getStore(): StoreInstance {
  if (!store) {
    store = new Store({
      name: 'neko-config',
      defaults: {
        settings: DEFAULT_SETTINGS,
        settingsVersion: SETTINGS_VERSION,
        disableEndTime: null,
        dailyStats: { ...DEFAULT_DAILY_STATS, dayKey: localDayKey() }
      }
    })
    migrateSettings(store)
  }
  return store
}

export function getPersistedSettings(): Settings {
  return {
    ...DEFAULT_SETTINGS,
    ...getStore().get('settings')
  }
}

let settingsOverride: Settings | null = null

export function getSettings(): Settings {
  if (settingsOverride) {
    return {
      ...DEFAULT_SETTINGS,
      ...settingsOverride
    }
  }
  return getPersistedSettings()
}

export function setSettingsOverride(settings: Settings | null): void {
  settingsOverride = settings
}

export function hasSettingsOverride(): boolean {
  return settingsOverride != null
}

export function setSettings(settings: Settings): void {
  getStore().set('settings', settings)
}

export function getDisableEndTime(): number | null {
  return getStore().get('disableEndTime')
}

export function setDisableEndTime(value: number | null): void {
  getStore().set('disableEndTime', value)
}

export function getDailyStats(): DailyStats {
  const today = localDayKey()
  const saved = getStore().get('dailyStats') ?? { ...DEFAULT_DAILY_STATS, dayKey: today }
  if (saved.dayKey !== today) {
    const fresh: DailyStats = {
      dayKey: today,
      workSeconds: 0,
      restSeconds: 0,
      completedBreaks: 0
    }
    getStore().set('dailyStats', fresh)
    return fresh
  }
  return saved
}

export function setDailyStats(stats: DailyStats): void {
  getStore().set('dailyStats', stats)
}

export function patchDailyStats(patch: Partial<Omit<DailyStats, 'dayKey'>>): DailyStats {
  const current = getDailyStats()
  const next: DailyStats = {
    ...current,
    ...patch,
    dayKey: localDayKey()
  }
  setDailyStats(next)
  return next
}
