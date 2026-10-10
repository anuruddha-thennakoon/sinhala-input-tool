//! Candidate popup: a non-activating topmost window drawn with GDI, styled like
//! Google Input Tools (typed text and blue caret on top, numbered rows, grey
//! highlight, page chevrons).

use sinhala_core::PopupView;
use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::*;

const CLASS_NAME: PCWSTR = w!("SinhalaTipCandidatePopup");

const WHITE: COLORREF = rgb(255, 255, 255);
const BLACK: COLORREF = rgb(0, 0, 0);
const BORDER: COLORREF = rgb(204, 204, 204);
const HIGHLIGHT: COLORREF = rgb(238, 238, 238);
const CARET_BLUE: COLORREF = rgb(64, 153, 242);
const CHEVRON: COLORREF = rgb(64, 64, 64);
const CHEVRON_OFF: COLORREF = rgb(190, 190, 190);

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

/// Scales a 96-dpi design value to the current DPI.
fn px(v: i32, dpi: u32) -> i32 {
    (v as i64 * dpi as i64 / 96) as i32
}

/// What `WM_PAINT` draws; shared with the window through `GWLP_USERDATA`.
#[derive(Default)]
struct PaintState {
    view: Option<PopupView>,
    buffer: String,
    dpi: u32,
}

pub struct Popup {
    hwnd: HWND,
    state: Box<RefCell<PaintState>>,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl Popup {
    pub fn new() -> windows::core::Result<Popup> {
        unsafe {
            let hinst = GetModuleHandleW(None)?;
            let class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DROPSHADOW | CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinst.into(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                lpszClassName: CLASS_NAME,
                ..Default::default()
            };
            // Registering twice (second text-service instance in this process) fails
            // with ERROR_CLASS_ALREADY_EXISTS, which is fine.
            RegisterClassExW(&class);

            let state: Box<RefCell<PaintState>> = Box::default();
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!(""),
                WS_POPUP,
                0,
                0,
                10,
                10,
                None,
                None,
                Some(hinst.into()),
                None,
            )?;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*state as *const RefCell<PaintState> as usize as _);
            Ok(Popup { hwnd, state })
        }
    }

    /// Shows the popup below `caret` (screen coordinates), flipping above if it
    /// would run off the bottom of the monitor.
    pub fn show(&self, view: PopupView, buffer: &str, caret: RECT) {
        unsafe {
            let dpi = GetDpiForSystem().max(96);
            {
                let mut st = self.state.borrow_mut();
                st.view = Some(view);
                st.buffer = buffer.to_string();
                st.dpi = dpi;
            }
            let (w, h) = self.measure(dpi);

            let mon = MonitorFromRect(&caret, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let _ = GetMonitorInfoW(mon, &mut mi);
            let area = mi.rcWork;

            let mut x = caret.left;
            let mut y = caret.bottom + px(2, dpi);
            if y + h > area.bottom {
                y = caret.top - h - px(2, dpi); // no room below
            }
            x = x.min(area.right - w).max(area.left);
            y = y.max(area.top);

            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn measure(&self, dpi: u32) -> (i32, i32) {
        unsafe {
            let hdc = GetDC(Some(self.hwnd));
            let st = self.state.borrow();
            let Some(view) = &st.view else {
                ReleaseDC(Some(self.hwnd), hdc);
                return (10, 10);
            };
            let l = Layout::new(dpi);
            let header = l.font(hdc, 20);
            let rows = l.font(hdc, 18);

            let old = SelectObject(hdc, header.into());
            let mut widest = text_width(hdc, &st.buffer) + px(8, dpi);
            SelectObject(hdc, rows.into());
            for (i, r) in view.rows.iter().enumerate() {
                widest = widest.max(text_width(hdc, &format!("{}. {}", i + 1, r)));
            }
            SelectObject(hdc, old);
            let _ = DeleteObject(header.into());
            let _ = DeleteObject(rows.into());
            ReleaseDC(Some(self.hwnd), hdc);

            let w = (widest + 2 * l.pad + px(8, dpi)).max(px(150, dpi));
            let footer = if view.page_count > 1 { l.footer } else { px(6, dpi) };
            let h = l.header + view.rows.len() as i32 * l.row + footer + l.pad / 2;
            (w, h)
        }
    }
}

impl Drop for Popup {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Design metrics, scaled for DPI.
struct Layout {
    dpi: u32,
    pad: i32,
    row: i32,
    header: i32,
    footer: i32,
}

impl Layout {
    fn new(dpi: u32) -> Layout {
        Layout {
            dpi,
            pad: px(12, dpi),
            row: px(30, dpi),
            header: px(38, dpi),
            footer: px(34, dpi),
        }
    }

    /// A Sinhala-capable font ("Nirmala UI" ships with Windows 8 and later).
    unsafe fn font(&self, _hdc: HDC, size_px_96: i32) -> HFONT {
        CreateFontW(
            -px(size_px_96, self.dpi),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Nirmala UI"),
        )
    }
}

unsafe fn text_width(hdc: HDC, s: &str) -> i32 {
    let mut size = SIZE::default();
    let _ = GetTextExtentPoint32W(hdc, &wide(s), &mut size);
    size.cx
}

unsafe fn fill(hdc: HDC, r: &RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, r, brush);
    let _ = DeleteObject(brush.into());
}

unsafe fn paint(hwnd: HWND, st: &PaintState) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    if let Some(view) = &st.view {
        let l = Layout::new(st.dpi.max(96));
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);

        fill(hdc, &client, WHITE);
        let frame = CreateSolidBrush(BORDER);
        FrameRect(hdc, &client, frame);
        let _ = DeleteObject(frame.into());

        SetBkMode(hdc, TRANSPARENT);
        let header_font = l.font(hdc, 20);
        let row_font = l.font(hdc, 18);
        let old = SelectObject(hdc, header_font.into());

        // Header: typed Latin text plus a blue caret.
        SetTextColor(hdc, BLACK);
        let _ = TextOutW(hdc, l.pad, px(6, l.dpi), &wide(&st.buffer));
        let caret_x = l.pad + text_width(hdc, &st.buffer) + px(2, l.dpi);
        fill(
            hdc,
            &RECT {
                left: caret_x,
                top: px(8, l.dpi),
                right: caret_x + px(2, l.dpi),
                bottom: px(8, l.dpi) + px(22, l.dpi),
            },
            CARET_BLUE,
        );

        // Rows.
        SelectObject(hdc, row_font.into());
        for (i, text) in view.rows.iter().enumerate() {
            let top = l.header + i as i32 * l.row;
            if i == view.selected {
                fill(
                    hdc,
                    &RECT {
                        left: l.pad - px(4, l.dpi),
                        top,
                        right: client.right - (l.pad - px(4, l.dpi)),
                        bottom: top + l.row,
                    },
                    HIGHLIGHT,
                );
            }
            let _ = TextOutW(hdc, l.pad, top + px(4, l.dpi), &wide(&format!("{}. {}", i + 1, text)));
        }

        // Page chevrons.
        if view.page_count > 1 {
            let top = l.header + view.rows.len() as i32 * l.row + px(6, l.dpi);
            chevron(hdc, &l, l.pad, top, true, view.page > 0);
            chevron(hdc, &l, l.pad + px(26, l.dpi), top, false, view.page + 1 < view.page_count);
        }

        SelectObject(hdc, old);
        let _ = DeleteObject(header_font.into());
        let _ = DeleteObject(row_font.into());
    }
    let _ = EndPaint(hwnd, &ps);
}

