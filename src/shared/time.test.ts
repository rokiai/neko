import { describe, expect, it } from 'vitest'
import { DEFAULT_SETTINGS } from './settings'
import { formatDuration, isWithinWorkingHours, minutesToLabel } from './time'

describe('isWithinWorkingHours', () => {
  it('returns true when working hours are disabled', () => {
    const settings = { ...DEFAULT_SETTINGS, workingHoursEnabled: false }
    expect(isWithinWorkingHours(settings, new Date('2026-07-20T23:00:00'))).toBe(true)
  })

  it('respects weekday ranges', () => {
    const mondayMorning = new Date('2026-07-20T10:00:00')
    const mondayLunch = new Date('2026-07-20T13:00:00')
    const mondayEvening = new Date('2026-07-20T20:00:00')
    const mondayNight = new Date('2026-07-20T22:30:00')
    expect(isWithinWorkingHours(DEFAULT_SETTINGS, mondayMorning)).toBe(true)
    expect(isWithinWorkingHours(DEFAULT_SETTINGS, mondayLunch)).toBe(false)
    expect(isWithinWorkingHours(DEFAULT_SETTINGS, mondayEvening)).toBe(true)
    expect(isWithinWorkingHours(DEFAULT_SETTINGS, mondayNight)).toBe(false)
  })

  it('enables sunday by default', () => {
    const sundayAfternoon = new Date('2026-07-19T15:00:00')
    expect(isWithinWorkingHours(DEFAULT_SETTINGS, sundayAfternoon)).toBe(true)
  })
})

describe('formatDuration', () => {
  it('formats compact durations', () => {
    expect(formatDuration(65)).toBe('1m 05s')
    expect(formatDuration(3661)).toBe('1h 01m')
  })
})

describe('minutesToLabel', () => {
  it('pads hours and minutes', () => {
    expect(minutesToLabel(9 * 60 + 5)).toBe('09:05')
  })
})
