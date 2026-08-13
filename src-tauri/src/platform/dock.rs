//! macOS Dock (activation policy) alignment with Settings visibility.
//! No-ops on other platforms.

use tauri::{AppHandle, Manager, Runtime};

pub(crate) fn settings_is_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window("settings")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub(crate) fn sync_macos_dock<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}
