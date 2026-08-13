//! Tray icon, tray menu, and macOS menu-bar title.
//!
//! The tooltip (and macOS title) refresh every scheduler tick; the menu itself
//! is only rebuilt when its rendered content changes, so an open menu is not
//! disturbed every second. Menu countdowns therefore use minute precision.

use chrono::TimeZone;
use parking_lot::Mutex;
use tauri::{
    AppHandle, Manager, Runtime,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};

#[cfg(target_os = "macos")]
use crate::config::TrayTextMode;
use crate::{
    config::RuntimeSettings,
    core::i18n::{self, Locale, Text},
    platform::show_settings,
    scheduler::{self, RuntimeStatus, state::AppState},
};

const TRAY_ID: &str = "neko-tray";

/// Rendered content of the menu currently installed on the tray.
static MENU_SIGNATURE: Mutex<String> = Mutex::new(String::new());
/// Values last written to the tray. The tick refreshes once a second, but the
/// rendered text usually does not change, and every write crosses into the
/// platform's menu-bar API.
static LAST_TOOLTIP: Mutex<String> = Mutex::new(String::new());
#[cfg(target_os = "macos")]
static LAST_TITLE: Mutex<String> = Mutex::new(String::new());

pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> Result<TrayIcon<R>, String> {
    let settings = app.state::<AppState>().runtime_settings();
    let snapshot = TraySnapshot::collect(app, settings);
    let menu = build_tray_menu(app, &snapshot)?;
    *MENU_SIGNATURE.lock() = snapshot.menu_signature();
    *LAST_TOOLTIP.lock() = snapshot.tooltip();
    #[cfg(target_os = "macos")]
    {
        *LAST_TITLE.lock() = snapshot.title();
    }

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(snapshot.tooltip())
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
        });
    #[cfg(target_os = "macos")]
    {
        // Menu-bar icons must be template images so they adapt to the
        // light/dark menu bar; the colored app icon is wrong there.
        builder = builder
            .icon(macos_template_icon()?)
            .icon_as_template(true)
            .title(snapshot.title());
    }
    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.icon(
            app.default_window_icon()
                .ok_or("Neko tray icon is missing")?
                .clone(),
        );
    }
    builder
        .build(app)
        .map_err(|error| format!("create tray: {error}"))
}

/// Refreshes from a freshly projected snapshot. For cold paths (break
/// lifecycle, settings changes) where no projection is at hand.
pub fn refresh_tray<R: Runtime>(app: &AppHandle<R>) {
    let settings = app.state::<AppState>().runtime_settings();
    refresh_tray_with(app, settings);
}

/// Refreshes reusing the caller's projection — used by the 1 Hz tick.
pub(crate) fn refresh_tray_with<R: Runtime>(app: &AppHandle<R>, settings: RuntimeSettings) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let snapshot = TraySnapshot::collect(app, settings);

    let tooltip = snapshot.tooltip();
    {
        let mut last = LAST_TOOLTIP.lock();
        if *last != tooltip && tray.set_tooltip(Some(&tooltip)).is_ok() {
            *last = tooltip;
        }
    }
    #[cfg(target_os = "macos")]
    {
        let title = snapshot.title();
        let mut last = LAST_TITLE.lock();
        if *last != title && tray.set_title(Some(&title)).is_ok() {
            *last = title;
        }
    }

    let signature = snapshot.menu_signature();
    if *MENU_SIGNATURE.lock() == signature {
        return;
    }
    match build_tray_menu(app, &snapshot) {
        Ok(menu) => {
            if tray.set_menu(Some(menu)).is_ok() {
                *MENU_SIGNATURE.lock() = signature;
            }
        }
        Err(error) => tracing::warn!("could not rebuild tray menu: {error}"),
    }
}

#[cfg(target_os = "macos")]
fn macos_template_icon() -> Result<tauri::image::Image<'static>, String> {
    tauri::image::Image::from_bytes(include_bytes!("../../icons/tray-template.png"))
        .map_err(|error| format!("decode tray template icon: {error}"))
}

struct TraySnapshot {
    locale: Locale,
    breaks_enabled: bool,
    start_enabled: bool,
    /// Second precision, for the tooltip.
    fine_status: String,
    /// Minute precision, for the status line inside the menu.
    coarse_status: String,
    #[cfg(target_os = "macos")]
    macos_title: String,
}

impl TraySnapshot {
    fn collect<R: Runtime>(app: &AppHandle<R>, settings: RuntimeSettings) -> Self {
        let status = scheduler::runtime_status_with(app, settings);
        let disable_end_time = app.state::<AppState>().config.lock().disable_end_time;
        let locale = settings.locale;
        Self {
            locale,
            breaks_enabled: status.breaks_enabled,
            start_enabled: status.breaks_enabled && !status.having_break,
            fine_status: status_line(locale, &status, disable_end_time, false),
            coarse_status: status_line(locale, &status, disable_end_time, true),
            #[cfg(target_os = "macos")]
            macos_title: macos_title(app, &status, settings),
        }
    }

