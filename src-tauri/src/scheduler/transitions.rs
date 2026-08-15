//! Pure per-tick scheduler state transitions.
//!
//! Everything here operates on `SchedulerState` plus plain inputs — no
//! `AppHandle`, clocks, or OS probes — so the break state machine is unit
//! testable. Side effects (windows, notifications, persistence) are decided by
//! the caller from the returned `TickOutcome`.

use crate::monitors::idle::IdleStatus;
use crate::scheduler::state::SchedulerState;

pub(crate) const STATS_FLUSH_INTERVAL_MS: i64 = 15_000;
/// Work time is derived from consecutive live ticks. Longer gaps are a sleep,
/// app suspension, or scheduler stall and must not be credited as work.
const MAX_ACCOUNTABLE_WORK_GAP_MS: i64 = 5_000;

pub(crate) struct TickInput {
    pub now: i64,
    pub idle_status: IdleStatus,
    pub in_working_hours: bool,
    pub breaks_enabled: bool,
    pub frequency_seconds: i64,
    pub idle_threshold_seconds: i64,
    pub idle_reset_notification: bool,
}

#[derive(Debug, Default)]
pub(crate) struct TickOutcome {
    pub trigger_break: bool,
    /// `Some(minutes)` when an idle/lock period ended and the user opted into
    /// the "timer reset" notification.
    pub idle_reset_minutes: Option<i64>,
    pub flush_work_seconds: i64,
}

pub(crate) fn apply(scheduler: &mut SchedulerState, input: &TickInput) -> TickOutcome {
    let TickInput {
        now,
        idle_status,
        in_working_hours,
        breaks_enabled,
        frequency_seconds: frequency,
        idle_threshold_seconds: idle_threshold,
        idle_reset_notification,
    } = *input;
    let idle = idle_status.idle;
    let should_have_break = !scheduler.having_break && breaks_enabled && in_working_hours && !idle;
    let mut outcome = TickOutcome::default();

    if !idle {
        let idle_start = scheduler
            .idle_start_at_ms
            .take()
            .or(idle_status.lock_start_at_ms.filter(|_| !idle_status.locked));
        if let Some(idle_start) = idle_start {
            scheduler.last_completed_at_ms = Some(now);
            scheduler.postponed_count = 0;
            if idle_reset_notification {
                outcome.idle_reset_minutes = Some(
                    ((now.saturating_sub(idle_start) as f64 / 60_000.0).round() as i64).max(1),
                );
            }
        }
    }
    if !scheduler.was_in_working_hours && in_working_hours {
        scheduler.last_completed_at_ms = Some(now);
    }
    scheduler.was_in_working_hours = in_working_hours;
    scheduler.currently_idle = idle;
    if idle && scheduler.idle_start_at_ms.is_none() {
        scheduler.idle_start_at_ms = idle_status
            .lock_start_at_ms
            .or_else(|| (!idle_status.locked).then_some(now - idle_threshold * 1_000));
    }

    let elapsed_since_last_tick_ms = scheduler
        .last_tick_at_ms
        .map(|last_tick| now.saturating_sub(last_tick))
        .unwrap_or(0);
    let seconds_since_last_tick = scheduler
        .last_tick_at_ms
        .map(|last_tick| (now - last_tick).unsigned_abs() as i64 / 1_000)
        .unwrap_or(0);
    let break_was_overdue = scheduler.break_time_ms.is_some_and(|time| now > time);
    // While locked, `idle_status.lock_start_at_ms` is the live lock anchor, so
    // the state machine never has to reach into the probe's own state.
    if idle_status.locked
        && idle_status
            .lock_start_at_ms
            .is_some_and(|start| now.saturating_sub(start) > frequency * 1_000)
    {
        if scheduler.idle_start_at_ms.is_none() {
            scheduler.idle_start_at_ms = idle_status.lock_start_at_ms;
        }
        if !break_was_overdue {
            scheduler.break_time_ms = None;
        }
    } else if seconds_since_last_tick > frequency {
        if !break_was_overdue {
            scheduler.break_time_ms = None;
        }
    } else if seconds_since_last_tick > idle_threshold {
        if scheduler.idle_start_at_ms.is_none() {
            scheduler.idle_start_at_ms = scheduler.last_tick_at_ms;
        }
        if !break_was_overdue {
            schedule_next_locked(scheduler, now, frequency);
        }
    }

    if !should_have_break && !scheduler.having_break && scheduler.break_time_ms.is_some() {
        if scheduler.break_time_ms.is_some_and(|time| now > time) {
            scheduler.pending_break_due = true;
        }
        scheduler.break_time_ms = None;
    } else if should_have_break && scheduler.pending_break_due {
        scheduler.pending_break_due = false;
        outcome.trigger_break = true;
    } else if should_have_break && scheduler.break_time_ms.is_none() {
        schedule_next_locked(scheduler, now, frequency);
    } else if should_have_break && scheduler.break_time_ms.is_some_and(|time| now > time) {
        outcome.trigger_break = true;
    }

    if breaks_enabled && in_working_hours && !scheduler.having_break && idle_status.work_active {
        scheduler.pending_work_seconds += accountable_work_seconds(elapsed_since_last_tick_ms);
        if now - scheduler.last_stats_flush_at_ms >= STATS_FLUSH_INTERVAL_MS {
            outcome.flush_work_seconds = std::mem::take(&mut scheduler.pending_work_seconds);
            scheduler.last_stats_flush_at_ms = now;
        }
    }
    scheduler.last_tick_at_ms = Some(now);
    outcome
}

