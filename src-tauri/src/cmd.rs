use std::path::PathBuf;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewWindow, path::BaseDirectory};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    config::{normalize_settings, validate_settings},
    platform,
    scheduler::{self, state::AppState},
};

fn sound_resource_path<R: Runtime>(
    app: &AppHandle<R>,
    sound_type: &str,
    phase: &str,
) -> Result<PathBuf, String> {
    let sound = sound_type.to_ascii_lowercase();
    let path = app
        .path()
        .resolve(
            format!("resources/sounds/{sound}_{phase}.wav"),
            BaseDirectory::Resource,
        )
        .map_err(|error| format!("resolve sound resource: {error}"))?;
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("sound resource does not exist: {}", path.display()))
    }
}

pub(crate) fn play_sound<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    sound_type: String,
    volume: Option<f32>,
    phase: &str,
) -> Result<(), String> {
    if sound_type == "NONE" {
        state.audio.lock().close_device();
        return Ok(());
    }
    let path = sound_resource_path(app, &sound_type, phase)?;
    let volume = volume.unwrap_or(1.0);
    state
        .audio
        .lock()
        .play(&path, volume)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn settings_get(app: AppHandle) -> Value {
    scheduler::active_settings(&app)
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Value,
) -> Result<(), String> {
    let settings = normalize_settings(&settings);
    validate_settings(&settings)?;
    {
        let mut config = state.config.lock();
        config.settings = settings.clone();
    }
    state.save_config().map_err(|error| error.to_string())?;
    if settings.get("soundType").and_then(Value::as_str) == Some("NONE") {
        state.audio.lock().close_device();
    }
    if !cfg!(debug_assertions) {
        let auto_launch = settings
            .get("autoLaunch")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let result = if auto_launch {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        if let Err(error) = result {
            tracing::warn!("could not update login launch setting: {error}");
        }
    }
    scheduler::settings_changed(&app);
    app.emit("neko://settings/changed", settings)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn break_allow_postpone_get(app: AppHandle) -> bool {
    scheduler::allow_postpone(&app)
}

#[tauri::command]
pub fn break_postpone(
    app: AppHandle,
    state: State<'_, AppState>,
    window: WebviewWindow,
    action: String,
) -> Result<(), String> {
    let _ = state;
    platform::require_primary_break_window(&window)?;
    scheduler::postpone_break(&app, &action);
    Ok(())
}

#[tauri::command]
pub fn break_start(app: AppHandle, window: WebviewWindow) -> Result<i64, String> {
    platform::require_primary_break_window(&window)?;
    Ok(scheduler::begin_popup_break(&app))
}

#[tauri::command]
pub fn break_end(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    platform::require_primary_break_window(&window)?;
    scheduler::end_popup_break(&app);
    Ok(())
}

#[tauri::command]
pub fn break_active_end_time_get(app: AppHandle) -> Option<i64> {
    scheduler::active_break_end_time(&app)
}

#[tauri::command]
pub fn break_window_ready(window: WebviewWindow) -> Result<(), String> {
    platform::show_break_window(&window)
}

#[tauri::command]
pub fn break_length_get(app: AppHandle) -> i64 {
    scheduler::active_settings(&app)
        .get("breakLengthSeconds")
        .and_then(Value::as_i64)
        .unwrap_or(120)
}

#[tauri::command]
pub fn break_window_resize(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    platform::resize_break_window(&app, window.label())
}

#[tauri::command]
pub fn break_tracking_complete(
    app: AppHandle,
    state: State<'_, AppState>,
    window: WebviewWindow,
    break_duration_ms: u64,
) -> Result<(), String> {
    let _ = state;
    platform::require_primary_break_window(&window)?;
    scheduler::complete_break_tracking(&app, break_duration_ms);
    Ok(())
}

#[tauri::command]
pub fn time_since_last_break_get(app: AppHandle) -> Option<i64> {
    scheduler::time_since_last_break_seconds(&app)
}

#[tauri::command]
pub fn break_started_from_tray_get(app: AppHandle) -> bool {
    scheduler::started_from_tray(&app)
}

#[tauri::command]
pub fn runtime_status_get(app: AppHandle) -> scheduler::RuntimeStatus {
    scheduler::runtime_status(&app)
}

#[tauri::command]
pub fn app_version_get(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn sound_start_play(
    app: AppHandle,
    state: State<'_, AppState>,
    sound_type: String,
    volume: Option<f32>,
) -> Result<(), String> {
    play_sound(&app, &state, sound_type, volume, "start")
}

#[tauri::command]
pub fn sound_end_play(
    app: AppHandle,
    state: State<'_, AppState>,
    sound_type: String,
    volume: Option<f32>,
) -> Result<(), String> {
    play_sound(&app, &state, sound_type, volume, "end")
}

#[tauri::command]
pub fn break_preview(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Value,
) -> Result<(), String> {
    let _ = state;
    let settings = normalize_settings(&settings);
    validate_settings(&settings)?;
    scheduler::preview_break(&app, settings)
}

#[tauri::command]
pub fn autolaunch_onboarding_seen_get(state: State<'_, AppState>) -> bool {
    state.config.lock().auto_launch_onboarding_seen
}

#[tauri::command]
pub fn autolaunch_onboarding_dismiss(state: State<'_, AppState>) -> Result<(), String> {
    state.config.lock().auto_launch_onboarding_seen = true;
    state.save_config().map_err(|error| error.to_string())
}
