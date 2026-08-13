use std::time::Duration;

use chrono::{Datelike, Timelike};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

pub(crate) mod state;

use crate::{cmd, monitors::idle, platform, scheduler::state::AppState};

const STATS_FLUSH_INTERVAL_MS: i64 = 15_000;

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
            tick(&app);
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

pub fn start_break_now<R: Runtime>(app: &AppHandle<R>) {
    let should_start = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        scheduler.started_from_tray = true;
        scheduler.break_time_ms = Some(now_ms());
        !scheduler.having_break
    };
    if should_start {
        trigger_break(app);
    }
}

pub fn postpone_break<R: Runtime>(app: &AppHandle<R>, action: &str) {
    let now = now_ms();
    let postpone_seconds = postpone_length_seconds(app);
    let frequency_seconds = break_frequency_seconds(app);
    let state = app.state::<AppState>();
    let mut scheduler = state.scheduler.lock();
    if action == "snoozed" && !allow_postpone_locked(&scheduler, app) {
        return;
    }
    scheduler.postponed_count = scheduler.postponed_count.saturating_add(1);
    clear_active_break_locked(&mut scheduler);
    if action == "skipped" {
        schedule_next_locked(&mut scheduler, now, frequency_seconds);
    } else {
        schedule_next_locked(&mut scheduler, now, postpone_seconds);
    }
    drop(scheduler);
    let _ = app.emit("neko://break/end", ());
    platform::close_break_windows(app);
    platform::refresh_tray(app);
}

pub fn begin_popup_break<R: Runtime>(app: &AppHandle<R>) -> i64 {
    let end_at = now_ms() + break_length_seconds_for_active_break(app) * 1_000;
    let state = app.state::<AppState>();
    let mut scheduler = state.scheduler.lock();
    let was_started = scheduler.break_started_at.is_some();
    if !was_started {
        scheduler.break_started_at = Some(std::time::Instant::now());
        scheduler.break_end_at_ms = Some(end_at);
    }
    let end_at = scheduler.break_end_at_ms.unwrap_or(end_at);
    drop(scheduler);
    if !was_started {
        let _ = app.emit("neko://break/start", end_at);
    }
    platform::refresh_tray(app);
    end_at
}

pub fn end_popup_break<R: Runtime>(app: &AppHandle<R>) {
    let frequency = break_frequency_seconds(app);
    let (duration_ms, should_record, was_preview) = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        let scheduled_break_time = scheduler.break_time_ms;
        let postponed_count = scheduler.postponed_count;
        let duration = scheduler
            .break_started_at
            .take()
            .map(|started| started.elapsed().as_millis() as u64);
        let was_preview = scheduler.preview_active;
        let should_record = !was_preview && duration.is_some();
        let now = now_ms();
        clear_active_break_locked(&mut scheduler);
        if was_preview {
            scheduler.postponed_count = postponed_count;
            if scheduled_break_time.is_some_and(|time| time > now) {
                scheduler.break_time_ms = scheduled_break_time;
            } else {
                schedule_next_locked(&mut scheduler, now, frequency);
                scheduler.postponed_count = postponed_count;
            }
        } else {
            scheduler.postponed_count = 0;
            schedule_next_locked(&mut scheduler, now, frequency);
        }
        (duration, should_record, was_preview)
    };

    if should_record && let Some(duration_ms) = duration_ms {
        let state = app.state::<AppState>();
        if state.record_break(duration_ms) {
            state.scheduler.lock().last_completed_at_ms = Some(now_ms());
        }
        if let Err(error) = state.save_config() {
            tracing::warn!("could not persist Break stats: {error}");
        }
    }
    let _ = app.emit("neko://break/end", ());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(450)).await;
        platform::close_break_windows(&app);
        platform::refresh_tray(&app);
    });
    let _ = was_preview;
}

pub fn break_window_destroyed<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let should_finish = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        let known_window = scheduler
            .break_window_labels
            .iter()
            .any(|item| item == label);
        scheduler.break_window_labels.retain(|item| item != label);
        scheduler
            .break_window_ready_labels
            .retain(|item| item != label);
        known_window && scheduler.having_break && !scheduler.preview_relaunch_pending
    };
    if should_finish {
        end_popup_break(app);
    }
}

pub fn complete_break_tracking<R: Runtime>(app: &AppHandle<R>, duration_ms: u64) {
    let is_preview = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        let preview = scheduler.preview_active;
        scheduler.break_started_at = None;
        preview
    };
    if !is_preview {
        let state = app.state::<AppState>();
        let completed = state.record_break(duration_ms);
        if completed {
            state.scheduler.lock().last_completed_at_ms = Some(now_ms());
        }
        if let Err(error) = state.save_config() {
            tracing::warn!("could not persist Break stats: {error}");
        }
    }
}

