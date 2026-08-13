//! Break lifecycle: starting, ending, postponing, previewing, and the
//! enable/disable switches.

use std::time::Duration;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::{
    platform,
    scheduler::state::AppState,
    scheduler::transitions::{clear_active_break_locked, schedule_next_locked},
    scheduler::util::{
        break_frequency_seconds, break_length_seconds_for_active_break, integer_setting, now_ms,
        postpone_length_seconds, set_bool, set_string,
    },
};

pub fn start_break_now<R: Runtime>(app: &AppHandle<R>) {
    let should_start = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        scheduler.started_from_tray = true;
        scheduler.break_time_ms = Some(now_ms());
        !scheduler.having_break
    };
    if should_start {
        super::tick::trigger_break(app);
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
    let (duration_ms, should_record) = {
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
        (duration, should_record)
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
    super::reset_schedule(app);
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
    super::reset_schedule(app);
}

pub fn allow_postpone<R: Runtime>(app: &AppHandle<R>) -> bool {
    let limit = integer_setting(app, "postponeLimit", 0).max(0) as u32;
    let postponed = app.state::<AppState>().scheduler.lock().postponed_count;
    limit == 0 || postponed < limit
}

fn allow_postpone_locked<R: Runtime>(
    scheduler: &crate::scheduler::state::SchedulerState,
    app: &AppHandle<R>,
) -> bool {
    let limit = integer_setting(app, "postponeLimit", 0).max(0) as u32;
    limit == 0 || scheduler.postponed_count < limit
}
