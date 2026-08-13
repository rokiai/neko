use user_idle2::UserIdle;

use crate::scheduler::state::SchedulerState;

pub const MAX_DETECTION_FAILURES: u8 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IdleStatus {
    pub idle: bool,
    pub locked: bool,
    pub lock_start_at_ms: Option<i64>,
}

pub fn read_status(
    scheduler: &mut SchedulerState,
    threshold_seconds: i64,
    idle_reset_enabled: bool,
    now_ms: i64,
) -> IdleStatus {
    if scheduler.idle_detection_disabled {
        return IdleStatus::default();
    }

    match UserIdle::get_time() {
        Ok(idle) => {
            scheduler.idle_detection_failures = 0;
            let locked = is_screen_locked();
            let previous_lock_start = scheduler.lock_start_at_ms;
            if locked {
                scheduler.lock_start_at_ms.get_or_insert(now_ms);
            } else {
                scheduler.lock_start_at_ms = None;
            }
            let threshold_ms = threshold_seconds.max(1) * 1_000;
            let idle_long_enough = idle.as_seconds() >= threshold_seconds.max(1) as u64;
            let locked_long_enough = scheduler
                .lock_start_at_ms
                .is_some_and(|start| now_ms.saturating_sub(start) >= threshold_ms);
            IdleStatus {
                idle: locked_long_enough || (idle_reset_enabled && idle_long_enough && !locked),
                locked,
                lock_start_at_ms: if locked {
                    scheduler.lock_start_at_ms
                } else {
                    previous_lock_start
                },
            }
        }
        Err(error) => {
            scheduler.idle_detection_failures = scheduler.idle_detection_failures.saturating_add(1);
            if scheduler.idle_detection_failures >= MAX_DETECTION_FAILURES {
                scheduler.idle_detection_disabled = true;
                tracing::warn!("idle detection disabled after repeated failures: {error}");
            } else {
                tracing::warn!("could not read idle duration: {error}");
            }
            IdleStatus::default()
        }
    }
}

#[cfg(target_os = "macos")]
fn is_screen_locked() -> bool {
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

#[cfg(not(target_os = "macos"))]
fn is_screen_locked() -> bool {
    false
}
