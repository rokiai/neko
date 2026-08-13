use user_idle2::UserIdle;

use crate::monitors::lock;
use crate::scheduler::state::SchedulerState;

pub const MAX_DETECTION_FAILURES: u8 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IdleStatus {
    pub idle: bool,
    pub locked: bool,
    /// While locked: when the lock began. On the first status after unlocking:
    /// the lock start, but only when the lock lasted at least the threshold —
    /// shorter locks must not reset the schedule.
    pub lock_start_at_ms: Option<i64>,
}

pub fn read_status(
    scheduler: &mut SchedulerState,
    threshold_seconds: i64,
    idle_reset_enabled: bool,
    now_ms: i64,
) -> IdleStatus {
    if scheduler.idle_detection_disabled {
        return IdleStatus::default();
    }

    match UserIdle::get_time() {
        Ok(idle) => {
            scheduler.idle_detection_failures = 0;
            evaluate(
                &mut scheduler.lock_start_at_ms,
                idle.as_seconds(),
                lock::is_screen_locked(),
                threshold_seconds,
                idle_reset_enabled,
                now_ms,
            )
        }
        Err(error) => {
            scheduler.idle_detection_failures = scheduler.idle_detection_failures.saturating_add(1);
            if scheduler.idle_detection_failures >= MAX_DETECTION_FAILURES {
                scheduler.idle_detection_disabled = true;
                tracing::warn!("idle detection disabled after repeated failures: {error}");
            } else {
                tracing::warn!("could not read idle duration: {error}");
            }
            IdleStatus::default()
        }
    }
}

/// Pure idle/lock classification — testable without OS probes.
fn evaluate(
    lock_start_state: &mut Option<i64>,
    idle_seconds: u64,
    locked: bool,
    threshold_seconds: i64,
    idle_reset_enabled: bool,
    now_ms: i64,
) -> IdleStatus {
    let previous_lock_start = *lock_start_state;
    if locked {
        lock_start_state.get_or_insert(now_ms);
    } else {
        *lock_start_state = None;
    }
    let threshold_seconds = threshold_seconds.max(1);
    let threshold_ms = threshold_seconds * 1_000;
    let idle_long_enough = idle_seconds >= threshold_seconds as u64;
    let locked_long_enough =
        lock_start_state.is_some_and(|start| now_ms.saturating_sub(start) >= threshold_ms);
    IdleStatus {
        idle: locked_long_enough || (idle_reset_enabled && idle_long_enough && !locked),
        locked,
        lock_start_at_ms: if locked {
            *lock_start_state
        } else {
            previous_lock_start.filter(|start| now_ms.saturating_sub(*start) >= threshold_ms)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{IdleStatus, evaluate};

    const THRESHOLD: i64 = 300;

    #[test]
    fn short_lock_does_not_report_idle_or_reset() {
        let mut lock_start = None;

        let during = evaluate(&mut lock_start, 3, true, THRESHOLD, false, 1_000);
        assert_eq!(lock_start, Some(1_000));
        assert!(!during.idle);
        assert!(during.locked);

        // Unlock 5 seconds later: no idle, and no finished-lock anchor that
        // would reset "time since last break" or the postpone counter.
        let after = evaluate(&mut lock_start, 0, false, THRESHOLD, false, 6_000);
        assert_eq!(lock_start, None);
        assert_eq!(
            after,
            IdleStatus {
                idle: false,
                locked: false,
                lock_start_at_ms: None
            }
        );
    }

    #[test]
    fn long_lock_reports_idle_then_reset_anchor_once() {
        let mut lock_start = None;

        evaluate(&mut lock_start, 0, true, THRESHOLD, false, 0);
        let while_locked = evaluate(
            &mut lock_start,
            0,
            true,
            THRESHOLD,
            false,
            THRESHOLD * 1_000,
        );
        assert!(
            while_locked.idle,
            "lock beyond threshold counts as idle even with idle reset off"
        );
        assert_eq!(while_locked.lock_start_at_ms, Some(0));

        let first_unlocked = evaluate(
            &mut lock_start,
            0,
            false,
            THRESHOLD,
            false,
            THRESHOLD * 1_000 + 1_000,
        );
        assert!(!first_unlocked.idle);
        assert_eq!(
            first_unlocked.lock_start_at_ms,
            Some(0),
            "finished long lock must surface its start exactly once for the reset"
        );

        let second_unlocked = evaluate(
            &mut lock_start,
            0,
            false,
            THRESHOLD,
            false,
            THRESHOLD * 1_000 + 2_000,
        );
        assert_eq!(second_unlocked.lock_start_at_ms, None);
    }

    #[test]
    fn plain_idle_requires_idle_reset_enabled() {
        let mut lock_start = None;
        let disabled = evaluate(&mut lock_start, 900, false, THRESHOLD, false, 0);
        assert!(!disabled.idle);

        let enabled = evaluate(&mut lock_start, 900, false, THRESHOLD, true, 0);
        assert!(enabled.idle);
    }
}
