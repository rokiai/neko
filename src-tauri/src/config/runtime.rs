//! Typed projection of the stored settings for the 1 Hz hot paths.
//!
//! The scheduler tick, the tray refresh and `runtime_status` only ever read
//! scalars out of the settings JSON. Building this `Copy` snapshot once while
//! the config lock is held replaces the repeated deep clones of the whole
//! settings object (dozens of heap allocations each) that used to happen
//! several times per second, and it resolves the locale once instead of
//! querying the OS locale on every tray refresh.
//!
//! Cold paths that need owned strings (break copy, sound names) keep reading
//! the JSON directly — they run once per break, not once per second.

use serde_json::Value;

use crate::core::i18n::{self, Locale};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayTextMode {
    TimeToNextBreak,
    TimeSinceLastBreak,
}

/// Everything the 1 Hz paths need, as plain scalars.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeSettings {
    pub locale: Locale,
    pub breaks_enabled: bool,
    pub break_frequency_seconds: i64,
    pub idle_reset_enabled: bool,
    pub idle_reset_length_seconds: i64,
    pub idle_reset_notification: bool,
    pub working_hours_enabled: bool,
    /// Already evaluated against the current local time, so the snapshot does
    /// not need to keep the per-weekday ranges around.
    pub in_working_hours: bool,
    pub tray_text_enabled: bool,
    pub tray_text_mode: TrayTextMode,
}

impl RuntimeSettings {
    pub fn from_settings(settings: &Value) -> Self {
        let working_hours_enabled = bool_at(settings, "workingHoursEnabled", true);
        Self {
            locale: i18n::resolve(settings),
            breaks_enabled: bool_at(settings, "breaksEnabled", true),
            break_frequency_seconds: integer_at(settings, "breakFrequencySeconds", 1_680).max(1),
            idle_reset_enabled: bool_at(settings, "idleResetEnabled", false),
            idle_reset_length_seconds: integer_at(settings, "idleResetLengthSeconds", 300).max(1),
            idle_reset_notification: bool_at(settings, "idleResetNotification", false),
            working_hours_enabled,
            in_working_hours: !working_hours_enabled || within_working_hours(settings),
            tray_text_enabled: bool_at(settings, "trayTextEnabled", false),
            tray_text_mode: match settings.get("trayTextMode").and_then(Value::as_str) {
                Some("TIME_SINCE_LAST_BREAK") => TrayTextMode::TimeSinceLastBreak,
                _ => TrayTextMode::TimeToNextBreak,
            },
        }
    }

    /// True when the schedule should report the user as outside working hours.
    pub fn outside_working_hours(&self) -> bool {
        self.working_hours_enabled && !self.in_working_hours
    }
}

/// Reads a boolean out of the settings JSON, falling back per key.
pub fn bool_at(settings: &Value, key: &str, fallback: bool) -> bool {
    settings
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(fallback)
}

/// Reads an integer out of the settings JSON, falling back per key.
pub fn integer_at(settings: &Value, key: &str, fallback: i64) -> i64 {
    settings
        .get(key)
        .and_then(Value::as_i64)
        .unwrap_or(fallback)
}

/// Evaluates today's configured ranges against the current local time.
/// Assumes the caller already checked `workingHoursEnabled`.
fn within_working_hours(settings: &Value) -> bool {
    use chrono::{Datelike, Timelike};

    let now = chrono::Local::now();
    let day_key = match now.weekday() {
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
    let minutes = i64::from(now.hour() * 60 + now.minute());
    day.get("ranges")
        .and_then(Value::as_array)
        .is_none_or(|ranges| {
            ranges.iter().any(|range| {
                let from = integer_at(range, "fromMinutes", 0);
                let to = integer_at(range, "toMinutes", 1_439);
                minutes >= from && minutes <= to
            })
        })
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use serde_json::json;

    use super::{RuntimeSettings, TrayTextMode};

    #[test]
    fn projects_defaults_from_an_empty_object() {
        let snapshot = RuntimeSettings::from_settings(&json!({}));

        assert!(snapshot.breaks_enabled);
        assert_eq!(snapshot.break_frequency_seconds, 1_680);
        assert!(!snapshot.idle_reset_enabled);
        assert_eq!(snapshot.idle_reset_length_seconds, 300);
        assert!(snapshot.working_hours_enabled);
        // No per-day object stored: treated as always within working hours.
        assert!(snapshot.in_working_hours);
        assert!(!snapshot.outside_working_hours());
        assert_eq!(snapshot.tray_text_mode, TrayTextMode::TimeToNextBreak);
    }

    #[test]
    fn disabled_working_hours_never_report_outside() {
        let snapshot = RuntimeSettings::from_settings(&json!({
            "workingHoursEnabled": false,
            "workingHoursMonday": { "enabled": false, "ranges": [] }
        }));

        assert!(snapshot.in_working_hours);
        assert!(!snapshot.outside_working_hours());
    }

    #[test]
    fn a_disabled_weekday_reports_outside_working_hours() {
        let weekday_key = match chrono::Local::now().weekday() {
            chrono::Weekday::Mon => "workingHoursMonday",
            chrono::Weekday::Tue => "workingHoursTuesday",
            chrono::Weekday::Wed => "workingHoursWednesday",
            chrono::Weekday::Thu => "workingHoursThursday",
            chrono::Weekday::Fri => "workingHoursFriday",
            chrono::Weekday::Sat => "workingHoursSaturday",
            chrono::Weekday::Sun => "workingHoursSunday",
        };
        let snapshot = RuntimeSettings::from_settings(&json!({
            weekday_key: { "enabled": false, "ranges": [] }
        }));

        assert!(!snapshot.in_working_hours);
        assert!(snapshot.outside_working_hours());
    }

    #[test]
    fn clamps_a_nonsensical_frequency_to_at_least_one_second() {
        let snapshot = RuntimeSettings::from_settings(&json!({ "breakFrequencySeconds": 0 }));

        assert_eq!(snapshot.break_frequency_seconds, 1);
    }

    #[test]
    fn reads_the_time_since_last_break_tray_mode() {
        let snapshot = RuntimeSettings::from_settings(&json!({
            "trayTextEnabled": true,
            "trayTextMode": "TIME_SINCE_LAST_BREAK"
        }));

        assert!(snapshot.tray_text_enabled);
        assert_eq!(snapshot.tray_text_mode, TrayTextMode::TimeSinceLastBreak);
    }
}
