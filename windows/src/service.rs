//! The text service: key handling, composition, and the glue to the Rust core.
//!
//! TSF only lets a text service touch a document inside an "edit session", so every
//! change to the composition (typing, commit, cancel) goes through `edit()`. The
//! service lives on the host app's UI thread (apartment-threaded), so shared state is
//! `Rc<RefCell<..>>`; no borrow is ever held across a call into TSF.

use crate::config;
use crate::keys::{route, KeyAction, Mods};
use crate::langbar::LangBarButton;
use crate::log::log;
use crate::popup::Popup;
use sinhala_core::{Dictionary, Session};
use std::cell::RefCell;
use std::rc::Rc;
use windows::core::{implement, AsImpl, IUnknownImpl, Interface, Result, BOOL};
use windows::Win32::Foundation::{E_FAIL, LPARAM, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows::Win32::UI::TextServices::*;

pub static OBJECT_COUNT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub(crate) struct Inner {
    pub(crate) enabled: bool,
    tid: u32,
    thread_mgr: Option<ITfThreadMgr>,
    session: Session<'static>,
    composition: Option<ITfComposition>,
    popup: Option<Popup>,
    /// This service as a composition sink. Holds a reference to ourselves, so it is
    /// set in Activate and cleared in Deactivate to break the cycle.
    sink: Option<ITfCompositionSink>,
    langbar: Option<ITfLangBarItemButton>,
}

pub(crate) type Shared = Rc<RefCell<Inner>>;

#[implement(ITfTextInputProcessorEx, ITfKeyEventSink, ITfCompositionSink)]
pub struct TextService {
    inner: Shared,
}

impl TextService {
    pub fn new() -> TextService {
        OBJECT_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        TextService {
            inner: Rc::new(RefCell::new(Inner {
                enabled: config::load_enabled(),
                langbar: None,
                tid: 0,
                thread_mgr: None,
                session: Session::new(Dictionary::embedded()),
                composition: None,
                popup: None,
                sink: None,
            })),
        }
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        OBJECT_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------- lifecycle

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: windows::core::Ref<ITfThreadMgr>, tid: u32) -> Result<()> {
        self.activate(ptim, tid)
    }

    fn Deactivate(&self) -> Result<()> {
        log("Deactivate");
        let thread_mgr = {
            let mut inner = self.inner.borrow_mut();
            inner.session.cancel();
            inner.composition = None;
            inner.popup = None; // destroys the popup window
            inner.sink = None;
            inner.thread_mgr.take()
        };
        let langbar = self.inner.borrow_mut().langbar.take();
        if let Some(tm) = thread_mgr {
            let tid = self.inner.borrow().tid;
            unsafe {
                if let (Some(button), Ok(mgr)) = (&langbar, tm.cast::<ITfLangBarItemMgr>()) {
                    let _ = mgr.RemoveItem(button);
                }
                tm.cast::<ITfKeystrokeMgr>()?.UnadviseKeyEventSink(tid)?
            };
        }
        self.inner.borrow_mut().tid = 0;
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: windows::core::Ref<ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        self.activate(ptim, tid)
    }
}

impl TextService_Impl {
    fn activate(&self, ptim: windows::core::Ref<ITfThreadMgr>, tid: u32) -> Result<()> {
        log(format!("Activate tid={tid}"));
        let tm: ITfThreadMgr = ptim.ok()?.clone();
        let sink: ITfKeyEventSink = self.to_interface();
        unsafe { tm.cast::<ITfKeystrokeMgr>()?.AdviseKeyEventSink(tid, &sink, true)? };
        let mut inner = self.inner.borrow_mut();
        inner.tid = tid;
        inner.enabled = config::load_enabled();
        inner.popup = Popup::new().ok();
        inner.sink = Some(self.to_interface());

        // Tray / language-bar button. Failure here must not stop typing from working.
        let button: ITfLangBarItemButton = LangBarButton::new(self.inner.clone()).into();
        if let Ok(mgr) = tm.cast::<ITfLangBarItemMgr>() {
            if unsafe { mgr.AddItem(&button) }.is_ok() {
                inner.langbar = Some(button);
            }
        }
        inner.thread_mgr = Some(tm);
        Ok(())
    }
}

// ----------------------------------------------------------------- key events

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, foreground: BOOL) -> Result<()> {
        if foreground.as_bool() {
            // Another process may have toggled typing while it had focus.
            let on = config::load_enabled();
            let changed = std::mem::replace(&mut self.inner.borrow_mut().enabled, on) != on;
            if changed {
                notify_langbar(&self.inner);
            }
        }
        Ok(())
    }

    fn OnTestKeyDown(&self, _pic: windows::core::Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(self.action_for(wparam) != KeyAction::PassThrough))
    }

    fn OnTestKeyUp(&self, _pic: windows::core::Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        // Swallow the key-up of keys whose key-down we ate, so the app never sees half a keystroke.
        Ok(BOOL::from(self.action_for(wparam) != KeyAction::PassThrough && self.is_composing()))
    }

    fn OnKeyDown(&self, pic: windows::core::Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let ctx: ITfContext = pic.ok()?.clone();
        let action = self.action_for(wparam);
        log(format!("OnKeyDown vk={:#x} -> {action:?}", wparam.0));
        Ok(BOOL::from(self.perform(&ctx, action)?))
    }

    fn OnKeyUp(&self, _pic: windows::core::Ref<ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(self.action_for(wparam) != KeyAction::PassThrough && self.is_composing()))
    }

    fn OnPreservedKey(&self, _pic: windows::core::Ref<ITfContext>, _guid: *const windows::core::GUID) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }
}

