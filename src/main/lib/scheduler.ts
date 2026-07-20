import { powerMonitor } from 'electron'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import { BREAK_COMPLETION_RATIO } from '@shared/constants'
import { IpcChannel } from '@shared/ipc'
import { NotificationType, type Settings } from '@shared/settings'
import {
  computeDailyGoal,
  computeFocusStars,
  computeProgressPercent,
  type RuntimeStatus
} from '@shared/stats'
import { clampSeconds, isWithinWorkingHours } from '@shared/time'
import { broadcast } from './broadcast'
import { showNotification, stripHtml } from './notifications'
import { tt } from './i18n'
import { getDailyStats, getSettings, patchDailyStats, setSettingsOverride } from './store'

type IdleKind = 'active' | 'idle' | 'locked' | 'unknown'

let breakTime: Date | null = null
let havingBreak = false
let postponedCount = 0
let idleStart: Date | null = null
let lockStart: Date | null = null
let lastTick: Date | null = null
let startedFromTray = false
let lastCompletedBreakTime: Date | null = null
let currentBreakStartTime: Date | null = null
let tickTimer: NodeJS.Timeout | null = null
let wasInWorkingHours = true
let currentlyIdle = false

let rebuildTray: (() => void) | null = null
let openBreakWindows: (() => void) | null = null
let closeBreakWindowsFn: (() => void) | null = null
/** Seconds worked today not yet flushed to disk (tick hot path). */
let pendingWorkSeconds = 0
let lastStatsFlush = 0
let previewActive = false
let previewRelaunchPending = false
/** Break was due while idle/outside hours; fire once scheduling resumes. */
let pendingBreakDue = false

function bumpWorkSecond(): void {
  pendingWorkSeconds += 1
  const now = Date.now()
  if (now - lastStatsFlush >= 15_000) flushPendingWorkSeconds()
}

function flushPendingWorkSeconds(): void {
  if (pendingWorkSeconds <= 0) return
  const today = getDailyStats()
  patchDailyStats({ workSeconds: today.workSeconds + pendingWorkSeconds })
  pendingWorkSeconds = 0
  lastStatsFlush = Date.now()
}

function todayWithPending(): ReturnType<typeof getDailyStats> {
  const today = getDailyStats()
  return {
    ...today,
    workSeconds: today.workSeconds + pendingWorkSeconds
  }
}

export function setTrayRebuild(fn: () => void): void {
  rebuildTray = fn
}

export function setBreakWindowsOpener(fn: () => void): void {
  openBreakWindows = fn
}

export function setBreakWindowsCloser(fn: (() => void) | null): void {
  closeBreakWindowsFn = fn
}

export function isBreakPreview(): boolean {
  return previewActive
}

function tray(): void {
  rebuildTray?.()
}

export function getBreakTime(): Date | null {
  return breakTime
}

export function getLastCompletedBreakTime(): Date | null {
  return lastCompletedBreakTime
}

export function getTimeSinceLastBreakSeconds(): number | null {
  if (!lastCompletedBreakTime) return null
  return Math.floor((Date.now() - lastCompletedBreakTime.getTime()) / 1000)
}

export function wasBreakStartedFromTray(): boolean {
  return startedFromTray
}

export function getAllowPostpone(): boolean {
  const { postponeLimit } = getSettings()
  return !postponeLimit || postponedCount < postponeLimit
}

export function resetTimeSinceLastBreak(): void {
  lastCompletedBreakTime = new Date()
}

function checkIdle(settings: Settings): boolean {
  const threshold = clampSeconds(settings.idleResetLengthSeconds, 1)
  const state = powerMonitor.getSystemIdleState(threshold) as IdleKind

  if (state === 'locked') {
    if (!lockStart) {
      lockStart = new Date()
      return false
    }
    const lockedFor = (Date.now() - lockStart.getTime()) / 1000
    return lockedFor > threshold
  }

  lockStart = null
  if (!settings.idleResetEnabled) return false
  return state === 'idle'
}

function maybeNotifyIdle(settings: Settings): void {
  if (!idleStart || !settings.idleResetNotification) return
  const minutes = Math.max(1, Math.round((Date.now() - idleStart.getTime()) / 60000))
  showNotification(tt('notify.idleTitle'), tt('notify.idleBody', { minutes }))
}

