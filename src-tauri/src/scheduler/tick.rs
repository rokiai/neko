//! The 1-second scheduler tick and break triggering (popup or notification).

use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::{
    cmd,
    core::i18n,
    monitors::idle,
    platform,
    scheduler::state::AppState,
    scheduler::transitions::{self, TickInput, clear_active_break_locked, schedule_next_locked},
    scheduler::util::{
        bool_from_settings, break_frequency_seconds, integer_from_settings, now_ms,
        string_from_settings, strip_html,
    },
};

pub(crate) fn tick<R: Runtime>(app: &AppHandle<R>) {
    check_disable_timeout(app);
    let now = now_ms();
    let state = app.state::<AppState>();
    let settings = state.config.lock().settings.clone();
    let idle_threshold = integer_from_settings(&settings, "idleResetLengthSeconds", 300).max(1);

    let outcome = {
        let mut scheduler = state.scheduler.lock();
        let idle_status = idle::read_status(
            &mut scheduler,
            idle_threshold,
            bool_from_settings(&settings, "idleResetEnabled", false),
            now,
        );
        transitions::apply(
            &mut scheduler,
            &TickInput {
                now,
                idle_status,
                in_working_hours: super::is_within_working_hours(&settings),
                breaks_enabled: bool_from_settings(&settings, "breaksEnabled", true),
                frequency_seconds: integer_from_settings(&settings, "breakFrequencySeconds", 1_680)
                    .max(1),
                idle_threshold_seconds: idle_threshold,
                idle_reset_notification: bool_from_settings(
                    &settings,
                    "idleResetNotification",
                    false,
                ),
            },
        )
    };

    if outcome.flush_work_seconds > 0 {
        state.add_work_seconds(outcome.flush_work_seconds);
        if let Err(error) = state.save_config() {
            tracing::warn!("could not persist work stats: {error}");
        }
    }
    if let Some(minutes) = outcome.idle_reset_minutes {
        let locale = i18n::resolve(&settings);
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
    if state.scheduler.lock().having_break {
        platform::maintain_break_windows(app);
    }
    platform::refresh_tray(app);
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
        || bool_from_settings(&settings, "immediatelyStartBreaks", true)
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
        super::enable_breaks(app);
    }
}
