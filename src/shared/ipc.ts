import type { BreakVideoSource, Settings, SoundType } from './settings'
import type { RuntimeStatus } from './stats'

export const IpcChannel = {
  SettingsGet: 'neko:settings:get',
  SettingsSet: 'neko:settings:set',
  AllowPostponeGet: 'neko:break:allow-postpone',
  BreakPostpone: 'neko:break:postpone',
  BreakStart: 'neko:break:start',
  BreakEnd: 'neko:break:end',
  BreakLengthGet: 'neko:break:length',
  BreakWindowResize: 'neko:break:resize',
  BreakTrackingComplete: 'neko:break:tracking-complete',
  TimeSinceLastBreakGet: 'neko:break:time-since-last',
  WasStartedFromTrayGet: 'neko:break:from-tray',
  RuntimeStatusGet: 'neko:runtime:status',
  AppVersionGet: 'neko:app:version',
  SoundStartPlay: 'neko:sound:start',
  SoundEndPlay: 'neko:sound:end',
  BreakVideoSrcGet: 'neko:break-video:src',
  BreakVideoPick: 'neko:break-video:pick',
  BreakVideoImport: 'neko:break-video:import',
  BreakPreview: 'neko:break:preview'
} as const

export type IpcChannelName = (typeof IpcChannel)[keyof typeof IpcChannel]

export type PostponeAction = 'snoozed' | 'skipped'

export interface BreakVideoSrcQuery {
  breakVideoSource: BreakVideoSource
  breakVideoPath: string
}

export interface NekoApi {
  getSettings: () => Promise<Settings>
  setSettings: (settings: Settings) => Promise<void>
  getAllowPostpone: () => Promise<boolean>
  postponeBreak: (action: PostponeAction) => Promise<void>
  startBreak: () => Promise<void>
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
  getBreakVideoSrc: (query?: BreakVideoSrcQuery) => Promise<string>
  pickBreakVideo: () => Promise<string | null>
  importBreakVideo: () => Promise<string | null>
  previewBreak: (settings: Settings) => Promise<void>
  onBreakStart: (listener: (breakEndTime: number) => void) => () => void
  onBreakEnd: (listener: () => void) => () => void
  onSoundStart: (listener: (type: SoundType, volume: number) => void) => () => void
  onSoundEnd: (listener: (type: SoundType, volume: number) => void) => () => void
  platform: NodeJS.Platform
}
