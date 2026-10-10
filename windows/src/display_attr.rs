//! Display attribute provider: tells TSF (and through it the host app) how to draw our
//! composition text. Without it the typed Latin text looks like finished text; with it
//! the text is underlined, the usual cue for "still being composed".

use crate::guids::GUID_DISPLAY_ATTR_INPUT;
use std::cell::Cell;
use windows::core::{implement, Result, BOOL, BSTR, GUID};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL, S_FALSE};
use windows::Win32::UI::TextServices::*;

#[implement(ITfDisplayAttributeInfo)]
pub struct DisplayAttrInfo;

impl ITfDisplayAttributeInfo_Impl for DisplayAttrInfo_Impl {
    fn GetGUID(&self) -> Result<GUID> {
        Ok(GUID_DISPLAY_ATTR_INPUT)
    }

    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::from("Sinhala Input: composing text"))
    }

    fn GetAttributeInfo(&self, out: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        if out.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // Default colours (type NONE), a solid underline, "input" (not yet converted).
        let attr = TF_DISPLAYATTRIBUTE {
            crText: TF_DA_COLOR::default(),
            crBk: TF_DA_COLOR::default(),
            lsStyle: TF_LS_SOLID,
            fBoldLine: BOOL::from(false),
            crLine: TF_DA_COLOR::default(),
            bAttr: TF_ATTR_INPUT,
        };
        unsafe { *out = attr };
        Ok(())
    }

    fn SetAttributeInfo(&self, _attr: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        Err(E_NOTIMPL.into()) // not user-customisable
    }

    fn Reset(&self) -> Result<()> {
        Ok(())
    }
}

/// Enumerates the one attribute we provide.
#[implement(IEnumTfDisplayAttributeInfo)]
pub struct DisplayAttrEnum {
    next: Cell<u32>,
}

impl DisplayAttrEnum {
    pub fn new() -> DisplayAttrEnum {
        DisplayAttrEnum { next: Cell::new(0) }
    }
}

impl IEnumTfDisplayAttributeInfo_Impl for DisplayAttrEnum_Impl {
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(DisplayAttrEnum { next: Cell::new(self.next.get()) }.into())
    }

    fn Next(&self, count: u32, out: *mut Option<ITfDisplayAttributeInfo>, fetched: *mut u32) -> Result<()> {
        let mut got = 0;
        if count >= 1 && self.next.get() == 0 && !out.is_null() {
            unsafe { out.write(Some(DisplayAttrInfo.into())) };
            self.next.set(1);
            got = 1;
        }
        if !fetched.is_null() {
            unsafe { *fetched = got };
        }
        if got == count {
            Ok(())
        } else {
            Err(S_FALSE.into()) // fewer than requested: success code S_FALSE
        }
    }

    fn Reset(&self) -> Result<()> {
        self.next.set(0);
        Ok(())
    }

    fn Skip(&self, count: u32) -> Result<()> {
        self.next.set(self.next.get().saturating_add(count).min(1));
        Ok(())
    }
}
