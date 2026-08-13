//! Per-platform screen-lock probes.
//!
//! Lock state feeds the scheduler independently of `idleResetEnabled`: a lock
//! longer than the idle threshold must pause/reset the schedule even when
//! plain idle detection is turned off (see `monitors::idle`).

#[cfg(target_os = "macos")]
pub fn is_screen_locked() -> bool {
    use std::ffi::c_void;

    type CFTypeRef = *const c_void;
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGSessionCopyCurrentDictionary() -> CFTypeRef;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            allocator: CFTypeRef,
            c_str: *const i8,
            encoding: u32,
        ) -> CFTypeRef;
        fn CFDictionaryGetValue(dictionary: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
        fn CFBooleanGetValue(value: CFTypeRef) -> u8;
        fn CFRelease(value: CFTypeRef);
    }

    let dictionary = unsafe { CGSessionCopyCurrentDictionary() };
    if dictionary.is_null() {
        return false;
    }
    let key_name = b"CGSSessionScreenIsLocked\0";
    let key = unsafe {
        CFStringCreateWithCString(std::ptr::null(), key_name.as_ptr().cast(), 0x0800_0100)
    };
    if key.is_null() {
        unsafe { CFRelease(dictionary) };
        return false;
    }
    let value = unsafe { CFDictionaryGetValue(dictionary, key) };
    let locked = !value.is_null() && unsafe { CFBooleanGetValue(value) != 0 };
    unsafe {
        CFRelease(key);
        CFRelease(dictionary);
    }
    locked
}

#[cfg(windows)]
pub fn is_screen_locked() -> bool {
    use windows::Win32::System::RemoteDesktop::{
        WTS_CURRENT_SERVER_HANDLE, WTS_CURRENT_SESSION, WTS_SESSIONSTATE_LOCK, WTSFreeMemory,
        WTSINFOEXW, WTSQuerySessionInformationW, WTSSessionInfoEx,
    };
    use windows::core::PWSTR;

    let mut buffer = PWSTR::null();
    let mut bytes = 0u32;
    let query = unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &mut buffer,
            &mut bytes,
        )
    };
    if query.is_err() || buffer.is_null() || (bytes as usize) < size_of::<WTSINFOEXW>() {
        return false;
    }
    // SessionFlags is only meaningful for Level 1 payloads. WTS_SESSIONSTATE_LOCK
    // is 0 on Windows 8+; the inverted Windows 7 semantics are not supported.
    let locked = unsafe {
        let info = &*buffer.as_ptr().cast::<WTSINFOEXW>();
        info.Level == 1 && info.Data.WTSInfoExLevel1.SessionFlags == WTS_SESSIONSTATE_LOCK as i32
    };
    unsafe { WTSFreeMemory(buffer.as_ptr().cast()) };
    locked
}

// Linux: parity with the Electron baseline, which never had lock detection
// there (powerMonitor reports `locked` on macOS/Windows only). A D-Bus probe
// covering org.freedesktop.ScreenSaver / GNOME / login1 is tracked as review
// item N2 in docs/TAURI-MIGRATION-REVIEW.md.
#[cfg(not(any(target_os = "macos", windows)))]
pub fn is_screen_locked() -> bool {
    false
}
