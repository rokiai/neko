//! Explicit Rust mirror of the TypeScript `Settings` interface
//! (`src/shared/settings.ts`) — the DTO contract at the IPC boundary.
//! Deserializing into this schema validates field presence and types;
//! runtime reads elsewhere keep using `serde_json::Value` with per-key
//! fallbacks. Keep both sides in sync.

use serde::Deserialize;
use serde_json::Value;

/// Validates normalized settings against the schema. Returns a displayable
/// error naming the offending field instead of silently falling back.
pub fn validate_settings(settings: &Value) -> Result<(), String> {
    serde_path_to_error::deserialize::<_, SettingsSchema>(settings.clone())
        .map(|_| ())
        .map_err(|error| {
            format!(
                "settings do not match the expected schema at `{}`: {}",
                error.path(),
                error.inner()
            )
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // deserialization alone enforces the field types
struct SettingsSchema {
    locale: LocalePreference,
    auto_launch: bool,
    breaks_enabled: bool,
    tray_text_enabled: bool,
    tray_text_mode: TrayTextMode,
    notification_type: NotificationType,
    break_frequency_seconds: u32,
    break_length_seconds: u32,
    postpone_length_seconds: u32,
    postpone_limit: u32,
    working_hours_enabled: bool,
    working_hours_monday: WorkingHours,
    working_hours_tuesday: WorkingHours,
    working_hours_wednesday: WorkingHours,
    working_hours_thursday: WorkingHours,
    working_hours_friday: WorkingHours,
    working_hours_saturday: WorkingHours,
    working_hours_sunday: WorkingHours,
    idle_reset_enabled: bool,
    idle_reset_length_seconds: u32,
    idle_reset_notification: bool,
    sound_type: SoundType,
    break_sound_volume: f64,
    break_title: String,
    break_message: String,
    background_color: String,
    text_color: String,
    show_backdrop: bool,
    backdrop_opacity: f64,
    break_popup_style: BreakPopupStyle,
    end_break_enabled: bool,
    skip_break_enabled: bool,
    postpone_break_enabled: bool,
    immediately_start_breaks: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum LocalePreference {
    System,
    En,
    Zh,
    Ja,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TrayTextMode {
    TimeToNextBreak,
    TimeSinceLastBreak,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NotificationType {
    Notification,
    Popup,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SoundType {
    None,
    Gong,
    Blip,
    Bloop,
    Ping,
    Scifi,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BreakPopupStyle {
    Card,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WorkingHours {
    enabled: bool,
    ranges: Vec<WorkingHoursRange>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct WorkingHoursRange {
    from_minutes: u32,
    to_minutes: u32,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::validate_settings;
    use crate::config::{default_settings, normalize_settings};

    #[test]
    fn default_settings_pass_validation() {
        validate_settings(&default_settings()).expect("defaults match the schema");
    }

    #[test]
    fn wrongly_typed_field_is_rejected_with_a_named_error() {
        let mut settings = default_settings();
        settings["breakFrequencySeconds"] = json!("1680");
        let error = validate_settings(&settings).expect_err("string frequency must be rejected");
        assert!(
            error.contains("breakFrequencySeconds"),
            "error was: {error}"
        );
    }

    #[test]
    fn unknown_enum_value_is_rejected() {
        let mut settings = default_settings();
        settings["notificationType"] = json!("BANNER");
        assert!(validate_settings(&settings).is_err());
    }

    #[test]
    fn normalization_makes_partial_input_valid() {
        let settings = normalize_settings(&json!({ "breaksEnabled": false }));
        validate_settings(&settings).expect("normalized partial input matches the schema");
    }
}
