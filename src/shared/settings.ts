export enum NotificationType {
  Notification = 'NOTIFICATION',
  Popup = 'POPUP'
}

export enum SoundType {
  None = 'NONE',
  Gong = 'GONG',
  Blip = 'BLIP',
  Bloop = 'BLOOP',
  Ping = 'PING',
  Scifi = 'SCIFI'
}

export enum TrayTextMode {
  TimeToNextBreak = 'TIME_TO_NEXT_BREAK',
  TimeSinceLastBreak = 'TIME_SINCE_LAST_BREAK'
}

/** How the popup break presents itself. */
export enum BreakPopupStyle {
  Card = 'CARD',
  Video = 'VIDEO'
}

export enum BreakVideoSource {
  Builtin = 'BUILTIN',
  Custom = 'CUSTOM'
}

export type LocalePreference = 'system' | 'en' | 'zh' | 'ja'

export interface WorkingHoursRange {
  fromMinutes: number
  toMinutes: number
}

export interface WorkingHours {
  enabled: boolean
  ranges: WorkingHoursRange[]
}

export type WorkingHoursDayKey =
  | 'workingHoursMonday'
  | 'workingHoursTuesday'
  | 'workingHoursWednesday'
  | 'workingHoursThursday'
  | 'workingHoursFriday'
  | 'workingHoursSaturday'
  | 'workingHoursSunday'

export interface Settings {
  locale: LocalePreference
  autoLaunch: boolean
  breaksEnabled: boolean
  trayTextEnabled: boolean
  trayTextMode: TrayTextMode
  notificationType: NotificationType
  breakFrequencySeconds: number
  breakLengthSeconds: number
  postponeLengthSeconds: number
  postponeLimit: number
  workingHoursEnabled: boolean
  workingHoursMonday: WorkingHours
  workingHoursTuesday: WorkingHours
  workingHoursWednesday: WorkingHours
  workingHoursThursday: WorkingHours
  workingHoursFriday: WorkingHours
  workingHoursSaturday: WorkingHours
  workingHoursSunday: WorkingHours
  idleResetEnabled: boolean
  idleResetLengthSeconds: number
  idleResetNotification: boolean
  soundType: SoundType
  breakSoundVolume: number
  breakTitle: string
  breakMessage: string
  backgroundColor: string
  textColor: string
  showBackdrop: boolean
  backdropOpacity: number
  breakPopupStyle: BreakPopupStyle
  breakVideoSource: BreakVideoSource
  /** Absolute path when breakVideoSource is Custom. */
  breakVideoPath: string
  /** When true, mute the break video and keep the separate break sound. */
  breakVideoMuted: boolean
  endBreakEnabled: boolean
  skipBreakEnabled: boolean
  postponeBreakEnabled: boolean
  immediatelyStartBreaks: boolean
}

const defaultDayHours = (): WorkingHours => ({
  enabled: true,
  // Full day by default so reminders run whenever the machine is on;
  // users can still narrow hours in Settings.
  ranges: [{ fromMinutes: 0, toMinutes: 23 * 60 + 59 }]
})

export const DEFAULT_SETTINGS: Settings = {
  locale: 'system',
  autoLaunch: true,
  breaksEnabled: true,
  trayTextEnabled: false,
  trayTextMode: TrayTextMode.TimeToNextBreak,
  notificationType: NotificationType.Popup,
  breakFrequencySeconds: 28 * 60,
  breakLengthSeconds: 2 * 60,
  postponeLengthSeconds: 3 * 60,
  postponeLimit: 0,
  workingHoursEnabled: true,
  workingHoursMonday: defaultDayHours(),
  workingHoursTuesday: defaultDayHours(),
  workingHoursWednesday: defaultDayHours(),
  workingHoursThursday: defaultDayHours(),
  workingHoursFriday: defaultDayHours(),
  workingHoursSaturday: defaultDayHours(),
  workingHoursSunday: defaultDayHours(),
  idleResetEnabled: false,
  idleResetLengthSeconds: 5 * 60,
  idleResetNotification: false,
  soundType: SoundType.None,
  breakSoundVolume: 1,
  breakTitle: '',
  breakMessage: '',
  backgroundColor: '#3F9F7E',
  textColor: '#F7F3EE',
  showBackdrop: true,
  backdropOpacity: 0.72,
  breakPopupStyle: BreakPopupStyle.Card,
  breakVideoSource: BreakVideoSource.Builtin,
  breakVideoPath: '',
  breakVideoMuted: true,
  endBreakEnabled: true,
  skipBreakEnabled: false,
  postponeBreakEnabled: true,
  immediatelyStartBreaks: true
}

export const WORKING_HOURS_DAY_KEYS: WorkingHoursDayKey[] = [
  'workingHoursMonday',
  'workingHoursTuesday',
  'workingHoursWednesday',
  'workingHoursThursday',
  'workingHoursFriday',
  'workingHoursSaturday',
  'workingHoursSunday'
]

export const SOUND_TYPES: SoundType[] = [
  SoundType.None,
  SoundType.Gong,
  SoundType.Blip,
  SoundType.Bloop,
  SoundType.Ping,
  SoundType.Scifi
]

export const BREAK_POPUP_STYLES: BreakPopupStyle[] = [BreakPopupStyle.Card, BreakPopupStyle.Video]

export const BUILTIN_BREAK_VIDEO = './videos/calm.mp4'

export function usesFullscreenBreakWindow(settings: Settings): boolean {
  return settings.breakPopupStyle === BreakPopupStyle.Video || settings.showBackdrop
}

/** @deprecated use WORKING_HOURS_DAY_KEYS + i18n day.* keys */
export const WORKING_HOURS_DAYS: Array<{ key: WorkingHoursDayKey; label: string }> = [
  { key: 'workingHoursMonday', label: 'Monday' },
  { key: 'workingHoursTuesday', label: 'Tuesday' },
  { key: 'workingHoursWednesday', label: 'Wednesday' },
  { key: 'workingHoursThursday', label: 'Thursday' },
  { key: 'workingHoursFriday', label: 'Friday' },
  { key: 'workingHoursSaturday', label: 'Saturday' },
  { key: 'workingHoursSunday', label: 'Sunday' }
]

/** @deprecated use SOUND_TYPES + i18n sound.* keys */
export const SOUND_OPTIONS: Array<{ value: SoundType; label: string }> = [
  { value: SoundType.None, label: 'Silent' },
  { value: SoundType.Gong, label: 'Gong' },
  { value: SoundType.Blip, label: 'Blip' },
  { value: SoundType.Bloop, label: 'Bloop' },
  { value: SoundType.Ping, label: 'Ping' },
  { value: SoundType.Scifi, label: 'Sci-fi' }
]
