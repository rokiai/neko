//! macOS Spaces follow for Break overlays.
//!
//! Plain WKWebView `NSWindow`s do not really join Spaces. Attach each Break
//! window as a child of an invisible non-activating `NSPanel` that carries
//! `CanJoinAllSpaces | FullScreenAuxiliary`; children inherit that membership.
//!
//! Do **not** mark children `Transient` — AppKit hides transient windows when
//! the active Space changes, which looks like the Break overlay vanishing.
//!
//! Show with `orderFrontRegardless` (Electron `showInactive`). Avoid Tauri
//! `show()` → `makeKeyAndOrderFront`, which flashes and can jump displays.

use std::collections::HashMap;
use std::sync::LazyLock;

use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSPanel, NSScreenSaverWindowLevel, NSWindow, NSWindowAnimationBehavior,
    NSWindowCollectionBehavior, NSWindowOrderingMode, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use parking_lot::Mutex;
use tauri::{Runtime, WebviewWindow};

struct MainThreadPanel(Retained<NSPanel>);

// SAFETY: all reads/writes happen on the AppKit main thread via `with_webview`.
unsafe impl Send for MainThreadPanel {}
unsafe impl Sync for MainThreadPanel {}

static SPACE_ANCHORS: LazyLock<Mutex<HashMap<isize, MainThreadPanel>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn panel_collection_behavior() -> NSWindowCollectionBehavior {
    NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::FullScreenAuxiliary
        | NSWindowCollectionBehavior::CanJoinAllApplications
        | NSWindowCollectionBehavior::Stationary
        | NSWindowCollectionBehavior::IgnoresCycle
}

fn child_collection_behavior() -> NSWindowCollectionBehavior {
    // Space membership comes from the NSPanel parent.
    // Never set Transient: it hides the window when the active Space changes.
    NSWindowCollectionBehavior::FullScreenAuxiliary
        | NSWindowCollectionBehavior::CanJoinAllApplications
        | NSWindowCollectionBehavior::Stationary
        | NSWindowCollectionBehavior::IgnoresCycle
}

fn create_space_anchor(mtm: MainThreadMarker) -> MainThreadPanel {
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        NSPanel::alloc(mtm),
        NSRect::new(NSPoint::new(-10_000.0, -10_000.0), NSSize::new(1.0, 1.0)),
        NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
        NSBackingStoreType::Buffered,
        false,
    );
    unsafe {
        panel.setReleasedWhenClosed(false);
    }
    panel.setFloatingPanel(true);
    panel.setBecomesKeyOnlyIfNeeded(true);
    panel.setCollectionBehavior(panel_collection_behavior());
    panel.setLevel(NSScreenSaverWindowLevel);
    panel.setAlphaValue(0.0);
    panel.setIgnoresMouseEvents(true);
    panel.setHidesOnDeactivate(false);
    panel.setCanHide(false);
    panel.setAnimationBehavior(NSWindowAnimationBehavior::None);
    panel.orderFrontRegardless();
    MainThreadPanel(panel)
}

fn apply_child_chrome(native_window: &NSWindow) {
    native_window.setCollectionBehavior(child_collection_behavior());
    native_window.setLevel(NSScreenSaverWindowLevel);
    native_window.setHidesOnDeactivate(false);
    native_window.setCanHide(false);
    native_window.setAnimationBehavior(NSWindowAnimationBehavior::None);
}

fn attach_quietly(native_window: &NSWindow) {
    let window_number = native_window.windowNumber();
    if window_number == 0 {
        return;
    }

    apply_child_chrome(native_window);

    if native_window.parentWindow().is_some() {
        return;
    }

    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!("Break Space attach skipped: not on AppKit main thread");
        return;
    };

    let mut anchors = SPACE_ANCHORS.lock();
    let anchor = anchors
        .entry(window_number)
        .or_insert_with(|| create_space_anchor(mtm));

    anchor.0.setCollectionBehavior(panel_collection_behavior());
    anchor.0.setLevel(NSScreenSaverWindowLevel);

    unsafe {
        anchor
            .0
            .addChildWindow_ordered(native_window, NSWindowOrderingMode::Above);
    }
}

fn with_native_window<R: Runtime>(
    window: &WebviewWindow<R>,
    f: impl FnOnce(&NSWindow) + Send + 'static,
) -> Result<(), String> {
    window
        .with_webview(move |webview| unsafe {
            let native_window = webview.ns_window().cast::<NSWindow>();
            if native_window.is_null() {
                return;
            }
            f(&*native_window);
        })
        .map_err(|error| format!("native Break window: {error}"))
}

/// Attach and keep the window fully transparent until the UI is ready to reveal.
pub fn prepare_break_window<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    with_native_window(window, |native_window| {
        native_window.setAlphaValue(0.0);
        attach_quietly(native_window);
    })
}

/// Reveal without `makeKeyAndOrderFront` — one AppKit turn, no alpha round-trips.
pub fn show_break_window_inactive<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    with_native_window(window, |native_window| {
        attach_quietly(native_window);
        // Content is already painted in the hidden webview; reveal at full opacity
        // in the same turn as ordering front to avoid a transparent/opaque flash.
        native_window.setAlphaValue(1.0);
        native_window.orderFrontRegardless();
        if native_window.isKeyWindow() {
            native_window.resignKeyWindow();
        }
    })
}

pub fn maintain_break_window<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    with_native_window(window, |native_window| {
        apply_child_chrome(native_window);
        if native_window.parentWindow().is_some() {
            return;
        }
        attach_quietly(native_window);
        if native_window.isVisible() {
            native_window.orderFrontRegardless();
        }
    })
}

pub fn release_all_space_anchors() {
    if MainThreadMarker::new().is_none() {
        tracing::warn!("Break Space anchor release skipped: not on AppKit main thread");
        return;
    }
    let mut anchors = SPACE_ANCHORS.lock();
    for (_, panel) in anchors.drain() {
        panel.0.orderOut(None);
        panel.0.close();
    }
}
