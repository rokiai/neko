import { SoundType, type WorkingHoursDayKey } from '@shared/settings'

export const DAY_LABEL_KEY: Record<WorkingHoursDayKey, string> = {
  workingHoursMonday: 'day.monday',
  workingHoursTuesday: 'day.tuesday',
  workingHoursWednesday: 'day.wednesday',
  workingHoursThursday: 'day.thursday',
  workingHoursFriday: 'day.friday',
  workingHoursSaturday: 'day.saturday',
  workingHoursSunday: 'day.sunday'
}

export const SOUND_LABEL_KEY: Record<SoundType, string> = {
  [SoundType.None]: 'sound.none',
  [SoundType.Gong]: 'sound.gong',
  [SoundType.Blip]: 'sound.blip',
  [SoundType.Bloop]: 'sound.bloop',
  [SoundType.Ping]: 'sound.ping',
  [SoundType.Scifi]: 'sound.scifi'
}
