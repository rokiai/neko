use user_idle2::UserIdle;

use crate::scheduler::state::SchedulerState;

pub const MAX_DETECTION_FAILURES: u8 = 3;

pub fn read_seconds(
    scheduler: &mut SchedulerState,
    threshold_seconds: i64,
    idle_reset_enabled: bool,
) -> bool {
    if scheduler.idle_detection_disabled {
        return false;
    }

    match UserIdle::get_time() {
        Ok(idle) => {
            scheduler.idle_detection_failures = 0;
            idle_reset_enabled && idle.as_seconds() >= threshold_seconds.max(1) as u64
        }
        Err(error) => {
            scheduler.idle_detection_failures = scheduler.idle_detection_failures.saturating_add(1);
            if scheduler.idle_detection_failures >= MAX_DETECTION_FAILURES {
                scheduler.idle_detection_disabled = true;
                tracing::warn!("idle detection disabled after repeated failures: {error}");
            } else {
                tracing::warn!("could not read idle duration: {error}");
            }
            false
        }
    }
}