pub fn preview_break<R: Runtime>(app: &AppHandle<R>, settings: Value) -> Result<(), String> {
    let mut settings = crate::config::normalize_settings(&settings);
    set_string(&mut settings, "notificationType", "POPUP");
    set_bool(&mut settings, "endBreakEnabled", true);
    let was_active = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        if scheduler.having_break {
            scheduler.preview_relaunch_pending = true;
            scheduler.preview_active = true;
            scheduler.break_started_at = None;
            scheduler.break_end_at_ms = None;
            scheduler.started_from_tray = true;
            scheduler.active_break_settings = Some(settings.clone());
            true
        } else {
            scheduler.having_break = true;
            scheduler.started_from_tray = true;
            scheduler.preview_active = true;
            scheduler.active_break_settings = Some(settings.clone());
            false
        }
    };
    if was_active {
        let _ = app.emit("neko://break/end", ());
        platform::close_break_windows(app);
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(450)).await;
            let should_launch = {
                let state = app.state::<AppState>();
                let mut scheduler = state.scheduler.lock();
                if !scheduler.preview_relaunch_pending {
                    false
                } else {
                    scheduler.preview_relaunch_pending = false;
                    scheduler.having_break = true;
                    true
                }
            };
            if should_launch {
                if let Err(error) = platform::create_break_windows(&app) {
                    tracing::warn!("could not relaunch Break preview: {error}");
                    clear_active_break_locked(&mut app.state::<AppState>().scheduler.lock());
                } else {
                    platform::refresh_tray(&app);
                }
            }
        });
        return Ok(());
    }
    if let Err(error) = platform::create_break_windows(app) {
        let state = app.state::<AppState>();
        clear_active_break_locked(&mut state.scheduler.lock());
        return Err(error);
    }
    platform::refresh_tray(app);
    Ok(())
}

pub fn disable_breaks_for<R: Runtime>(app: &AppHandle<R>, duration_ms: Option<i64>) {
    {
        let state = app.state::<AppState>();
        let mut config = state.config.lock();
        set_bool(&mut config.settings, "breaksEnabled", false);
        config.disable_end_time = duration_ms.map(|duration| now_ms() + duration.max(0));
    }
    if let Err(error) = app.state::<AppState>().save_config() {
        tracing::warn!("could not save disabled Break settings: {error}");
    }
    reset_schedule(app);
}

pub fn enable_breaks<R: Runtime>(app: &AppHandle<R>) {
    {
        let state = app.state::<AppState>();
        let mut config = state.config.lock();
        set_bool(&mut config.settings, "breaksEnabled", true);
        config.disable_end_time = None;
    }
    if let Err(error) = app.state::<AppState>().save_config() {
        tracing::warn!("could not save enabled Break settings: {error}");
    }
    reset_schedule(app);
}

