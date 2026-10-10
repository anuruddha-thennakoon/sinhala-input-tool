//! Language-bar / tray button: shows whether Sinhala typing is on and offers the
//! on/off switch and an About box.

use crate::guids::{CLSID_SINHALA_TIP, GUID_LANGBAR_ITEM};
use crate::service::{guard_err, toggle_enabled, Shared};
use sinhala_core::about_text;
use std::cell::RefCell;
use windows::core::{implement, Interface, Ref, Result, BOOL, BSTR, GUID, IUnknown};
use windows::Win32::Foundation::{E_INVALIDARG, POINT, RECT};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::TextServices::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const MENU_TOGGLE: u32 = 1;
const MENU_ABOUT: u32 = 2;
const SINK_COOKIE: u32 = 1;

#[implement(ITfLangBarItemButton, ITfSource)]
pub(crate) struct LangBarButton {
    shared: Shared,
    sink: RefCell<Option<ITfLangBarItemSink>>,
}

impl LangBarButton {
    pub(crate) fn new(shared: Shared) -> LangBarButton {
        LangBarButton { shared, sink: RefCell::new(None) }
    }

    /// Tells the language bar to re-read icon and tooltip (after a toggle).
    pub(crate) fn notify(&self) {
        if let Some(sink) = self.sink.borrow().as_ref() {
            let _ = unsafe { sink.OnUpdate(TF_LBI_ICON | TF_LBI_TEXT | TF_LBI_TOOLTIP) };
        }
    }

    fn enabled(&self) -> bool {
        self.shared.borrow().enabled
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl ITfLangBarItem_Impl for LangBarButton_Impl {
    fn GetInfo(&self, info: *mut TF_LANGBARITEMINFO) -> Result<()> {
        if info.is_null() {
            return Err(E_INVALIDARG.into());
        }
        let mut item = TF_LANGBARITEMINFO {
            clsidService: CLSID_SINHALA_TIP,
            guidItem: GUID_LANGBAR_ITEM,
            dwStyle: TF_LBI_STYLE_BTN_MENU | TF_LBI_STYLE_SHOWNINTRAY,
            ulSort: 0,
            szDescription: [0; 32],
        };
        for (dst, src) in item.szDescription.iter_mut().zip(wide("Sinhala Input").into_iter().take(31)) {
            *dst = src;
        }
        unsafe { *info = item };
        Ok(())
    }

    fn GetStatus(&self) -> Result<u32> {
        Ok(0)
    }

    fn Show(&self, _show: BOOL) -> Result<()> {
        Ok(())
    }

    fn GetTooltipString(&self) -> Result<BSTR> {
        guard_err(|| {
            let state = if self.enabled() { "On" } else { "Off" };
            Ok(BSTR::from(format!("Sinhala Input: {state} (Ctrl+Shift+Space)")))
        })
    }
}

impl ITfLangBarItemButton_Impl for LangBarButton_Impl {
    fn OnClick(&self, _click: TfLBIClick, _pt: &POINT, _area: *const RECT) -> Result<()> {
        Ok(()) // BTN_MENU style: the language bar shows the menu itself
    }

    fn InitMenu(&self, menu: Ref<ITfMenu>) -> Result<()> {
        guard_err(|| self.init_menu(menu))
    }

    fn OnMenuSelect(&self, id: u32) -> Result<()> {
        guard_err(|| self.on_menu_select(id))
    }

    fn GetIcon(&self) -> Result<HICON> {
        guard_err(|| make_icon(self.enabled()))
    }

    fn GetText(&self) -> Result<BSTR> {
        Ok(BSTR::from("Sinhala Input"))
    }
}

impl LangBarButton_Impl {
    fn init_menu(&self, menu: Ref<ITfMenu>) -> Result<()> {
        let menu = menu.ok()?;
        let flags = if self.enabled() { TF_LBMENUF_CHECKED } else { 0 };
        unsafe {
            menu.AddMenuItem(
                MENU_TOGGLE,
                flags,
                HBITMAP::default(),
                HBITMAP::default(),
                &wide("Sinhala typing  (Ctrl+Shift+Space)"),
                std::ptr::null_mut(),
            )?;
            menu.AddMenuItem(
                MENU_ABOUT,
                0,
                HBITMAP::default(),
                HBITMAP::default(),
                &wide("About Sinhala Input..."),
                std::ptr::null_mut(),
            )?;
        }
        Ok(())
    }

    fn on_menu_select(&self, id: u32) -> Result<()> {
        match id {
            MENU_TOGGLE => toggle_enabled(&self.shared),
            MENU_ABOUT => unsafe {
                let text = wide(&about_text()).into_iter().chain([0]).collect::<Vec<u16>>();
                let title = wide("Sinhala Input").into_iter().chain([0]).collect::<Vec<u16>>();
                MessageBoxW(
                    None,
                    windows::core::PCWSTR(text.as_ptr()),
                    windows::core::PCWSTR(title.as_ptr()),
                    MB_OK | MB_ICONINFORMATION,
                );
            },
            _ => {}
        }
        Ok(())
    }
}

impl ITfSource_Impl for LangBarButton_Impl {
    fn AdviseSink(&self, riid: *const GUID, sink: Ref<IUnknown>) -> Result<u32> {
        if riid.is_null() || unsafe { *riid } != ITfLangBarItemSink::IID {
            return Err(E_INVALIDARG.into());
        }
        *self.sink.borrow_mut() = Some(sink.ok()?.cast()?);
        Ok(SINK_COOKIE)
    }

    fn UnadviseSink(&self, cookie: u32) -> Result<()> {
        if cookie != SINK_COOKIE {
            return Err(E_INVALIDARG.into());
        }
        *self.sink.borrow_mut() = None;
        Ok(())
    }
}

/// 16x16 status icon: white "සි" on blue when typing is on, white "EN" on grey when off.
/// The language bar takes ownership of the returned icon.
fn make_icon(enabled: bool) -> Result<HICON> {
    unsafe {
        let screen = GetDC(None);
        let dc = CreateCompatibleDC(Some(screen));
        let color = CreateCompatibleBitmap(screen, 16, 16);
        let mask = CreateBitmap(16, 16, 1, 1, None); // all zero = fully opaque
        let old = SelectObject(dc, color.into());

        let bg = if enabled { 0x00C0_7A2E } else { 0x0080_8080 }; // BGR
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(bg));
        FillRect(dc, &RECT { left: 0, top: 0, right: 16, bottom: 16 }, brush);
        let _ = DeleteObject(brush.into());

        let font = CreateFontW(
            -12, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            windows::core::w!("Nirmala UI"),
        );
        let old_font = SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));
        let mut text = wide(if enabled { "සි" } else { "EN" });
        let mut rect = RECT { left: 0, top: 0, right: 16, bottom: 16 };
        DrawTextW(dc, &mut text, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(dc, old_font);
        SelectObject(dc, old);
        let info = ICONINFO { fIcon: BOOL::from(true), xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&info);

        let _ = DeleteObject(font.into());
        let _ = DeleteObject(mask.into());
        let _ = DeleteObject(color.into());
        let _ = DeleteDC(dc);
        ReleaseDC(None, screen);
        icon
    }
}