impl ITfCompositionSink_Impl for TextService_Impl {
    /// The app or TSF ended our composition (focus change, document edit, ...).
    fn OnCompositionTerminated(&self, _ec: u32, _comp: windows::core::Ref<ITfComposition>) -> Result<()> {
        log("OnCompositionTerminated");
        let mut inner = self.inner.borrow_mut();
        inner.composition = None;
        inner.session.cancel();
        if let Some(p) = &inner.popup {
            p.hide();
        }
        Ok(())
    }
}

impl TextService_Impl {
    fn is_composing(&self) -> bool {
        self.inner.borrow().session.is_composing()
    }

    fn action_for(&self, wparam: WPARAM) -> KeyAction {
        route(wparam.0 as u32, read_mods(), self.is_composing(), self.inner.borrow().enabled)
    }

    /// Applies `action` to the session and the document. Returns whether the key was eaten.
    fn perform(&self, ctx: &ITfContext, action: KeyAction) -> Result<bool> {
        let eaten = match action {
            KeyAction::PassThrough => return Ok(false),
            KeyAction::Type(c) => {
                self.inner.borrow_mut().session.push_char(c);
                true
            }
            KeyAction::MoveSelection(d) => {
                self.inner.borrow_mut().session.move_selection(d);
                true
            }
            KeyAction::TurnPage(d) => {
                self.inner.borrow_mut().session.turn_page(d);
                true
            }
            KeyAction::Backspace => self.inner.borrow_mut().session.backspace(),
            KeyAction::Cancel => {
                self.inner.borrow_mut().session.cancel();
                true
            }
            KeyAction::ToggleEnabled => {
                let pending = self.inner.borrow_mut().session.commit();
                if let Some(text) = pending {
                    finish(&self.inner, ctx, &text)?;
                }
                toggle_enabled(&self.inner);
                return Ok(true);
            }
            KeyAction::Commit | KeyAction::CommitAndSpace | KeyAction::CommitThenPassThrough | KeyAction::PickNumber(_) => {
                let text = {
                    let mut inner = self.inner.borrow_mut();
                    match action {
                        KeyAction::PickNumber(n) => inner.session.select_number(n as usize),
                        _ => inner.session.commit(),
                    }
                };
                let Some(mut text) = text else {
                    // Number out of range for this page: keep composing, just redraw.
                    sync(&self.inner, ctx)?;
                    return Ok(true);
                };
                if action == KeyAction::CommitAndSpace {
                    text.push(' ');
                }
                finish(&self.inner, ctx, &text)?;
                return Ok(action != KeyAction::CommitThenPassThrough);
            }
        };
        if eaten {
            sync(&self.inner, ctx)?;
        }
        Ok(eaten)
    }
}

fn read_mods() -> Mods {
    let down = |vk: u16| unsafe { GetKeyState(vk as i32) } < 0;
    Mods {
        ctrl: down(VK_CONTROL.0),
        alt: down(VK_MENU.0),
        shift: down(VK_SHIFT.0),
        caps: unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0,
    }
}

// --------------------------------------------------------------- edit sessions

type EditFn = Box<dyn FnOnce(u32) -> Result<()>>;

#[implement(ITfEditSession)]
struct EditSession {
    f: RefCell<Option<EditFn>>,
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        match self.f.borrow_mut().take() {
            Some(f) => f(ec),
            None => Ok(()),
        }
    }
}

/// Runs `f` synchronously inside a read/write edit session on `ctx`.
fn edit(inner: &Shared, ctx: &ITfContext, f: impl FnOnce(u32) -> Result<()> + 'static) -> Result<()> {
    let tid = inner.borrow().tid;
    let session: ITfEditSession = EditSession { f: RefCell::new(Some(Box::new(f))) }.into();
    unsafe {
        let hr = ctx.RequestEditSession(tid, &session, TF_ES_SYNC | TF_ES_READWRITE)?;
        hr.ok()
    }
}

