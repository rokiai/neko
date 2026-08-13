//! Windows virtual-desktop follow for Break overlays.
//!
//! Windows has no public "show on every virtual desktop" flag: Tauri's
//! `visible_on_all_workspaces` is documented as unsupported here, and real
//! pinning needs the undocumented `IVirtualDesktopPinnedApps` COM interface
//! whose CLSID changes between Windows builds.
//!
//! Use the documented `IVirtualDesktopManager` instead and move the overlay
//! onto the desktop the user is currently on. That mirrors the macOS
//! `MoveToActiveSpace` semantics implemented in `macos_spaces`.
//!
//! Showing needs no extra work: Break windows are built with `focused(false)`,
//! so tao shows them with `SW_SHOWNOACTIVATE` and marks them `WS_EX_NOACTIVATE`.

use std::cell::RefCell;
use std::ffi::c_void;

use tauri::{Manager, Runtime, WebviewWindow};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Shell::{IVirtualDesktopManager, VirtualDesktopManager};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, HWND_TOPMOST, IsWindowVisible, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SetWindowPos, ShowWindow,
};
use windows::core::GUID;

thread_local! {
    /// `IVirtualDesktopManager` is neither `Send` nor `Sync`; keep it on the
    /// Tauri main thread, which already has COM/OLE initialized by tao.
    static DESKTOP_MANAGER: RefCell<Option<IVirtualDesktopManager>> = const { RefCell::new(None) };
}

/// Move the Break overlay onto the active virtual desktop if it drifted off it.
pub fn follow_active_virtual_desktop<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    let handle = window
        .hwnd()
        .map_err(|error| format!("resolve Break window handle: {error}"))?
        .0 as isize;
    window
        .app_handle()
        .run_on_main_thread(move || move_to_active_desktop(HWND(handle as *mut c_void)))
        .map_err(|error| format!("dispatch Break window to main thread: {error}"))
}

fn move_to_active_desktop(overlay: HWND) {
    DESKTOP_MANAGER.with(|cell| {
        let mut cached = cell.borrow_mut();
        if cached.is_none() {
            // tao only calls OleInitialize when drag and drop is enabled, so do
            // not assume this thread already joined an apartment. Re-initializing
            // is harmless (S_FALSE / RPC_E_CHANGED_MODE) and never uninitialized.
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            }
            match unsafe { CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL) } {
                Ok(manager) => *cached = Some(manager),
                Err(error) => {
                    tracing::debug!("virtual desktop manager unavailable: {error}");
                    return;
                }
            }
        }
        let Some(manager) = cached.as_ref() else {
            return;
        };

        match unsafe { manager.IsWindowOnCurrentVirtualDesktop(overlay) } {
            Ok(on_current) if on_current.as_bool() => return,
            Ok(_) => {}
            Err(error) => {
                tracing::debug!("could not read Break window desktop: {error}");
                return;
            }
        }

        let Some(active_desktop) = active_desktop_id(manager) else {
            return;
        };
        if let Err(error) = unsafe { manager.MoveWindowToDesktop(overlay, &active_desktop) } {
            tracing::debug!("could not move Break window to active desktop: {error}");
            return;
        }
        raise_without_activating(overlay);
    });
}

/// The active desktop id is only reachable through a window that lives on it.
fn active_desktop_id(manager: &IVirtualDesktopManager) -> Option<GUID> {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.0.is_null() {
        return None;
    }
    let desktop = unsafe { manager.GetWindowDesktopId(foreground) }.ok()?;
    if desktop == GUID::zeroed() {
        return None;
    }
    Some(desktop)
}

fn raise_without_activating(overlay: HWND) {
    unsafe {
        if !IsWindowVisible(overlay).as_bool() {
            let _ = ShowWindow(overlay, SW_SHOWNOACTIVATE);
        }
        let _ = SetWindowPos(
            overlay,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}
