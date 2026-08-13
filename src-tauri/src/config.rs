use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

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
        let mut config = read_config(&path)?;
        config.normalize();
        save(&path, &config)?;
        return Ok((config, path));
    }

    for legacy_path in legacy_paths() {
        if !legacy_path.exists() {
            continue;
        }
        match read_config(&legacy_path) {
            Ok(mut config) => {
                config.normalize();
                config.migration_version = Some(1);
                config.migrated_from = Some(legacy_path.display().to_string());
                save(&path, &config)?;
                backup_legacy(&legacy_path)?;
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

pub fn save(path: &Path, config: &StoredConfig) -> Result<()> {
    let parent = path.parent().context("configuration path has no parent")?;
    fs::create_dir_all(parent).context("create config directory")?;
    let data = serde_json::to_vec_pretty(config).context("serialize configuration")?;
    let temporary = path.with_extension("json.tmp");
    write_temporary_config(&temporary, &data)?;
    replace_config_file(&temporary, path)?;
    Ok(())
}

fn write_temporary_config(path: &Path, data: &[u8]) -> Result<()> {
    let mut file = File::create(path).context("create temporary configuration")?;
    file.write_all(data)
        .context("write temporary configuration")?;
    file.sync_all().context("sync temporary configuration")?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_config_file(temporary: &Path, target: &Path) -> Result<()> {
    fs::rename(temporary, target).context("replace configuration atomically")
}

#[cfg(target_os = "windows")]
fn replace_config_file(temporary: &Path, target: &Path) -> Result<()> {
    if !target.exists() {
        return fs::rename(temporary, target).context("create configuration");
    }

    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let target = wide(target);
    let temporary = wide(temporary);
    // ReplaceFileW atomically swaps the new file into place when the destination already
    // exists. `rename` alone has inconsistent replacement semantics across Windows versions.
    let result = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            temporary.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error()).context("replace configuration atomically");
    }
    Ok(())
}

fn read_config(path: &Path) -> Result<StoredConfig> {
    let source = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&source).with_context(|| format!("parse {}", path.display()))
}

fn backup_legacy(path: &PathBuf) -> Result<()> {
    let stamp = Local::now().format("%Y%m%d%H%M%S");
    let backup = path.with_extension(format!("json.{stamp}.bak"));
    match fs::copy(path, backup) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("backup legacy configuration"),
    }
}

fn legacy_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let home = env::var_os("HOME").map(PathBuf::from);

    #[cfg(target_os = "macos")]
    if let Some(home) = &home {
        paths.push(home.join("Library/Application Support/neko/neko-config.json"));
        paths.push(home.join("Library/Application Support/Neko/neko-config.json"));
        paths.push(home.join("Library/Application Support/com.neko.app/neko-config.json"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = env::var_os("APPDATA") {
            paths.push(PathBuf::from(&app_data).join("neko/neko-config.json"));
            paths.push(PathBuf::from(app_data).join("Neko/neko-config.json"));
        }
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            paths.push(PathBuf::from(local_app_data).join("neko/neko-config.json"));
        }
        if let Some(program_data) = env::var_os("PROGRAMDATA") {
            paths.push(PathBuf::from(program_data).join("com.neko.app/neko-config.json"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
            paths.push(PathBuf::from(config_home).join("neko/neko-config.json"));
        }
        if let Some(home) = home {
            paths.push(home.join(".config/neko/neko-config.json"));
            paths.push(home.join(".config/Neko/neko-config.json"));
            paths.push(home.join(".config/com.neko.app/neko-config.json"));
        }
    }

    paths
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
