export interface DailyStats {
  /** Local calendar day key: YYYY-MM-DD */
  dayKey: string
  workSeconds: number
  restSeconds: number
  completedBreaks: number
}

export interface RuntimeStatus {
  breaksEnabled: boolean
  havingBreak: boolean
  idle: boolean
  /** True when working-hours gate is on and now is outside the window */
  outsideWorkingHours: boolean
  /** Seconds until next scheduled break; null when not scheduling */
  secondsToNextBreak: number | null
  today: DailyStats
  /** Soft daily break goal used for progress ring / stars */
  dailyGoal: number
  focusStars: number
  progressPercent: number
}

export const DEFAULT_DAILY_STATS: DailyStats = {
  dayKey: '',
  workSeconds: 0,
  restSeconds: 0,
  completedBreaks: 0
}

export function localDayKey(date = new Date()): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

export function computeDailyGoal(breakFrequencySeconds: number): number {
  const freqMin = Math.max(1, Math.round(breakFrequencySeconds / 60))
  // Roughly an 8-hour focus day worth of breaks, clamped to a friendly range.
  return Math.min(16, Math.max(4, Math.round((8 * 60) / freqMin)))
}

export function computeFocusStars(completed: number, goal: number): number {
  if (goal <= 0) return 0
  return Math.max(0, Math.min(5, Math.round((completed / goal) * 5)))
}

export function computeProgressPercent(completed: number, goal: number): number {
  if (goal <= 0) return 0
  return Math.max(0, Math.min(100, Math.round((completed / goal) * 100)))
}
