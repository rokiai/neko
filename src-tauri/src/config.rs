//! Stored configuration: schema defaults, normalization, and v1–v5 migration.
//! Persistence lives in `config::persist`, the typed DTO contract in
//! `config::schema`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

mod persist;
mod schema;

pub use persist::save;
pub use schema::validate_settings;

pub const SETTINGS_VERSION: u32 = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfig {
    #[serde(default = "default_settings_version", alias = "settingsVersion")]
    pub settings_version: u32,
    #[serde(default = "default_settings")]
    pub settings: Value,
    #[serde(default)]
    pub disable_end_time: Option<i64>,
    #[serde(default = "default_daily_stats")]
    pub daily_stats: Value,
    #[serde(default)]
    pub auto_launch_onboarding_seen: bool,
    #[serde(default)]
    pub migration_version: Option<u32>,
    #[serde(default)]
    pub migrated_from: Option<String>,
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,
            settings: default_settings(),
            disable_end_time: None,
            daily_stats: default_daily_stats(),
            auto_launch_onboarding_seen: false,
            migration_version: None,
            migrated_from: None,
        }
    }
}

fn default_settings_version() -> u32 {
    1
}

fn default_working_hours() -> Value {
    json!({ "enabled": true, "ranges": [{ "fromMinutes": 0, "toMinutes": 1439 }] })
}

pub fn default_settings() -> Value {
    let hours = default_working_hours();
    json!({
        "locale": "system",
        "autoLaunch": true,
        "breaksEnabled": true,
        "trayTextEnabled": false,
        "trayTextMode": "TIME_TO_NEXT_BREAK",
        "notificationType": "POPUP",
        "breakFrequencySeconds": 1680,
        "breakLengthSeconds": 120,
        "postponeLengthSeconds": 180,
        "postponeLimit": 0,
        "workingHoursEnabled": true,
        "workingHoursMonday": hours,
        "workingHoursTuesday": default_working_hours(),
        "workingHoursWednesday": default_working_hours(),
        "workingHoursThursday": default_working_hours(),
        "workingHoursFriday": default_working_hours(),
        "workingHoursSaturday": default_working_hours(),
        "workingHoursSunday": default_working_hours(),
        "idleResetEnabled": false,
        "idleResetLengthSeconds": 300,
        "idleResetNotification": false,
        "soundType": "NONE",
        "breakSoundVolume": 1.0,
        "breakTitle": "",
        "breakMessage": "",
        "backgroundColor": "#3F9F7E",
        "textColor": "#F7F3EE",
        "showBackdrop": true,
        "backdropOpacity": 0.72,
        "breakPopupStyle": "CARD",
        "endBreakEnabled": true,
        "skipBreakEnabled": false,
        "postponeBreakEnabled": true,
        "immediatelyStartBreaks": true
    })
}

pub fn default_daily_stats() -> Value {
    json!({
        "dayKey": Local::now().format("%Y-%m-%d").to_string(),
        "workSeconds": 0,
        "restSeconds": 0,
        "completedBreaks": 0
    })
}

pub fn normalize_settings(settings: &Value) -> Value {
    let mut normalized = default_settings();
    merge_json(&mut normalized, settings);

    if let Some(object) = normalized.as_object_mut() {
        object.insert("breakPopupStyle".into(), Value::String("CARD".into()));
        object.remove("breakVideoSource");
        object.remove("breakVideoPath");
        object.remove("breakVideoMuted");
    }

    normalized
}

fn merge_json(base: &mut Value, incoming: &Value) {
    let (Some(base_object), Some(incoming_object)) = (base.as_object_mut(), incoming.as_object())
    else {
        *base = incoming.clone();
        return;
    };

    for (key, incoming_value) in incoming_object {
        match base_object.get_mut(key) {
            Some(base_value) if base_value.is_object() && incoming_value.is_object() => {
                merge_json(base_value, incoming_value);
            }
            _ => {
                base_object.insert(key.clone(), incoming_value.clone());
            }
        }
    }
}

impl StoredConfig {
    pub fn normalize(&mut self) {
        let version = self.settings_version;
        self.settings = normalize_settings(&self.settings);
        migrate_settings(&mut self.settings, version);
        self.settings_version = SETTINGS_VERSION;
        if !self.daily_stats.is_object() {
            self.daily_stats = default_daily_stats();
        }
        // Loaded configs are not rejected (per-key fallbacks keep the app
        // usable), but deviations must be visible instead of silent.
        if let Err(error) = validate_settings(&self.settings) {
            tracing::warn!("stored settings deviate from the schema: {error}");
        }
    }
}

