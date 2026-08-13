//! macOS Dock (activation policy) alignment with Settings visibility.
//! No-ops on other platforms.

use tauri::{AppHandle, Manager, Runtime};

pub(crate) fn settings_is_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window("settings")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

/// Last policy applied through [`sync_macos_dock`]. Startup leaves the app in
/// Accessory mode (`lib.rs` setup), matching the initial `false`.
#[cfg(target_os = "macos")]
static DOCK_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn sync_macos_dock<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        // `setActivationPolicy` re-registers the app with the window server
        // even when the policy is unchanged, which visibly flickers windows
        // that are on screen or mid-animation (e.g. a window restoring from
        // the Dock). Only apply real transitions.
        if DOCK_VISIBLE.swap(visible, std::sync::atomic::Ordering::AcqRel) == visible {
            return;
        }
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
