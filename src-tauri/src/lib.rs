use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::{config::load_or_migrate, core::audio::AudioPlayer, scheduler::state::AppState};

pub mod cmd;
pub mod config;
pub mod core;
pub mod monitors;
pub mod platform;
pub mod scheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // single-instance must be first so a second instance exits before
        // initializing anything else (per plugin docs).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            platform::show_settings(app);
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("neko".into()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .level_for("tao", log::LevelFilter::Warn)
                .level_for("wry", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Neko starting");
            let (config, config_path) = load_or_migrate(app.handle())?;
            let audio = AudioPlayer::new()
                .map_err(|error| {
                    tracing::warn!("audio device unavailable, sound playback disabled: {error}");
                    error
                })
                .ok();
            app.manage(AppState::new(config, config_path, audio));
            if !cfg!(debug_assertions) {
                let state = app.state::<AppState>();
                let enabled = state
                    .config
                    .lock()
                    .settings
                    .get("autoLaunch")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let result = if enabled {
                    app.autolaunch().enable()
                } else {
                    app.autolaunch().disable()
                };
                if let Err(error) = result {
                    tracing::warn!("could not synchronize login launch setting: {error}");
                }
            }
            platform::init_settings_lifecycle(app.handle());
            let _tray = platform::init_tray(app.handle())?;
            scheduler::init(app.handle());
            platform::show_settings(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::settings_get,
            cmd::settings_set,
            cmd::break_allow_postpone_get,
            cmd::break_postpone,
            cmd::break_start,
            cmd::break_end,
            cmd::break_active_end_time_get,
            cmd::break_window_ready,
            cmd::break_length_get,
            cmd::break_window_resize,
            cmd::break_tracking_complete,
            cmd::time_since_last_break_get,
            cmd::break_started_from_tray_get,
            cmd::runtime_status_get,
            cmd::app_version_get,
            cmd::sound_start_play,
            cmd::sound_end_play,
            cmd::break_preview,
            cmd::autolaunch_onboarding_seen_get,
            cmd::autolaunch_onboarding_dismiss
        ])
        .build(tauri::generate_context!())
        .expect("error while building Neko Tauri application");

    app.run(|_app_handle, _event| {
        // Dock icon click while the Settings window is minimized/hidden must
        // bring Settings back (parity with Electron's `activate` handler).
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            platform::show_settings(_app_handle);
        }
    });
}
