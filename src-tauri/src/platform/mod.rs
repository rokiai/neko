use chrono::TimeZone;
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};

use crate::{scheduler, scheduler::state::AppState};

#[cfg(target_os = "macos")]
mod macos_spaces;

const BREAK_LABEL_PREFIX: &str = "break-";
const BREAK_CARD_WIDTH: f64 = 520.0;
const BREAK_CARD_HEIGHT: f64 = 320.0;

pub fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        sync_macos_dock(app, true);
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
            // macOS Break overlays join Spaces via an NSPanel parent in
            // macos_spaces — Tauri's visible_on_all_workspaces alone is a no-op
            // for WKWebView NSWindow Space membership.
            .visible_on_all_workspaces(cfg!(not(target_os = "macos")))
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
        window
            .show()
            .map_err(|error| format!("show Break window: {error}"))?;
        window
            .set_always_on_top(true)
            .map_err(|error| format!("keep Break window on top: {error}"))?;
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

        #[cfg(target_os = "macos")]
        if let Err(error) = macos_spaces::maintain_break_window(&window) {
            tracing::warn!(label = %label, "could not restore Break window after Space change: {error}");
        }
    }
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

pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> Result<TrayIcon<R>, String> {
    let menu = build_tray_menu(app)?;
    let icon = app
        .default_window_icon()
        .ok_or("Neko tray icon is missing")?
        .clone();
    TrayIconBuilder::with_id("neko-tray")
        .menu(&menu)
        .icon(icon)
        .tooltip("Neko")
        .on_menu_event(|app, event| handle_tray_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| format!("create tray: {error}"))
}

pub fn refresh_tray<R: Runtime>(app: &AppHandle<R>) {
    let Ok(menu) = build_tray_menu(app) else {
        tracing::warn!("could not rebuild tray menu");
        return;
    };
    if let Some(tray) = app.tray_by_id("neko-tray") {
        let _ = tray.set_menu(Some(menu));
        let _ = tray.set_tooltip(Some(tray_status(app)));
        #[cfg(target_os = "macos")]
        let _ = tray.set_title(Some(tray_title(app)));
    }
}

fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, String> {
    let status_item = MenuItem::with_id(app, "status", tray_status(app), false, None::<&str>)
        .map_err(|error| error.to_string())?;
    let runtime_status = scheduler::runtime_status(app);
    let start = MenuItem::with_id(
        app,
        "start-now",
        "Start break now",
        runtime_status.breaks_enabled && !runtime_status.having_break,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let disable_30 = MenuItem::with_id(app, "disable-30", "For 30 minutes", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let disable_60 = MenuItem::with_id(app, "disable-60", "For 1 hour", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let disable_120 = MenuItem::with_id(app, "disable-120", "For 2 hours", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let disable_240 = MenuItem::with_id(app, "disable-240", "For 4 hours", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let disable_eod = MenuItem::with_id(app, "disable-eod", "Until end of day", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let disable_forever =
        MenuItem::with_id(app, "disable-forever", "Indefinitely", true, None::<&str>)
            .map_err(|error| error.to_string())?;
    let enable = MenuItem::with_id(app, "enable", "Enable breaks", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let separator_after_status =
        PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    let separator_after_start =
        PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    let separator_before_settings =
        PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    let separator_before_quit =
        PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    let state = app.state::<AppState>();
    let config = state.config.lock();
    let enabled = config
        .settings
        .get("breaksEnabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    drop(config);

    let menu = Menu::new(app).map_err(|error| error.to_string())?;
    menu.append(&status_item)
        .map_err(|error| error.to_string())?;
    menu.append(&separator_after_status)
        .map_err(|error| error.to_string())?;
    menu.append(&start).map_err(|error| error.to_string())?;
    menu.append(&separator_after_start)
        .map_err(|error| error.to_string())?;
    if enabled {
        let disabled = Submenu::with_items(
            app,
            "Disable",
            true,
            &[
                &disable_30,
                &disable_60,
                &disable_120,
                &disable_240,
                &disable_eod,
                &disable_forever,
            ],
        )
        .map_err(|error| error.to_string())?;
        menu.append(&disabled).map_err(|error| error.to_string())?;
    } else {
        menu.append(&enable).map_err(|error| error.to_string())?;
    }
    menu.append(&separator_before_settings)
        .map_err(|error| error.to_string())?;
    menu.append(&settings).map_err(|error| error.to_string())?;
    menu.append(&separator_before_quit)
        .map_err(|error| error.to_string())?;
    menu.append(&quit).map_err(|error| error.to_string())?;
    Ok(menu)
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "start-now" => scheduler::start_break_now(app),
        "disable-30" => scheduler::disable_breaks_for(app, Some(30 * 60 * 1_000)),
        "disable-60" => scheduler::disable_breaks_for(app, Some(60 * 60 * 1_000)),
        "disable-120" => scheduler::disable_breaks_for(app, Some(2 * 60 * 60 * 1_000)),
        "disable-240" => scheduler::disable_breaks_for(app, Some(4 * 60 * 60 * 1_000)),
        "disable-eod" => scheduler::disable_breaks_for(app, Some(milliseconds_until_end_of_day())),
        "disable-forever" => scheduler::disable_breaks_for(app, None),
        "enable" => scheduler::enable_breaks(app),
        "settings" => show_settings(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn has_break_windows<R: Runtime>(app: &AppHandle<R>) -> bool {
    !app.state::<AppState>()
        .scheduler
        .lock()
        .break_window_labels
        .is_empty()
}

fn settings_is_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window("settings")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

fn sync_macos_dock<R: Runtime>(app: &AppHandle<R>, visible: bool) {
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

fn milliseconds_until_end_of_day() -> i64 {
    let now = chrono::Local::now();
    let tomorrow = now
        .date_naive()
        .succ_opt()
        .expect("a successor local date exists");
    let midnight = tomorrow.and_hms_opt(0, 0, 0).expect("valid local midnight");
    chrono::Local
        .from_local_datetime(&midnight)
        .earliest()
        .map(|end| (end - now).num_milliseconds().max(0))
        .unwrap_or(0)
}

fn tray_status<R: Runtime>(app: &AppHandle<R>) -> String {
    let status = scheduler::runtime_status(app);
    if !status.breaks_enabled {
        return "Breaks disabled".to_owned();
    }
    if status.having_break {
        return "On a break".to_owned();
    }
    if status.outside_working_hours {
        return "Outside working hours".to_owned();
    }
    if status.idle {
        return "Idle - timer paused".to_owned();
    }
    status.seconds_to_next_break.map_or_else(
        || "Scheduling...".to_owned(),
        |seconds| format!("Next break in {}", format_duration(seconds)),
    )
}

#[cfg(target_os = "macos")]
fn tray_title<R: Runtime>(app: &AppHandle<R>) -> String {
    let state = app.state::<AppState>();
    let settings = state.config.lock().settings.clone();
    if !settings
        .get("trayTextEnabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return String::new();
    }
    let status = scheduler::runtime_status(app);
    if !status.breaks_enabled || status.having_break || status.idle || status.outside_working_hours
    {
        return String::new();
    }
    let mode = settings
        .get("trayTextMode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("TIME_TO_NEXT_BREAK");
    if mode == "TIME_SINCE_LAST_BREAK" {
        return scheduler::time_since_last_break_seconds(app)
            .map(format_duration)
            .unwrap_or_default();
    }
    status
        .seconds_to_next_break
        .map(format_duration)
        .unwrap_or_default()
}

fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}