fn accountable_work_seconds(elapsed_ms: i64) -> i64 {
    if !(500..=MAX_ACCOUNTABLE_WORK_GAP_MS).contains(&elapsed_ms) {
        return 0;
    }
    ((elapsed_ms + 500) / 1_000).max(1)
}

pub(crate) fn clear_active_break_locked(scheduler: &mut SchedulerState) {
    scheduler.having_break = false;
    scheduler.started_from_tray = false;
    scheduler.break_started_at = None;
    scheduler.break_end_at_ms = None;
    scheduler.preview_active = false;
    scheduler.preview_relaunch_pending = false;
    scheduler.active_break_settings = None;
    scheduler.break_time_ms = None;
}

pub(crate) fn schedule_next_locked(scheduler: &mut SchedulerState, now: i64, delay_seconds: i64) {
    if scheduler.idle_start_at_ms.is_some() {
        scheduler.last_completed_at_ms = Some(now);
        scheduler.idle_start_at_ms = None;
        scheduler.postponed_count = 0;
    }
    scheduler.pending_break_due = false;
    scheduler.break_time_ms = Some(now + delay_seconds.max(1) * 1_000);
}

#[cfg(test)]
mod tests {
    use super::{STATS_FLUSH_INTERVAL_MS, TickInput, accountable_work_seconds, apply};
    use crate::monitors::idle::IdleStatus;
    use crate::scheduler::state::SchedulerState;

    const FREQUENCY: i64 = 1_680;
    const THRESHOLD: i64 = 300;

    fn input(now: i64, idle_status: IdleStatus) -> TickInput {
        TickInput {
            now,
            idle_status,
            in_working_hours: true,
            breaks_enabled: true,
            frequency_seconds: FREQUENCY,
            idle_threshold_seconds: THRESHOLD,
            idle_reset_notification: true,
        }
    }

    fn active() -> IdleStatus {
        IdleStatus {
            work_active: true,
            ..IdleStatus::default()
        }
    }

    #[test]
    fn short_lock_and_unlock_preserves_schedule() {
        let mut scheduler = SchedulerState {
            last_completed_at_ms: Some(1_000),
            postponed_count: 2,
            break_time_ms: Some(900_000),
            last_tick_at_ms: Some(99_000),
            ..SchedulerState::default()
        };
        let locked = IdleStatus {
            work_active: false,
            idle: false,
            locked: true,
            lock_start_at_ms: Some(100_000),
        };
        let outcome = apply(&mut scheduler, &input(105_000, locked));
        assert!(!outcome.trigger_break);

        // Unlocked below the threshold: idle::evaluate suppresses the lock
        // anchor, so the status carries none and nothing may reset.
        let outcome = apply(&mut scheduler, &input(106_000, active()));
        assert_eq!(outcome.idle_reset_minutes, None);
        assert_eq!(scheduler.last_completed_at_ms, Some(1_000));
        assert_eq!(scheduler.postponed_count, 2);
        assert_eq!(scheduler.break_time_ms, Some(900_000));
    }

