//! Drives the session through the exported C ABI, exactly as a native shell would.
use sinhala_core::ffi::*;
use std::ffi::{CStr, CString};
use std::ptr;

fn take(p: *mut std::os::raw::c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
    unsafe { sinhala_string_free(p) };
    Some(s)
}

fn type_str(s: *mut SinhalaSession, text: &str) {
    for c in text.chars() {
        assert!(unsafe { sinhala_push_char(s, c as u32) });
    }
}

#[test]
fn full_flow_through_c_abi() {
    unsafe {
        let s = sinhala_session_new();
        assert!(!s.is_null());
        assert!(!sinhala_is_composing(s));

        type_str(s, "niladhaari");
        assert!(sinhala_is_composing(s));
        assert_eq!(take(sinhala_buffer(s)).as_deref(), Some("niladhaari"));
        assert_eq!(sinhala_row_count(s), 6);
        assert_eq!(take(sinhala_row(s, 0)).as_deref(), Some("නිලධාරි")); // same first row as the reference UI
        assert_eq!(take(sinhala_row(s, 5)).as_deref(), Some("niladhaari"));
        assert_eq!(sinhala_selected_row(s), 0);
        assert!(sinhala_page_count(s) > 1);
        assert_eq!(sinhala_page(s), 0);
        assert!(sinhala_row(s, 6).is_null(), "out of range row");

        sinhala_move_selection(s, 1);
        assert_eq!(sinhala_selected_row(s), 1);
        assert_eq!(take(sinhala_commit(s)).as_deref(), Some("නිලධාරී"));
        assert!(!sinhala_is_composing(s));
        assert!(sinhala_commit(s).is_null());

        type_str(s, "mama");
        assert_eq!(take(sinhala_select_number(s, 1)).as_deref(), Some("මම"));

        type_str(s, "kopi");
        assert!(sinhala_backspace(s));
        sinhala_turn_page(s, 1);
        sinhala_cancel(s);
        assert!(!sinhala_is_composing(s));
        assert!(!sinhala_backspace(s));

        sinhala_session_free(s);
    }
}

#[test]
fn null_handles_are_safe() {
    unsafe {
        let n: *mut SinhalaSession = ptr::null_mut();
        assert!(!sinhala_push_char(n, 'a' as u32));
        assert!(!sinhala_backspace(n));
        assert!(!sinhala_is_composing(n));
        assert_eq!(sinhala_row_count(n), 0);
        assert!(sinhala_row(n, 0).is_null());
        assert!(sinhala_commit(n).is_null());
        assert!(sinhala_buffer(n).is_null());
        sinhala_move_selection(n, 1);
        sinhala_cancel(n);
        sinhala_session_free(n);
        sinhala_string_free(ptr::null_mut());
    }
}

#[test]
fn invalid_codepoint_is_rejected() {
    unsafe {
        let s = sinhala_session_new();
        assert!(!sinhala_push_char(s, 0xD800)); // lone surrogate
        assert!(!sinhala_push_char(s, 0x110000)); // beyond Unicode
        assert!(!sinhala_is_composing(s));
        sinhala_session_free(s);
    }
    let _ = CString::new("x");
}

#[test]
fn settings_and_about_through_c_abi() {
    let dir = std::env::temp_dir().join(format!("sinhala-ffi-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = CString::new(dir.join("settings.ini").to_str().unwrap()).unwrap();
    unsafe {
        assert!(sinhala_settings_load_enabled(path.as_ptr()), "default is enabled");
        assert!(sinhala_settings_save_enabled(path.as_ptr(), false));
        assert!(!sinhala_settings_load_enabled(path.as_ptr()));
        assert!(sinhala_settings_save_enabled(path.as_ptr(), true));
        assert!(sinhala_settings_load_enabled(path.as_ptr()));
        // NULL path: defaults / failure, no crash.
        assert!(sinhala_settings_load_enabled(ptr::null()));
        assert!(!sinhala_settings_save_enabled(ptr::null(), false));

        let about = take(sinhala_about_text()).unwrap();
        assert!(about.contains("Wikipedia"));
    }
    let _ = std::fs::remove_dir_all(&dir);
}
