import type { Settings, SoundType } from './settings'
import type { RuntimeStatus } from './stats'

export type PostponeAction = 'snoozed' | 'skipped'

export interface NekoApi {
  getSettings: () => Promise<Settings>
  setSettings: (settings: Settings) => Promise<void>
  getAllowPostpone: () => Promise<boolean>
  postponeBreak: (action: PostponeAction) => Promise<void>
  /** Starts the shared break clock and returns its absolute end timestamp. */
  startBreak: () => Promise<number>
  endBreak: () => Promise<void>
  getBreakLength: () => Promise<number>
  resizeBreakWindow: () => Promise<void>
  completeBreakTracking: (breakDurationMs: number) => Promise<void>
  getTimeSinceLastBreak: () => Promise<number | null>
  wasStartedFromTray: () => Promise<boolean>
  getRuntimeStatus: () => Promise<RuntimeStatus>
  getAppVersion: () => Promise<string>
  playStartSound: (type: SoundType, volume?: number) => Promise<void>
  playEndSound: (type: SoundType, volume?: number) => Promise<void>
  previewBreak: (settings: Settings) => Promise<void>
  getAutoLaunchOnboardingSeen: () => Promise<boolean>
  dismissAutoLaunchOnboarding: () => Promise<void>
  onBreakStart: (listener: (breakEndTime: number) => void) => () => void
  onBreakEnd: (listener: () => void) => () => void
  platform: NodeJS.Platform
}