pub fn allow_postpone<R: Runtime>(app: &AppHandle<R>) -> bool {
    let limit = integer_setting(app, "postponeLimit", 0).max(0) as u32;
    let postponed = app.state::<AppState>().scheduler.lock().postponed_count;
    limit == 0 || postponed < limit
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
    let (settings, mut today) = {
        let config = state.config.lock();
        (config.settings.clone(), config.daily_stats.clone())
    };
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
    let frequency = integer_from_settings(&settings, "breakFrequencySeconds", 1_680).max(60);
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

fn tick<R: Runtime>(app: &AppHandle<R>) {
    check_disable_timeout(app);
    let now = now_ms();
    let state = app.state::<AppState>();
    let settings = state.config.lock().settings.clone();
    let in_working_hours = is_within_working_hours(&settings);
    let idle_threshold = integer_from_settings(&settings, "idleResetLengthSeconds", 300).max(1);
    let idle_status = {
        let mut scheduler = state.scheduler.lock();
        idle::read_status(
            &mut scheduler,
            idle_threshold,
            bool_from_settings(&settings, "idleResetEnabled", false),
            now,
        )
    };
    let idle = idle_status.idle;
    let idle_reset_notification = bool_from_settings(&settings, "idleResetNotification", false);
    let breaks_enabled = bool_from_settings(&settings, "breaksEnabled", true);
    let frequency = integer_from_settings(&settings, "breakFrequencySeconds", 1_680).max(1);
    let should_have_break = {
        let scheduler = state.scheduler.lock();
        !scheduler.having_break && breaks_enabled && in_working_hours && !idle
    };
    let mut trigger = false;
    let mut flush_work_seconds = 0;
    let mut idle_reset_minutes = None;

    {
        let mut scheduler = state.scheduler.lock();
        if !idle {
            let idle_start = scheduler
                .idle_start_at_ms
                .take()
                .or(idle_status.lock_start_at_ms.filter(|_| !idle_status.locked));
            if let Some(idle_start) = idle_start {
                scheduler.last_completed_at_ms = Some(now);
                scheduler.postponed_count = 0;
                if idle_reset_notification {
                    idle_reset_minutes = Some(
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

        let seconds_since_last_tick = scheduler
            .last_tick_at_ms
            .map(|last_tick| (now - last_tick).unsigned_abs() as i64 / 1_000)
            .unwrap_or(0);
        let idle_seconds = idle_threshold;
        let break_was_overdue = scheduler.break_time_ms.is_some_and(|time| now > time);
        if idle_status.locked
            && scheduler
                .lock_start_at_ms
                .is_some_and(|start| now.saturating_sub(start) > frequency * 1_000)
        {
            if scheduler.idle_start_at_ms.is_none() {
                scheduler.idle_start_at_ms = scheduler.lock_start_at_ms;
            }
            if !break_was_overdue {
                scheduler.break_time_ms = None;
            }
        } else if seconds_since_last_tick > frequency {
            if !break_was_overdue {
                scheduler.break_time_ms = None;
            }
        } else if seconds_since_last_tick > idle_seconds {
            if scheduler.idle_start_at_ms.is_none() {
                scheduler.idle_start_at_ms = scheduler.last_tick_at_ms;
            }
            if !break_was_overdue {
                schedule_next_locked(&mut scheduler, now, frequency);
            }
        }

        if !should_have_break && !scheduler.having_break && scheduler.break_time_ms.is_some() {
            if scheduler.break_time_ms.is_some_and(|time| now > time) {
                scheduler.pending_break_due = true;
            }
            scheduler.break_time_ms = None;
        } else if should_have_break && scheduler.pending_break_due {
            scheduler.pending_break_due = false;
            trigger = true;
        } else if should_have_break && scheduler.break_time_ms.is_none() {
            schedule_next_locked(&mut scheduler, now, frequency);
        } else if should_have_break && scheduler.break_time_ms.is_some_and(|time| now > time) {
            trigger = true;
        }

        if breaks_enabled && !scheduler.having_break && !idle {
            scheduler.pending_work_seconds += 1;
            if now - scheduler.last_stats_flush_at_ms >= STATS_FLUSH_INTERVAL_MS {
                flush_work_seconds = std::mem::take(&mut scheduler.pending_work_seconds);
                scheduler.last_stats_flush_at_ms = now;
            }
        }
        scheduler.last_tick_at_ms = Some(now);
    }

    if flush_work_seconds > 0 {
        state.add_work_seconds(flush_work_seconds);
        if let Err(error) = state.save_config() {
            tracing::warn!("could not persist work stats: {error}");
        }
    }
    if let Some(minutes) = idle_reset_minutes {
        let _ = app
            .notification()
            .builder()
            .title("Break automatically detected")
            .body(format!("Away for about {minutes} minute(s). Timer reset."))
            .show();
    }
    if trigger {
        trigger_break(app);
    }
    if state.scheduler.lock().having_break {
        platform::maintain_break_windows(app);
    }
    platform::refresh_tray(app);
}

fn trigger_break<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let settings = state.config.lock().settings.clone();
    {
        let mut scheduler = state.scheduler.lock();
        if scheduler.having_break {
            return;
        }
        scheduler.having_break = true;
        scheduler.active_break_settings = Some(settings.clone());
    }

    let starts_immediately = settings
        .get("notificationType")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "NOTIFICATION")
        || bool_from_settings(&settings, "immediatelyStartBreaks", true)
        || app.state::<AppState>().scheduler.lock().started_from_tray;
    if starts_immediately
        && settings.get("notificationType").and_then(Value::as_str) != Some("NOTIFICATION")
    {
        begin_popup_break(app);
    }

    if settings.get("notificationType").and_then(Value::as_str) == Some("NOTIFICATION") {
        trigger_notification_break(app, &settings);
        return;
    }

    if let Err(error) = platform::create_break_windows(app) {
        tracing::error!("could not create Break windows: {error}");
        let frequency = break_frequency_seconds(app);
        let mut scheduler = state.scheduler.lock();
        clear_active_break_locked(&mut scheduler);
        schedule_next_locked(&mut scheduler, now_ms(), frequency);
    }
    platform::refresh_tray(app);
}

fn trigger_notification_break<R: Runtime>(app: &AppHandle<R>, settings: &Value) {
    let title = string_from_settings(settings, "breakTitle")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Time for a break.".to_owned());
    let body = string_from_settings(settings, "breakMessage")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Give your eyes a rest. Take a moment to unwind.".to_owned());
    if let Err(error) = app
        .notification()
        .builder()
        .title(title)
        .body(strip_html(&body))
        .show()
    {
        tracing::warn!("could not show Break notification: {error}");
    }
    let sound_type =
        string_from_settings(settings, "soundType").unwrap_or_else(|| "NONE".to_owned());
    let volume = settings
        .get("breakSoundVolume")
        .and_then(Value::as_f64)
        .unwrap_or(1.0) as f32;
    let state = app.state::<AppState>();
    if let Err(error) = cmd::play_sound(app, &state, sound_type, Some(volume), "start") {
        tracing::warn!("could not play Break start sound: {error}");
    }

    let length = integer_from_settings(settings, "breakLengthSeconds", 120).max(1);
    state.complete_notification_break(length, now_ms());
    if let Err(error) = state.save_config() {
        tracing::warn!("could not persist notification Break stats: {error}");
    }
    let frequency = break_frequency_seconds(app);
    {
        let mut scheduler = state.scheduler.lock();
        clear_active_break_locked(&mut scheduler);
        scheduler.postponed_count = 0;
        schedule_next_locked(&mut scheduler, now_ms(), frequency);
    }
    platform::refresh_tray(app);
}

fn check_disable_timeout<R: Runtime>(app: &AppHandle<R>) {
    let expired = app
        .state::<AppState>()
        .config
        .lock()
        .disable_end_time
        .is_some_and(|end| now_ms() >= end);
    if expired {
        enable_breaks(app);
    }
}

fn clear_active_break_locked(scheduler: &mut crate::scheduler::state::SchedulerState) {
    scheduler.having_break = false;
    scheduler.started_from_tray = false;
    scheduler.break_started_at = None;
    scheduler.break_end_at_ms = None;
    scheduler.preview_active = false;
    scheduler.preview_relaunch_pending = false;
    scheduler.active_break_settings = None;
    scheduler.break_time_ms = None;
}

fn schedule_next_locked(
    scheduler: &mut crate::scheduler::state::SchedulerState,
    now: i64,
    delay_seconds: i64,
) {
    if scheduler.idle_start_at_ms.is_some() {
        scheduler.last_completed_at_ms = Some(now);
        scheduler.idle_start_at_ms = None;
        scheduler.postponed_count = 0;
    }
    scheduler.pending_break_due = false;
    scheduler.break_time_ms = Some(now + delay_seconds.max(1) * 1_000);
}

fn allow_postpone_locked<R: Runtime>(
    scheduler: &crate::scheduler::state::SchedulerState,
    app: &AppHandle<R>,
) -> bool {
    let limit = integer_setting(app, "postponeLimit", 0).max(0) as u32;
    limit == 0 || scheduler.postponed_count < limit
}

fn is_within_working_hours(settings: &Value) -> bool {
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

fn break_frequency_seconds<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_setting(app, "breakFrequencySeconds", 1_680).max(1)
}

fn postpone_length_seconds<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_setting(app, "postponeLengthSeconds", 180).max(1)
}

fn break_length_seconds_for_active_break<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_from_settings(&active_settings(app), "breakLengthSeconds", 120).max(1)
}

fn integer_setting<R: Runtime>(app: &AppHandle<R>, key: &str, fallback: i64) -> i64 {
    integer_from_settings(
        &app.state::<AppState>().config.lock().settings,
        key,
        fallback,
    )
}

fn integer_from_settings(settings: &Value, key: &str, fallback: i64) -> i64 {
    settings
        .get(key)
        .and_then(Value::as_i64)
        .unwrap_or(fallback)
}

fn bool_setting<R: Runtime>(app: &AppHandle<R>, key: &str, fallback: bool) -> bool {
    bool_from_settings(
        &app.state::<AppState>().config.lock().settings,
        key,
        fallback,
    )
}

fn bool_from_settings(settings: &Value, key: &str, fallback: bool) -> bool {
    settings
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(fallback)
}

fn string_from_settings(settings: &Value, key: &str) -> Option<String> {
    settings.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn set_bool(settings: &mut Value, key: &str, value: bool) {
    if let Some(object) = settings.as_object_mut() {
        object.insert(key.to_owned(), Value::Bool(value));
    }
}

fn set_string(settings: &mut Value, key: &str, value: &str) {
    if let Some(object) = settings.as_object_mut() {
        object.insert(key.to_owned(), Value::String(value.to_owned()));
    }
}

fn strip_html(value: &str) -> String {
    let value = value
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    let mut text = String::with_capacity(value.len());
    let mut inside_tag = false;
    for character in value.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text
}

fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}
