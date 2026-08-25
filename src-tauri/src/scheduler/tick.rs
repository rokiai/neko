//! The 1-second scheduler tick and break triggering (popup or notification).

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::{
    cmd,
    config::{bool_at, integer_at},
    core::i18n,
    monitors::idle::IdleStatus,
    platform,
    scheduler::state::AppState,
    scheduler::transitions::{self, TickInput, clear_active_break_locked, schedule_next_locked},
    scheduler::util::{break_frequency_seconds, now_ms, string_from_settings, strip_html},
};

pub(crate) fn tick<R: Runtime>(app: &AppHandle<R>) {
    check_disable_timeout(app);
    let now = now_ms();
    let state = app.state::<AppState>();
    let settings = state.runtime_settings();

    // Idle/lock syscalls are only meaningful while the schedule can run. A
    // disabled timer still needs the 1 Hz loop for disable-until expiry and
    // tray text, but not IOKit / CGSession on every tick.
    let idle_status = if settings.breaks_enabled {
        state.idle.lock().read_status(
            settings.idle_reset_length_seconds,
            settings.idle_reset_enabled,
            now,
        )
    } else {
        IdleStatus::default()
    };

    let (outcome, having_break) = {
        let mut scheduler = state.scheduler.lock();
        let outcome = transitions::apply(
            &mut scheduler,
            &TickInput {
                now,
                idle_status,
                in_working_hours: settings.in_working_hours,
                breaks_enabled: settings.breaks_enabled,
                frequency_seconds: settings.break_frequency_seconds,
                idle_threshold_seconds: settings.idle_reset_length_seconds,
                idle_reset_notification: settings.idle_reset_notification,
            },
        );
        (outcome, scheduler.having_break)
    };

    if outcome.flush_work_seconds > 0 {
        state.add_work_seconds(outcome.flush_work_seconds);
        state.save_config_detached();
    }
    if let Some(minutes) = outcome.idle_reset_minutes {
        let locale = settings.locale;
        let _ = app
            .notification()
            .builder()
            .title(i18n::text(locale, i18n::Text::NotifyIdleTitle))
            .body(i18n::notify_idle_body(locale, minutes))
            .show();
    }
    if outcome.trigger_break {
        trigger_break(app);
    }
    if having_break {
        platform::maintain_break_windows(app);
    }
    platform::refresh_tray_with(app, settings);
    platform::push_runtime_status(app, settings);
}

pub(crate) fn trigger_break<R: Runtime>(app: &AppHandle<R>) {
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
        || bool_at(&settings, "immediatelyStartBreaks", true)
        || app.state::<AppState>().scheduler.lock().started_from_tray;
    if starts_immediately
        && settings.get("notificationType").and_then(Value::as_str) != Some("NOTIFICATION")
    {
        super::begin_popup_break(app);
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
        platform::refresh_tray(app);
        return;
    }
    // begin_popup_break may have emitted before any WebView existed. Broadcast
    // again now so secondary displays that already subscribed can skip polling.
    if let Some(end_at) = super::active_break_end_time(app) {
        let _ = app.emit("neko://break/start", end_at);
    }
    platform::refresh_tray(app);
}

fn trigger_notification_break<R: Runtime>(app: &AppHandle<R>, settings: &Value) {
    let locale = i18n::resolve(settings);
    let title = string_from_settings(settings, "breakTitle")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| i18n::text(locale, i18n::Text::BreakDefaultTitle).to_owned());
    let body = string_from_settings(settings, "breakMessage")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| i18n::text(locale, i18n::Text::BreakDefaultMessage).to_owned());
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

    let length = integer_at(settings, "breakLengthSeconds", 120).max(1);
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
        super::enable_breaks(app);
    }
}