fn migrate_settings(settings: &mut Value, version: u32) {
    let Some(object) = settings.as_object_mut() else {
        return;
    };

    if version < 2 {
        object.insert("soundType".to_owned(), Value::String("NONE".to_owned()));
    }
    if version < 3 {
        if object.get("breakTitle").and_then(Value::as_str) == Some("Time for a break.") {
            object.insert("breakTitle".to_owned(), Value::String(String::new()));
        }
        if object.get("breakMessage").and_then(Value::as_str)
            == Some("Rest your eyes.\nStretch your legs.\nBreathe. Relax.")
        {
            object.insert("breakMessage".to_owned(), Value::String(String::new()));
        }
    }
    if version < 4 {
        object.insert("trayTextEnabled".to_owned(), Value::Bool(false));
    }
    // v5 removes VIDEO permanently. `normalize_settings` already removes its fields;
    // write CARD here to make the version transition explicit and idempotent.
    if version < 5 {
        object.insert(
            "breakPopupStyle".to_owned(),
            Value::String("CARD".to_owned()),
        );
    }
}

pub fn config_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    let directory = app
        .path()
        .app_config_dir()
        .context("resolve app config directory")?;
    Ok(directory.join("neko-config.json"))
}

pub fn load_or_migrate<R: Runtime>(app: &AppHandle<R>) -> Result<(StoredConfig, PathBuf)> {
    let path = config_path(app)?;
    if path.exists() {
        let mut config = persist::read_config(&path)?;
        config.normalize();
        save(&path, &config)?;
        return Ok((config, path));
    }

    for legacy_path in persist::legacy_paths() {
        if !legacy_path.exists() {
            continue;
        }
        match persist::read_config(&legacy_path) {
            Ok(mut config) => {
                config.normalize();
                config.migration_version = Some(1);
                config.migrated_from = Some(legacy_path.display().to_string());
                save(&path, &config)?;
                persist::backup_legacy(&legacy_path)?;
                tracing::info!(source = %legacy_path.display(), target = %path.display(), "migrated Electron configuration");
                return Ok((config, path));
            }
            Err(error) => {
                tracing::warn!(path = %legacy_path.display(), "could not parse legacy configuration: {error}");
            }
        }
    }

    let config = StoredConfig::default();
    save(&path, &config)?;
    Ok((config, path))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{SETTINGS_VERSION, normalize_settings};

    #[test]
    fn normalizes_legacy_video_preferences_to_card() {
        let settings = normalize_settings(&json!({
            "breakPopupStyle": "VIDEO",
            "breakVideoSource": "CUSTOM",
            "breakVideoPath": "/tmp/calm.mp4",
            "breakVideoMuted": false,
            "breakFrequencySeconds": 600
        }));

        assert_eq!(settings["breakPopupStyle"], "CARD");
        assert!(settings.get("breakVideoSource").is_none());
        assert!(settings.get("breakVideoPath").is_none());
        assert!(settings.get("breakVideoMuted").is_none());
        assert_eq!(settings["breakFrequencySeconds"], 600);
    }

    #[test]
    fn normalization_keeps_new_schema_defaults() {
        let settings = normalize_settings(&json!({ "soundType": "PING" }));

        assert_eq!(settings["soundType"], "PING");
        assert_eq!(settings["breakPopupStyle"], "CARD");
        assert_eq!(settings["breakLengthSeconds"], 120);
        assert_eq!(SETTINGS_VERSION, 5);
    }

    #[test]
    fn carries_out_the_electron_v1_to_v5_migrations() {
        let mut config: super::StoredConfig = serde_json::from_value(json!({
            "settingsVersion": 1,
            "settings": {
                "soundType": "GONG",
                "trayTextEnabled": true,
                "breakTitle": "Time for a break.",
                "breakMessage": "Rest your eyes.\nStretch your legs.\nBreathe. Relax.",
                "breakPopupStyle": "VIDEO"
            }
        }))
        .expect("legacy settings deserialize");

        config.normalize();

        assert_eq!(config.settings_version, SETTINGS_VERSION);
        assert_eq!(config.settings["soundType"], "NONE");
        assert_eq!(config.settings["trayTextEnabled"], false);
        assert_eq!(config.settings["breakTitle"], "");
        assert_eq!(config.settings["breakMessage"], "");
        assert_eq!(config.settings["breakPopupStyle"], "CARD");
    }
}