export function scheduleNextBreak(isPostpone = false): void {
  const settings = getSettings()

  if (idleStart) {
    maybeNotifyIdle(settings)
    idleStart = null
    postponedCount = 0
    resetTimeSinceLastBreak()
  }

  const delaySeconds = isPostpone
    ? clampSeconds(settings.postponeLengthSeconds, 1)
    : clampSeconds(settings.breakFrequencySeconds, 1)

  pendingBreakDue = false
  breakTime = new Date(Date.now() + delaySeconds * 1000)
  tray()
}

function startBreakTracking(): void {
  if (previewActive) {
    currentBreakStartTime = new Date()
    return
  }
  currentBreakStartTime = new Date()
}

export function completeBreakTracking(breakDurationMs: number): void {
  if (previewActive) {
    currentBreakStartTime = null
    return
  }

  flushPendingWorkSeconds()
  const settings = getSettings()
  const required = settings.breakLengthSeconds * 1000 * BREAK_COMPLETION_RATIO
  const restSeconds = Math.max(0, Math.round(breakDurationMs / 1000))

  if (restSeconds > 0) {
    const today = getDailyStats()
    patchDailyStats({ restSeconds: today.restSeconds + restSeconds })
  }

  if (breakDurationMs >= required) {
    lastCompletedBreakTime = new Date()
    const today = getDailyStats()
    patchDailyStats({ completedBreaks: today.completedBreaks + 1 })
  }

  currentBreakStartTime = null
}

function doBreak(): void {
  const settings = getSettings()
  havingBreak = true

  const shouldTrackImmediately =
    settings.notificationType === NotificationType.Notification ||
    settings.immediatelyStartBreaks ||
    startedFromTray

  if (shouldTrackImmediately) startBreakTracking()

  if (settings.notificationType === NotificationType.Notification) {
    showNotification(
      resolveBreakTitle(settings.breakTitle, tt),
      stripHtml(resolveBreakMessage(settings.breakMessage, tt))
    )
    broadcast(IpcChannel.SoundStartPlay, settings.soundType, settings.breakSoundVolume)
    lastCompletedBreakTime = new Date()
    flushPendingWorkSeconds()
    const today = getDailyStats()
    patchDailyStats({
      completedBreaks: today.completedBreaks + 1,
      restSeconds: today.restSeconds + clampSeconds(settings.breakLengthSeconds, 1)
    })
    havingBreak = false
    startedFromTray = false
    currentBreakStartTime = null
    postponedCount = 0
    scheduleNextBreak()
    return
  }

  openBreakWindows?.()
  tray()
}

export function postponeBreak(action: 'snoozed' | 'skipped' = 'snoozed'): void {
  postponedCount += 1
  havingBreak = false
  startedFromTray = false
  currentBreakStartTime = null

  if (action === 'skipped') {
    scheduleNextBreak(false)
  } else {
    scheduleNextBreak(true)
  }
}

export function endPopupBreak(): void {
  if (previewRelaunchPending) {
    previewRelaunchPending = false
    havingBreak = false
    startedFromTray = false
    currentBreakStartTime = null
    launchBreakPreview()
    return
  }

  if (previewActive) {
    previewActive = false
    setSettingsOverride(null)
    havingBreak = false
    startedFromTray = false
    currentBreakStartTime = null
    tray()
    return
  }

  if (currentBreakStartTime) {
    completeBreakTracking(Date.now() - currentBreakStartTime.getTime())
  }

  havingBreak = false
  startedFromTray = false

  if (!breakTime || breakTime.getTime() <= Date.now()) {
    postponedCount = 0
    breakTime = null
    scheduleNextBreak()
  } else {
    tray()
  }
}

function launchBreakPreview(): void {
  havingBreak = true
  startedFromTray = true
  openBreakWindows?.()
  tray()
}

/** Open the real break UI using draft settings (does not persist or count stats). */
export function previewBreak(draft: Settings): void {
  setSettingsOverride({
    ...draft,
    notificationType: NotificationType.Popup,
    endBreakEnabled: true
  })
  previewActive = true

  if (havingBreak) {
    previewRelaunchPending = true
    closeBreakWindowsFn?.()
    return
  }

  launchBreakPreview()
}

export function startBreakNow(): void {
  startedFromTray = true
  breakTime = new Date()
  if (!havingBreak) {
    doBreak()
  }
}