    fn tooltip(&self) -> String {
        format!("Neko · {}", self.fine_status)
    }

    #[cfg(target_os = "macos")]
    fn title(&self) -> String {
        self.macos_title.clone()
    }

    fn menu_signature(&self) -> String {
        format!(
            "{:?}|{}|{}|{}",
            self.locale, self.breaks_enabled, self.start_enabled, self.coarse_status
        )
    }
}

fn status_line(
    locale: Locale,
    status: &RuntimeStatus,
    disable_end_time: Option<i64>,
    coarse: bool,
) -> String {
    if !status.breaks_enabled {
        if let Some(end) = disable_end_time {
            let remaining = (end - chrono::Local::now().timestamp_millis()).max(0) / 1_000;
            return i18n::tray_disabled_left(locale, &format_seconds(remaining, coarse));
        }
        return i18n::text(locale, Text::TrayDisabled).to_owned();
    }
    if status.having_break {
        return i18n::text(locale, Text::TrayOnBreak).to_owned();
    }
    if status.outside_working_hours {
        return i18n::text(locale, Text::TrayOutsideHours).to_owned();
    }
    if status.idle {
        return i18n::text(locale, Text::TrayIdle).to_owned();
    }
    status.seconds_to_next_break.map_or_else(
        || i18n::text(locale, Text::TrayScheduling).to_owned(),
        |seconds| i18n::tray_next_in(locale, &format_seconds(seconds, coarse)),
    )
}

#[cfg(target_os = "macos")]
fn macos_title<R: Runtime>(
    app: &AppHandle<R>,
    status: &RuntimeStatus,
    settings: RuntimeSettings,
) -> String {
    if !settings.tray_text_enabled {
        return String::new();
    }
    if !status.breaks_enabled || status.having_break || status.idle || status.outside_working_hours
    {
        return String::new();
    }
    if settings.tray_text_mode == TrayTextMode::TimeSinceLastBreak {
        return scheduler::time_since_last_break_seconds(app)
            .map(format_duration)
            .unwrap_or_default();
    }
    status
        .seconds_to_next_break
        .map(format_duration)
        .unwrap_or_default()
}

fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &TraySnapshot,
) -> Result<Menu<R>, String> {
    let locale = snapshot.locale;
    let item = |id: &str, label: &str, enabled: bool| {
        MenuItem::with_id(app, id, label, enabled, None::<&str>).map_err(|error| error.to_string())
    };
    let separator = || PredefinedMenuItem::separator(app).map_err(|error| error.to_string());

    let status_item = item("status", &snapshot.coarse_status, false)?;
    let start = item(
        "start-now",
        i18n::text(locale, Text::TrayStartNow),
        snapshot.start_enabled,
    )?;
    let settings_item = item("settings", i18n::text(locale, Text::TraySettings), true)?;
    let about = item("about", &i18n::tray_about(locale, "Neko"), true)?;
    let quit = item("quit", i18n::text(locale, Text::TrayQuit), true)?;

    let menu = Menu::new(app).map_err(|error| error.to_string())?;
    menu.append(&status_item)
        .map_err(|error| error.to_string())?;
    menu.append(&separator()?)
        .map_err(|error| error.to_string())?;
    menu.append(&start).map_err(|error| error.to_string())?;
    menu.append(&separator()?)
        .map_err(|error| error.to_string())?;
    if snapshot.breaks_enabled {
        let disable = Submenu::with_items(
            app,
            i18n::text(locale, Text::TrayDisable),
            true,
            &[
                &item("disable-30", i18n::text(locale, Text::TrayDisable30m), true)?,
                &item("disable-60", i18n::text(locale, Text::TrayDisable1h), true)?,
                &item("disable-120", i18n::text(locale, Text::TrayDisable2h), true)?,
                &item("disable-240", i18n::text(locale, Text::TrayDisable4h), true)?,
                &item(
                    "disable-eod",
                    i18n::text(locale, Text::TrayDisableEod),
                    true,
                )?,
                &item(
                    "disable-forever",
                    i18n::text(locale, Text::TrayDisableIndefinite),
                    true,
                )?,
            ],
        )
        .map_err(|error| error.to_string())?;
        menu.append(&disable).map_err(|error| error.to_string())?;
    } else {
        menu.append(&item("enable", i18n::text(locale, Text::TrayEnable), true)?)
            .map_err(|error| error.to_string())?;
    }
    menu.append(&separator()?)
        .map_err(|error| error.to_string())?;
    menu.append(&settings_item)
        .map_err(|error| error.to_string())?;
    menu.append(&about).map_err(|error| error.to_string())?;
    menu.append(&separator()?)
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
        "settings" | "about" => show_settings(app),
        "quit" => app.exit(0),
        _ => {}
    }
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

fn format_seconds(seconds: i64, coarse: bool) -> String {
    if coarse {
        format_duration_coarse(seconds)
    } else {
        format_duration(seconds)
    }
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

/// Minute precision keeps the in-menu status stable so the menu is rebuilt at
/// most once a minute while a countdown runs.
fn format_duration_coarse(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "<1m".to_owned()
    }
}
