use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::{config::RuntimeSettings, scheduler, scheduler::state::AppState};

mod dock;
#[cfg(target_os = "macos")]
mod macos_spaces;
mod tray;
#[cfg(windows)]
mod windows_desktops;

use dock::{settings_is_visible, sync_macos_dock};
pub(crate) use tray::refresh_tray_with;
pub use tray::{init_tray, refresh_tray};

const BREAK_LABEL_PREFIX: &str = "break-";
const BREAK_CARD_WIDTH: f64 = 520.0;
const BREAK_CARD_HEIGHT: f64 = 320.0;

pub fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        if window.is_minimized().unwrap_or(false) {
            // `show()` would makeKeyAndOrderFront the miniaturized window: it
            // pops in at full size and the deminiaturize animation then
            // re-presents it, which reads as a flash. Deminiaturizing alone
            // restores and orders the window in.
            let _ = window.unminimize();
        } else {
            let _ = window.show();
        }
        let _ = window.set_focus();
        sync_macos_dock(app, true);
        // Nothing was pushed while the window was hidden, so seed it now
        // instead of leaving stale numbers on screen until the next tick.
        push_runtime_status(app, app.state::<AppState>().runtime_settings());
    }
}

/// Pushes the runtime status to the Settings window while it is on screen.
///
/// Closing Settings hides the window instead of destroying it, so its webview
/// and React tree stay alive. Gating the push here — rather than letting the
/// frontend poll — means a hidden window costs nothing: no IPC round trip, no
/// status rebuild, no re-render.
pub(crate) fn push_runtime_status<R: Runtime>(app: &AppHandle<R>, settings: RuntimeSettings) {
    if !settings_is_visible(app) {
        return;
    }
    let Some(window) = app.get_webview_window("settings") else {
        return;
    };
    let status = scheduler::runtime_status_with(app, settings);
    if let Err(error) = window.emit("neko://runtime/status", status) {
        tracing::warn!("could not push runtime status: {error}");
    }
}

pub fn hide_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.hide();
    }
    sync_macos_dock(app, false);
}

pub fn init_settings_lifecycle<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("settings") else {
        return;
    };
    let app_handle = app.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide_settings(&app_handle);
        }
        WindowEvent::Destroyed => sync_macos_dock(&app_handle, false),
        _ => {}
    });
}

pub fn create_break_windows<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if has_break_windows(app) {
        return Ok(());
    }

    let state = app.state::<AppState>();
    let persisted_settings = state.config.lock().settings.clone();
    let settings = state
        .scheduler
        .lock()
        .active_break_settings
        .clone()
        .unwrap_or(persisted_settings);
    let fullscreen = settings
        .get("showBackdrop")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let monitors = app
        .available_monitors()
        .map_err(|error| format!("list displays: {error}"))?;
    if monitors.is_empty() {
        return Err("no displays available for Break".to_owned());
    }

    let nonce = chrono::Local::now().timestamp_millis();
    let mut labels = Vec::with_capacity(monitors.len());
    for (index, monitor) in monitors.iter().enumerate() {
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let position = monitor.position();
        let monitor_width = f64::from(size.width) / scale;
        let monitor_height = f64::from(size.height) / scale;
        let monitor_x = f64::from(position.x) / scale;
        let monitor_y = f64::from(position.y) / scale;
        let (width, height, x, y) = if fullscreen {
            (monitor_width, monitor_height, monitor_x, monitor_y)
        } else {
            (
                BREAK_CARD_WIDTH,
                BREAK_CARD_HEIGHT,
                monitor_x + (monitor_width - BREAK_CARD_WIDTH) / 2.0,
                monitor_y + (monitor_height - BREAK_CARD_HEIGHT) / 2.0,
            )
        };
        let label = format!("{BREAK_LABEL_PREFIX}{nonce}-{index}");
        let url = format!("/index.html?page=break&windowId={index}");
        let window = match WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("Neko")
            .inner_size(width, height)
            .position(x, y)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            // Only Linux honors this flag: macOS joins Spaces via an NSPanel
            // parent (macos_spaces) and Windows follows virtual desktops via
            // IVirtualDesktopManager (windows_desktops).
            .visible_on_all_workspaces(cfg!(not(any(target_os = "macos", windows))))
            .skip_taskbar(true)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .focused(false)
            .visible(false)
            .build()
        {
            Ok(window) => window,
            Err(error) => {
                destroy_windows(app, &labels);
                #[cfg(target_os = "macos")]
                {
                    let _ = app.run_on_main_thread(macos_spaces::release_all_space_anchors);
                }
                return Err(format!("create Break window for display {index}: {error}"));
            }
        };

        if let Err(error) = window.set_focusable(false) {
            tracing::warn!(label = %label, "could not disable Break window focus: {error}");
        }
        configure_break_window(&window, &label);
        let app_handle = app.clone();
        let window_label = label.clone();
        window.on_window_event(move |event| {
            if matches!(event, WindowEvent::Destroyed) {
                scheduler::break_window_destroyed(&app_handle, &window_label);
            }
        });
        labels.push(label);
    }

    state.scheduler.lock().break_window_labels = labels;
    // Keep Dock/activation policy aligned with Settings visibility. Forcing
    // Accessory during preview (Settings still open) steals focus and can jump
    // the user to another Space/display.
    sync_macos_dock(app, settings_is_visible(app));
    Ok(())
}