/// Brings the document and popup in line with the session: shows the Latin buffer as
/// the composition string and the popup under it, or ends the composition if empty.
fn sync(inner: &Shared, ctx: &ITfContext) -> Result<()> {
    if !inner.borrow().session.is_composing() {
        return finish(inner, ctx, "");
    }
    let shared = inner.clone();
    let ctx2 = ctx.clone();
    edit(inner, ctx, move |ec| unsafe {
        let buffer = shared.borrow().session.buffer().to_string();
        let range = composition_range(&shared, &ctx2, ec)?;
        let text: Vec<u16> = buffer.encode_utf16().collect();
        range.SetText(ec, 0, &text)?;

        // Caret after the composition text.
        let end = range.Clone()?;
        end.Collapse(ec, TF_ANCHOR_END)?;
        let selection = [TF_SELECTION {
            range: std::mem::ManuallyDrop::new(Some(end)),
            style: TF_SELECTIONSTYLE { ase: TF_AE_NONE, fInterimChar: BOOL::from(false) },
        }];
        let _ = ctx2.SetSelection(ec, &selection);

        // Popup under the composition.
        let mut rect = RECT::default();
        let mut clipped = BOOL::default();
        let view = ctx2.GetActiveView()?;
        view.GetTextExt(ec, &range, &mut rect, &mut clipped)?;
        let guard = shared.borrow();
        if let Some(popup) = &guard.popup {
            popup.show(guard.session.view(), guard.session.buffer(), rect);
        }
        Ok(())
    })
}

/// Replaces the composition with `text`, ends it, and hides the popup.
fn finish(inner: &Shared, ctx: &ITfContext, text: &str) -> Result<()> {
    let shared = inner.clone();
    let text16: Vec<u16> = text.encode_utf16().collect();
    let ctx2 = ctx.clone();
    let result = edit(inner, ctx, move |ec| unsafe {
        let comp = shared.borrow_mut().composition.take();
        // Nothing to replace and nothing to insert: no composition was ever started.
        if comp.is_none() && text16.is_empty() {
            return Ok(());
        }
        let range = match &comp {
            Some(c) => c.GetRange()?,
            None => {
                let r = composition_range(&shared, &ctx2, ec)?;
                shared.borrow_mut().composition.take(); // we only needed a place to insert
                r
            }
        };
        range.SetText(ec, 0, &text16)?;
        let end = range.Clone()?;
        end.Collapse(ec, TF_ANCHOR_END)?;
        let selection = [TF_SELECTION {
            range: std::mem::ManuallyDrop::new(Some(end)),
            style: TF_SELECTIONSTYLE { ase: TF_AE_NONE, fInterimChar: BOOL::from(false) },
        }];
        let _ = ctx2.SetSelection(ec, &selection);
        if let Some(c) = comp {
            c.EndComposition(ec)?;
        }
        Ok(())
    });
    if let Some(p) = &inner.borrow().popup {
        p.hide();
    }
    result
}

/// Returns the range of the active composition, starting one at the caret if needed.
unsafe fn composition_range(inner: &Shared, ctx: &ITfContext, ec: u32) -> Result<ITfRange> {
    if let Some(c) = &inner.borrow().composition {
        return c.GetRange();
    }
    let insert: ITfInsertAtSelection = ctx.cast()?;
    let at_caret = insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?;
    let owner = ctx.cast::<ITfContextComposition>()?;
    let sink = inner.borrow().sink.clone().ok_or_else(|| windows::core::Error::from(E_FAIL))?;
    let comp = owner.StartComposition(ec, &at_caret, &sink)?;
    let range = comp.GetRange()?;
    inner.borrow_mut().composition = Some(comp);
    Ok(range)
}

/// Flips Sinhala typing on/off, persists it, and refreshes the tray icon. Any word in
/// progress is abandoned (callers that have a document context commit it first).
pub(crate) fn toggle_enabled(inner: &Shared) {
    let on = {
        let mut i = inner.borrow_mut();
        i.enabled = !i.enabled;
        i.session.cancel();
        i.composition = None;
        if let Some(p) = &i.popup {
            p.hide();
        }
        i.enabled
    };
    config::save_enabled(on);
    log(format!("typing enabled = {on}"));
    notify_langbar(inner);
}

fn notify_langbar(inner: &Shared) {
    let button = inner.borrow().langbar.clone();
    if let Some(button) = button {
        let b: &LangBarButton = unsafe { button.as_impl() };
        b.notify();
    }
}
