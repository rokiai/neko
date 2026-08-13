//! Per-platform screen-lock probes.
//!
//! Lock state feeds the scheduler independently of `idleResetEnabled`: a lock
//! longer than the idle threshold must pause/reset the schedule even when
//! plain idle detection is turned off (see `monitors::idle`).

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;
    use std::sync::LazyLock;

    pub(super) type CFTypeRef = *const c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        pub(super) fn CGSessionCopyCurrentDictionary() -> CFTypeRef;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            allocator: CFTypeRef,
            c_str: *const i8,
            encoding: u32,
        ) -> CFTypeRef;
        pub(super) fn CFDictionaryGetValue(dictionary: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
        pub(super) fn CFBooleanGetValue(value: CFTypeRef) -> u8;
        pub(super) fn CFRelease(value: CFTypeRef);
    }

    struct LockedKey(CFTypeRef);

    // SAFETY: an immutable CFString is safe to read from any thread. This one
    // is created once and intentionally never released, so the pointer stays
    // valid for the life of the process.
    unsafe impl Send for LockedKey {}
    unsafe impl Sync for LockedKey {}

    /// Created once instead of on every probe: the lock state is polled every
    /// second for as long as the app runs.
    static LOCKED_KEY: LazyLock<LockedKey> = LazyLock::new(|| {
        const KEY: &[u8] = b"CGSSessionScreenIsLocked\0";
        const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
        LockedKey(unsafe {
            CFStringCreateWithCString(
                std::ptr::null(),
                KEY.as_ptr().cast(),
                K_CF_STRING_ENCODING_UTF8,
            )
        })
    });

    pub(super) fn locked_key() -> CFTypeRef {
        LOCKED_KEY.0
    }
}

#[cfg(target_os = "macos")]
pub fn is_screen_locked() -> bool {
    use macos::{
        CFBooleanGetValue, CFDictionaryGetValue, CFRelease, CGSessionCopyCurrentDictionary,
        locked_key,
    };

    let key = locked_key();
    if key.is_null() {
        return false;
    }
    let dictionary = unsafe { CGSessionCopyCurrentDictionary() };
    if dictionary.is_null() {
        return false;
    }
    let value = unsafe { CFDictionaryGetValue(dictionary, key) };
    let locked = !value.is_null() && unsafe { CFBooleanGetValue(value) != 0 };
    unsafe { CFRelease(dictionary) };
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
