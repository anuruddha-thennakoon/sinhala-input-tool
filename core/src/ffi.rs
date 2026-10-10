//! C ABI for the native shells (TSF on Windows, InputMethodKit on macOS).
//!
//! Conventions: strings are UTF-8, NUL-terminated, and every non-NULL string
//! returned must be released with `sinhala_string_free`. All functions accept a
//! NULL session and then do nothing / return 0, NULL or false. Panics never
//! cross the boundary. The header is generated into `include/sinhala_core.h`.

use crate::{Dictionary, Session};
use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

/// Opaque session handle.
pub struct SinhalaSession {
    inner: Session<'static>,
}

fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(default)
}

fn to_c(s: String) -> *mut c_char {
    CString::new(s).map_or(ptr::null_mut(), CString::into_raw)
}

unsafe fn with<T>(s: *mut SinhalaSession, default: T, f: impl FnOnce(&mut Session<'static>) -> T) -> T {
    if s.is_null() {
        return default;
    }
    // SAFETY: caller guarantees `s` came from `sinhala_session_new` and is live.
    let session = unsafe { &mut (*s).inner };
    guard(default, || f(session))
}

/// Creates a session backed by the embedded dictionary. Free with `sinhala_session_free`.
#[no_mangle]
pub extern "C" fn sinhala_session_new() -> *mut SinhalaSession {
    guard(ptr::null_mut(), || {
        Box::into_raw(Box::new(SinhalaSession {
            inner: Session::new(Dictionary::embedded()),
        }))
    })
}

/// # Safety
/// `s` must be NULL or a live handle from `sinhala_session_new`, not used afterwards.
#[no_mangle]
pub unsafe extern "C" fn sinhala_session_free(s: *mut SinhalaSession) {
    if !s.is_null() {
        // SAFETY: see above.
        drop(unsafe { Box::from_raw(s) });
    }
}

/// # Safety
/// `p` must be NULL or a string returned by this library, not freed before.
#[no_mangle]
pub unsafe extern "C" fn sinhala_string_free(p: *mut c_char) {
    if !p.is_null() {
        // SAFETY: see above.
        drop(unsafe { CString::from_raw(p) });
    }
}

/// Appends a typed Unicode code point. Returns false for NULL or an invalid code point.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_push_char(s: *mut SinhalaSession, codepoint: u32) -> bool {
    let Some(c) = char::from_u32(codepoint) else {
        return false;
    };
    unsafe {
        with(s, false, |se| {
            se.push_char(c);
            true
        })
    }
}

/// Returns false when there was nothing to delete (pass Backspace to the app).
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_backspace(s: *mut SinhalaSession) -> bool {
    unsafe { with(s, false, |se| se.backspace()) }
}

/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_is_composing(s: *mut SinhalaSession) -> bool {
    unsafe { with(s, false, |se| se.is_composing()) }
}

/// The raw Latin buffer. Free with `sinhala_string_free`.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_buffer(s: *mut SinhalaSession) -> *mut c_char {
    unsafe { with(s, ptr::null_mut(), |se| to_c(se.buffer().to_string())) }
}

/// Number of rows on the current page (candidates plus the raw Latin row).
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_row_count(s: *mut SinhalaSession) -> u32 {
    unsafe { with(s, 0, |se| se.view().rows.len() as u32) }
}

/// Row text for the current page, or NULL if out of range. Free with `sinhala_string_free`.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_row(s: *mut SinhalaSession, index: u32) -> *mut c_char {
    unsafe {
        with(s, ptr::null_mut(), |se| {
            se.view()
                .rows
                .get(index as usize)
                .map_or(ptr::null_mut(), |r| to_c(r.clone()))
        })
    }
}

/// Highlighted row index within the current page.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_selected_row(s: *mut SinhalaSession) -> u32 {
    unsafe { with(s, 0, |se| se.view().selected as u32) }
}

/// Current page, zero-based.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_page(s: *mut SinhalaSession) -> u32 {
    unsafe { with(s, 0, |se| se.view().page as u32) }
}

/// Total number of pages (at least 1).
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_page_count(s: *mut SinhalaSession) -> u32 {
    unsafe { with(s, 1, |se| se.view().page_count as u32) }
}

/// Moves the highlight by `delta` (negative = up), crossing pages.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_move_selection(s: *mut SinhalaSession, delta: i32) {
    unsafe { with(s, (), |se| se.move_selection(delta)) }
}

/// Turns pages by `delta` (negative = previous).
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_turn_page(s: *mut SinhalaSession, delta: i32) {
    unsafe { with(s, (), |se| se.turn_page(delta)) }
}

/// Commits the highlighted candidate; NULL if not composing. Free with `sinhala_string_free`.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_commit(s: *mut SinhalaSession) -> *mut c_char {
    unsafe { with(s, ptr::null_mut(), |se| se.commit().map_or(ptr::null_mut(), to_c)) }
}

/// Commits row `n` (1-based) of the current page; NULL if out of range.
/// Free with `sinhala_string_free`.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_select_number(s: *mut SinhalaSession, n: u32) -> *mut c_char {
    unsafe {
        with(s, ptr::null_mut(), |se| {
            se.select_number(n as usize).map_or(ptr::null_mut(), to_c)
        })
    }
}

/// Abandons composition.
///
/// # Safety
/// `s` must be NULL or a live session handle.
#[no_mangle]
pub unsafe extern "C" fn sinhala_cancel(s: *mut SinhalaSession) {
    unsafe { with(s, (), |se| se.cancel()) }
}