    #[test]
    fn long_lock_unlock_resets_schedule_once() {
        let mut scheduler = SchedulerState {
            postponed_count: 3,
            break_time_ms: Some(2_000_000),
            last_tick_at_ms: Some(399_000),
            ..SchedulerState::default()
        };
        // First status after a 400s lock carries the anchor exactly once.
        let unlocked = IdleStatus {
            work_active: true,
            idle: false,
            locked: false,
            lock_start_at_ms: Some(0),
        };
        let outcome = apply(&mut scheduler, &input(400_000, unlocked));
        assert_eq!(outcome.idle_reset_minutes, Some(7));
        assert_eq!(scheduler.last_completed_at_ms, Some(400_000));
        assert_eq!(scheduler.postponed_count, 0);

        let outcome = apply(&mut scheduler, &input(401_000, active()));
        assert_eq!(outcome.idle_reset_minutes, None);
    }

    #[test]
    fn overdue_break_while_idle_fires_once_after_resume() {
        let mut scheduler = SchedulerState {
            break_time_ms: Some(10_000),
            last_tick_at_ms: Some(19_000),
            ..SchedulerState::default()
        };
        let idle_now = IdleStatus {
            work_active: false,
            idle: true,
            locked: false,
            lock_start_at_ms: None,
        };
        let outcome = apply(&mut scheduler, &input(20_000, idle_now));
        assert!(!outcome.trigger_break);
        assert!(scheduler.pending_break_due);
        assert_eq!(scheduler.break_time_ms, None);

        let outcome = apply(&mut scheduler, &input(21_000, active()));
        assert!(outcome.trigger_break, "deferred break fires on resume");
        assert!(!scheduler.pending_break_due);

        let outcome = apply(&mut scheduler, &input(22_000, active()));
        assert!(!outcome.trigger_break, "deferred break fires only once");
    }

    #[test]
    fn sleep_gap_reschedules_future_break() {
        let mut scheduler = SchedulerState {
            break_time_ms: Some(7_300_000),
            last_tick_at_ms: Some(0),
            ..SchedulerState::default()
        };
        let outcome = apply(&mut scheduler, &input(7_200_000, active()));
        assert!(!outcome.trigger_break);
        assert_eq!(
            scheduler.break_time_ms,
            Some(7_200_000 + FREQUENCY * 1_000),
            "a not-yet-due break is rescheduled after a long tick gap"
        );
    }

    #[test]
    fn flushes_pending_work_seconds_on_interval() {
        let mut scheduler = SchedulerState {
            break_time_ms: Some(10_000_000),
            pending_work_seconds: 9,
            last_tick_at_ms: Some(STATS_FLUSH_INTERVAL_MS - 1_000),
            ..SchedulerState::default()
        };
        let outcome = apply(&mut scheduler, &input(STATS_FLUSH_INTERVAL_MS, active()));
        assert_eq!(outcome.flush_work_seconds, 10);
        assert_eq!(scheduler.pending_work_seconds, 0);
    }

    #[test]
    fn records_only_recent_activity_during_working_hours() {
        let mut scheduler = SchedulerState {
            break_time_ms: Some(10_000_000),
            last_tick_at_ms: Some(1_000),
            ..SchedulerState::default()
        };
        let outcome = apply(&mut scheduler, &input(2_000, active()));
        assert_eq!(outcome.flush_work_seconds, 0);
        assert_eq!(scheduler.pending_work_seconds, 1);

        let inactive = IdleStatus {
            work_active: false,
            ..IdleStatus::default()
        };
        apply(&mut scheduler, &input(3_000, inactive));
        assert_eq!(scheduler.pending_work_seconds, 1, "idle time is excluded");

        let mut outside_hours = input(4_000, active());
        outside_hours.in_working_hours = false;
        apply(&mut scheduler, &outside_hours);
        assert_eq!(
            scheduler.pending_work_seconds, 1,
            "time outside configured hours is excluded"
        );
    }

    #[test]
    fn does_not_backfill_work_after_a_long_tick_gap() {
        assert_eq!(accountable_work_seconds(999), 1);
        assert_eq!(accountable_work_seconds(1_500), 2);
        assert_eq!(accountable_work_seconds(5_000), 5);
        assert_eq!(accountable_work_seconds(5_001), 0);
        assert_eq!(accountable_work_seconds(60 * 60 * 1_000), 0);

        let mut scheduler = SchedulerState {
            break_time_ms: Some(10_000_000),
            last_tick_at_ms: Some(0),
            ..SchedulerState::default()
        };
        apply(&mut scheduler, &input(60 * 60 * 1_000, active()));
        assert_eq!(scheduler.pending_work_seconds, 0);
    }
}