export function beginBreakFromRenderer(): number {
  startBreakTracking()
  const settings = getSettings()
  const breakEndTime = Date.now() + settings.breakLengthSeconds * 1000
  broadcast(IpcChannel.BreakStart, breakEndTime)
  return breakEndTime
}

export function signalBreakEnd(): void {
  broadcast(IpcChannel.BreakEnd)
}

function tick(): void {
  const settings = getSettings()
  const now = new Date()
  const inWorkingHours = isWithinWorkingHours(settings, now)

  if (!wasInWorkingHours && inWorkingHours) {
    resetTimeSinceLastBreak()
  }
  wasInWorkingHours = inWorkingHours

  const idle = checkIdle(settings)
  currentlyIdle = idle

  const shouldHaveBreak = !havingBreak && settings.breaksEnabled && inWorkingHours && !idle

  try {
    const secondsSinceLastTick = lastTick ? Math.abs(now.getTime() - lastTick.getTime()) / 1000 : 0
    const breakSeconds = clampSeconds(settings.breakFrequencySeconds, 1)
    const idleSeconds = clampSeconds(settings.idleResetLengthSeconds, 1)
    const lockSeconds = lockStart ? (now.getTime() - lockStart.getTime()) / 1000 : 0
    const breakWasOverdue = !!breakTime && now.getTime() > breakTime.getTime()

    if (lockStart && lockSeconds > breakSeconds) {
      idleStart = null
      lockStart = null
    } else if (secondsSinceLastTick > breakSeconds) {
      lockStart = null
      // Sleep longer than one break interval: keep overdue time so we still pop on wake.
      if (!breakWasOverdue) {
        breakTime = null
      }
    } else if (secondsSinceLastTick > idleSeconds) {
      if (!idleStart) idleStart = lastTick
      // Long idle/sleep gap: reset schedule unless a break was already due.
      if (!breakWasOverdue) {
        scheduleNextBreak()
      }
    }

    if (!shouldHaveBreak && !havingBreak && breakTime) {
      if (now.getTime() > breakTime.getTime()) {
        pendingBreakDue = true
      }
      if (idle) {
        idleStart = new Date(now.getTime() - idleSeconds * 1000)
      }
      breakTime = null
      tray()
      return
    }

    if (shouldHaveBreak && pendingBreakDue && !havingBreak) {
      pendingBreakDue = false
      doBreak()
      return
    }

    if (shouldHaveBreak && !breakTime) {
      scheduleNextBreak()
      return
    }

    if (shouldHaveBreak && breakTime && now.getTime() > breakTime.getTime()) {
      doBreak()
    }
  } finally {
    // Count outside early-returns. Work time tracks real focus while Neko is on,
    // even when outside configured working hours (e.g. weekends during development).
    if (settings.breaksEnabled && !havingBreak && !idle) {
      bumpWorkSecond()
    }
    lastTick = new Date()
  }
}

export function initBreaks(): void {
  if (tickTimer) clearInterval(tickTimer)

  const settings = getSettings()
  havingBreak = false
  breakTime = null
  postponedCount = 0
  pendingBreakDue = false

  if (settings.breaksEnabled) {
    scheduleNextBreak()
  } else {
    tray()
  }

  tickTimer = setInterval(tick, 1000)
  tick()
}

export function isHavingBreak(): boolean {
  return havingBreak
}

export function isIdleNow(): boolean {
  return currentlyIdle
}

export function getRuntimeStatus(): RuntimeStatus {
  const settings = getSettings()
  const today = todayWithPending()
  const dailyGoal = computeDailyGoal(settings.breakFrequencySeconds)
  const inWorkingHours = isWithinWorkingHours(settings)
  const outsideWorkingHours = settings.workingHoursEnabled && !inWorkingHours

  const secondsToNextBreak =
    breakTime && !havingBreak
      ? Math.max(0, Math.floor((breakTime.getTime() - Date.now()) / 1000))
      : havingBreak
        ? 0
        : null

  return {
    breaksEnabled: settings.breaksEnabled,
    havingBreak,
    idle: currentlyIdle,
    outsideWorkingHours,
    secondsToNextBreak,
    today,
    dailyGoal,
    focusStars: computeFocusStars(today.completedBreaks, dailyGoal),
    progressPercent: computeProgressPercent(today.completedBreaks, dailyGoal)
  }
}