unsafe fn chevron(hdc: HDC, l: &Layout, left: i32, top: i32, up: bool, enabled: bool) {
    let (w, h) = (px(24, l.dpi), px(22, l.dpi));
    let box_rect = RECT { left, top, right: left + w, bottom: top + h };
    let frame = CreateSolidBrush(BORDER);
    FrameRect(hdc, &box_rect, frame);
    let _ = DeleteObject(frame.into());

    let pen = CreatePen(PS_SOLID, px(2, l.dpi), if enabled { CHEVRON } else { CHEVRON_OFF });
    let old = SelectObject(hdc, pen.into());
    let (cx, cy) = (left + w / 2, top + h / 2);
    let (dx, dy) = (px(6, l.dpi), px(3, l.dpi));
    let (tip_y, base_y) = if up { (cy - dy, cy + dy) } else { (cy + dy, cy - dy) };
    let _ = MoveToEx(hdc, cx - dx, base_y, None);
    let _ = LineTo(hdc, cx, tip_y);
    let _ = LineTo(hdc, cx + dx, base_y);
    SelectObject(hdc, old);
    let _ = DeleteObject(pen.into());
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // A panic must not unwind out of a window procedure (it would abort the host app).
    match std::panic::catch_unwind(|| unsafe { handle_message(hwnd, msg, wparam, lparam) }) {
        Ok(result) => result,
        Err(_) => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn handle_message(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as usize as *const RefCell<PaintState>;
            if let Some(cell) = ptr.as_ref() {
                if let Ok(st) = cell.try_borrow() {
                    paint(hwnd, &st);
                    return LRESULT(0);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize), // never steal focus from the app
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
