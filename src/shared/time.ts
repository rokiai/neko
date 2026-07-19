import type { Settings, WorkingHours, WorkingHoursDayKey } from './settings'

const DAY_KEYS: WorkingHoursDayKey[] = [
  'workingHoursSunday',
  'workingHoursMonday',
  'workingHoursTuesday',
  'workingHoursWednesday',
  'workingHoursThursday',
  'workingHoursFriday',
  'workingHoursSaturday'
]

export function getWorkingHoursForDate(settings: Settings, date = new Date()): WorkingHours {
  return settings[DAY_KEYS[date.getDay()]]
}

export function isWithinWorkingHours(settings: Settings, date = new Date()): boolean {
  if (!settings.workingHoursEnabled) return true

  const day = getWorkingHoursForDate(settings, date)
  if (!day.enabled) return false

  const minutes = date.getHours() * 60 + date.getMinutes()
  return day.ranges.some((range) => minutes >= range.fromMinutes && minutes <= range.toMinutes)
}

export function minutesToLabel(totalMinutes: number): string {
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

export function labelToMinutes(label: string): number {
  const [h, m] = label.split(':').map(Number)
  return h * 60 + m
}

export function formatDuration(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60

  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m`
  if (m > 0) return `${m}m ${String(s).padStart(2, '0')}s`
  return `${s}s`
}

/** Clock-style duration for dashboards: 02:45 */
export function formatClockDuration(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}

/** Compact countdown: 24:30 or 1:05:09 */
export function formatCountdown(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  }
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

export function clampSeconds(value: number, fallback = 1): number {
  if (!Number.isFinite(value) || value <= 0) return fallback
  return Math.floor(value)
}
