use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

use crate::{config::load_or_migrate, core::audio::AudioPlayer, scheduler::state::AppState};

pub mod cmd;
pub mod config;
pub mod core;
pub mod monitors;
pub mod platform;
pub mod scheduler;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            platform::show_settings(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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
        .run(tauri::generate_context!())
        .expect("error while running Neko Tauri application");
}
