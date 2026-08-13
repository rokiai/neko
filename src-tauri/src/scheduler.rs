//! Break scheduler: public API and runtime status.
//!
//! Submodules: `tick` (1s loop + triggering), `transitions` (pure state
//! machine), `breaks` (break lifecycle), `util` (settings access), `state`.

use std::time::Duration;

use chrono::{Datelike, Timelike};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

mod breaks;
pub(crate) mod state;
mod tick;
mod transitions;
mod util;

pub use breaks::{
    allow_postpone, begin_popup_break, break_window_destroyed, complete_break_tracking,
    disable_breaks_for, enable_breaks, end_popup_break, postpone_break, preview_break,
    start_break_now,
};

use crate::{
    platform,
    scheduler::state::AppState,
    scheduler::transitions::schedule_next_locked,
    scheduler::util::{bool_from_settings, bool_setting, break_frequency_seconds, now_ms},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub breaks_enabled: bool,
    pub having_break: bool,
    pub idle: bool,
    pub outside_working_hours: bool,
    pub seconds_to_next_break: Option<i64>,
    pub today: Value,
    pub daily_goal: i64,
    pub focus_stars: i64,
    pub progress_percent: i64,
}

pub fn init<R: Runtime>(app: &AppHandle<R>) {
    reset_schedule(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            tick::tick(&app);
        }
    });
}

pub fn reset_schedule<R: Runtime>(app: &AppHandle<R>) {
    let now = now_ms();
    let enabled = bool_setting(app, "breaksEnabled", true);
    let frequency = break_frequency_seconds(app);
    let state = app.state::<AppState>();
    let mut scheduler = state.scheduler.lock();
    let pending_work_seconds = std::mem::take(&mut scheduler.pending_work_seconds);
    scheduler.having_break = false;
    scheduler.break_time_ms = None;
    scheduler.postponed_count = 0;
    scheduler.idle_start_at_ms = None;
    scheduler.lock_start_at_ms = None;
    scheduler.pending_break_due = false;
    scheduler.started_from_tray = false;
    scheduler.break_started_at = None;
    scheduler.break_end_at_ms = None;
    scheduler.preview_active = false;
    scheduler.preview_relaunch_pending = false;
    scheduler.active_break_settings = None;
    scheduler.break_window_ready_labels.clear();
    if enabled {
        schedule_next_locked(&mut scheduler, now, frequency);
    }
    drop(scheduler);
    platform::close_break_windows(app);
    if pending_work_seconds > 0 {
        state.add_work_seconds(pending_work_seconds);
        if let Err(error) = state.save_config() {
            tracing::warn!("could not persist work stats before scheduler reset: {error}");
        }
    }
    platform::refresh_tray(app);
}

pub fn settings_changed<R: Runtime>(app: &AppHandle<R>) {
    reset_schedule(app);
}

pub fn active_settings<R: Runtime>(app: &AppHandle<R>) -> Value {
    let state = app.state::<AppState>();
    let persisted_settings = state.config.lock().settings.clone();
    state
        .scheduler
        .lock()
        .active_break_settings
        .clone()
        .unwrap_or(persisted_settings)
}

pub fn started_from_tray<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.state::<AppState>().scheduler.lock().started_from_tray
}

pub fn active_break_end_time<R: Runtime>(app: &AppHandle<R>) -> Option<i64> {
    app.state::<AppState>().scheduler.lock().break_end_at_ms
}

pub fn time_since_last_break_seconds<R: Runtime>(app: &AppHandle<R>) -> Option<i64> {
    let state = app.state::<AppState>();
    state
        .scheduler
        .lock()
        .last_completed_at_ms
        .map(|time| (now_ms() - time).max(0) / 1_000)
}

pub fn runtime_status<R: Runtime>(app: &AppHandle<R>) -> RuntimeStatus {
    let now = now_ms();
    let state = app.state::<AppState>();
    let (settings, stats) = {
        let config = state.config.lock();
        (config.settings.clone(), config.daily_stats.clone())
    };
    // Present a fresh zeroed day after midnight even before the next write
    // rolls the persisted stats over.
    let mut today = state::today_stats_snapshot(&stats);
    let scheduler = state.scheduler.lock();
    if let Some(stats) = today.as_object_mut() {
        let worked = stats
            .get("workSeconds")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + scheduler.pending_work_seconds;
        stats.insert("workSeconds".to_owned(), Value::from(worked));
    }
    let completed = today
        .get("completedBreaks")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let frequency = util::integer_from_settings(&settings, "breakFrequencySeconds", 1_680).max(60);
    let daily_goal = ((8 * 60 * 60) / frequency).clamp(4, 16);
    RuntimeStatus {
        breaks_enabled: bool_from_settings(&settings, "breaksEnabled", true),
        having_break: scheduler.having_break,
        idle: scheduler.currently_idle,
        outside_working_hours: bool_from_settings(&settings, "workingHoursEnabled", true)
            && !is_within_working_hours(&settings),
        seconds_to_next_break: scheduler
            .break_time_ms
            .map(|time| (time - now).max(0) / 1_000),
        today,
        daily_goal,
        focus_stars: ((completed as f64 * 5.0 / daily_goal as f64).round() as i64).clamp(0, 5),
        progress_percent: (completed * 100 / daily_goal).clamp(0, 100),
    }
}

pub(crate) fn is_within_working_hours(settings: &Value) -> bool {
    if !bool_from_settings(settings, "workingHoursEnabled", true) {
        return true;
    }
    let day_key = match chrono::Local::now().weekday() {
        chrono::Weekday::Mon => "workingHoursMonday",
        chrono::Weekday::Tue => "workingHoursTuesday",
        chrono::Weekday::Wed => "workingHoursWednesday",
        chrono::Weekday::Thu => "workingHoursThursday",
        chrono::Weekday::Fri => "workingHoursFriday",
        chrono::Weekday::Sat => "workingHoursSaturday",
        chrono::Weekday::Sun => "workingHoursSunday",
    };
    let Some(day) = settings.get(day_key) else {
        return true;
    };
    if !day.get("enabled").and_then(Value::as_bool).unwrap_or(true) {
        return false;
    }
    let now = chrono::Local::now();
    let minutes = i64::from(now.hour() * 60 + now.minute());
    day.get("ranges")
        .and_then(Value::as_array)
        .is_none_or(|ranges| {
            ranges.iter().any(|range| {
                let from = range
                    .get("fromMinutes")
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                let to = range
                    .get("toMinutes")
                    .and_then(Value::as_i64)
                    .unwrap_or(1_439);
                minutes >= from && minutes <= to
            })
        })
}