pub fn show_break_window<R: Runtime>(window: &tauri::WebviewWindow<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // orderFrontRegardless only — Tauri show() → makeKeyAndOrderFront flashes
        // and can jump Spaces. Without Transient, inactive show stays visible.
        macos_spaces::show_break_window_inactive(window)?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Break windows are built with focused(false), so tao shows them with
        // SW_SHOWNOACTIVATE on Windows — no extra silent-show handling needed.
        window
            .show()
            .map_err(|error| format!("show Break window: {error}"))?;
        window
            .set_always_on_top(true)
            .map_err(|error| format!("keep Break window on top: {error}"))?;
        // Best effort: the overlay is already on screen, so a desktop-follow
        // failure must not abort the Break window handshake.
        #[cfg(windows)]
        if let Err(error) = windows_desktops::follow_active_virtual_desktop(window) {
            tracing::warn!(label = %window.label(), "could not move Break window to the active desktop: {error}");
        }
    }

    mark_break_window_ready(window.app_handle(), window.label());

    Ok(())
}

fn mark_break_window_ready<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let state = app.state::<AppState>();
    let mut scheduler = state.scheduler.lock();
    if !scheduler
        .break_window_ready_labels
        .iter()
        .any(|item| item == label)
    {
        scheduler.break_window_ready_labels.push(label.to_owned());
    }
}

fn configure_break_window<R: Runtime>(window: &tauri::WebviewWindow<R>, label: &str) {
    #[cfg(not(target_os = "macos"))]
    if let Err(error) = window.set_visible_on_all_workspaces(true) {
        tracing::warn!(label = %label, "could not keep Break window on all workspaces: {error}");
    }
    #[cfg(target_os = "macos")]
    if let Err(error) = configure_macos_break_window(window) {
        tracing::warn!(label = %label, "could not configure Break window for macOS Spaces: {error}");
    }

    #[cfg(not(target_os = "macos"))]
    if let Err(error) = window.set_always_on_top(true) {
        tracing::warn!(label = %label, "could not keep Break window on top: {error}");
    }
}

#[cfg(target_os = "macos")]
fn configure_macos_break_window<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<(), String> {
    macos_spaces::prepare_break_window(window)
}

pub fn maintain_break_windows<R: Runtime>(app: &AppHandle<R>) {
    let labels = app
        .state::<AppState>()
        .scheduler
        .lock()
        .break_window_ready_labels
        .clone();
    for label in labels {
        let Some(window) = app.get_webview_window(&label) else {
            continue;
        };
        if let Err(error) = keep_break_window_on_active_workspace(&window) {
            tracing::warn!(label = %label, "could not restore Break window after workspace change: {error}");
        }
    }
}

#[cfg(target_os = "macos")]
fn keep_break_window_on_active_workspace<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<(), String> {
    macos_spaces::maintain_break_window(window)
}

#[cfg(windows)]
fn keep_break_window_on_active_workspace<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<(), String> {
    windows_desktops::follow_active_virtual_desktop(window)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn keep_break_window_on_active_workspace<R: Runtime>(
    _window: &tauri::WebviewWindow<R>,
) -> Result<(), String> {
    Ok(())
}

pub fn require_primary_break_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window
        .label()
        .rsplit_once('-')
        .is_some_and(|(_, index)| index == "0")
    {
        Ok(())
    } else {
        Err("only the primary Break window can control the shared timer".to_owned())
    }
}

pub fn close_break_windows<R: Runtime>(app: &AppHandle<R>) {
    let labels = {
        let state = app.state::<AppState>();
        let mut scheduler = state.scheduler.lock();
        scheduler.break_window_ready_labels.clear();
        std::mem::take(&mut scheduler.break_window_labels)
    };
    destroy_windows(app, &labels);
    #[cfg(target_os = "macos")]
    {
        let _ = app.run_on_main_thread(macos_spaces::release_all_space_anchors);
    }
    sync_macos_dock(app, settings_is_visible(app));
}

fn destroy_windows<R: Runtime>(app: &AppHandle<R>, labels: &[String]) {
    for label in labels {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.destroy();
        }
    }
}

pub fn resize_break_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let persisted_settings = state.config.lock().settings.clone();
    let settings = state
        .scheduler
        .lock()
        .active_break_settings
        .clone()
        .unwrap_or(persisted_settings);
    let fullscreen = settings
        .get("showBackdrop")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| format!("Break window {label} is not available"))?;
    let monitor = window
        .current_monitor()
        .map_err(|error| format!("resolve Break display: {error}"))?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("no display available for Break")?;
    let scale = monitor.scale_factor();
    let size = monitor.size();
    let position = monitor.position();
    let monitor_width = f64::from(size.width) / scale;
    let monitor_height = f64::from(size.height) / scale;
    let monitor_x = f64::from(position.x) / scale;
    let monitor_y = f64::from(position.y) / scale;
    let (width, height, x, y) = if fullscreen {
        (monitor_width, monitor_height, monitor_x, monitor_y)
    } else {
        (
            BREAK_CARD_WIDTH,
            BREAK_CARD_HEIGHT,
            monitor_x + (monitor_width - BREAK_CARD_WIDTH) / 2.0,
            monitor_y + (monitor_height - BREAK_CARD_HEIGHT) / 2.0,
        )
    };
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|error| format!("resize Break: {error}"))?;
    window
        .set_position(tauri::LogicalPosition::new(x, y))
        .map_err(|error| format!("position Break: {error}"))
}

fn has_break_windows<R: Runtime>(app: &AppHandle<R>) -> bool {
    !app.state::<AppState>()
        .scheduler
        .lock()
        .break_window_labels
        .is_empty()
}
