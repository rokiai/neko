use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

use crate::{
    config::{bool_at, integer_at},
    scheduler::state::AppState,
};

pub(crate) fn now_ms() -> i64 {
    chrono::Local::now().timestamp_millis()
}

pub(crate) fn break_frequency_seconds<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_setting(app, "breakFrequencySeconds", 1_680).max(1)
}

pub(crate) fn postpone_length_seconds<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_setting(app, "postponeLengthSeconds", 180).max(1)
}

pub(crate) fn break_length_seconds_for_active_break<R: Runtime>(app: &AppHandle<R>) -> i64 {
    integer_at(&super::active_settings(app), "breakLengthSeconds", 120).max(1)
}

pub(crate) fn integer_setting<R: Runtime>(app: &AppHandle<R>, key: &str, fallback: i64) -> i64 {
    integer_at(
        &app.state::<AppState>().config.lock().settings,
        key,
        fallback,
    )
}

pub(crate) fn bool_setting<R: Runtime>(app: &AppHandle<R>, key: &str, fallback: bool) -> bool {
    bool_at(
        &app.state::<AppState>().config.lock().settings,
        key,
        fallback,
    )
}

pub(crate) fn string_from_settings(settings: &Value, key: &str) -> Option<String> {
    settings.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub(crate) fn set_bool(settings: &mut Value, key: &str, value: bool) {
    if let Some(object) = settings.as_object_mut() {
        object.insert(key.to_owned(), Value::Bool(value));
    }
}

pub(crate) fn set_string(settings: &mut Value, key: &str, value: &str) {
    if let Some(object) = settings.as_object_mut() {
        object.insert(key.to_owned(), Value::String(value.to_owned()));
    }
}

pub(crate) fn strip_html(value: &str) -> String {
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
