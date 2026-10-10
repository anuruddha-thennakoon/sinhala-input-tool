//! COM DLL entry points (regsvr32 and TSF load the service through these).

use crate::guids::CLSID_SINHALA_TIP;
use crate::log::log;
use crate::registration;
use crate::service::{TextService, OBJECT_COUNT};
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::core::{implement, IUnknown, Interface, Ref, BOOL, GUID, HRESULT};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_POINTER, HINSTANCE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
use windows::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleFileNameW};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::TextServices::ITfTextInputProcessorEx;

static MODULE: AtomicIsize = AtomicIsize::new(0);

#[no_mangle]
pub extern "system" fn DllMain(hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        MODULE.store(hinst.0 as isize, Ordering::SeqCst);
        unsafe {
            let _ = DisableThreadLibraryCalls(hinst.into());
        }
    }
    BOOL::from(true)
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(&self, outer: Ref<IUnknown>, riid: *const GUID, object: *mut *mut c_void) -> windows::core::Result<()> {
        if object.is_null() || riid.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *object = std::ptr::null_mut() };
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let service: ITfTextInputProcessorEx = TextService::new().into();
        log("CreateInstance");
        unsafe { service.query(riid, object).ok() }
    }

    fn LockServer(&self, lock: BOOL) -> windows::core::Result<()> {
        if lock.as_bool() {
            OBJECT_COUNT.fetch_add(1, Ordering::SeqCst);
        } else {
            OBJECT_COUNT.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

/// # Safety
/// Called by COM with valid out-pointers (or null, which is rejected).
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(clsid: *const GUID, riid: *const GUID, object: *mut *mut c_void) -> HRESULT {
    if clsid.is_null() || riid.is_null() || object.is_null() {
        return E_POINTER;
    }
    if unsafe { *clsid } != CLSID_SINHALA_TIP {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = Factory.into();
    unsafe { factory.query(riid, object) }
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if OBJECT_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

fn module_path() -> Option<String> {
    let mut buf = [0u16; 1024];
    let hinst = HINSTANCE(MODULE.load(Ordering::SeqCst) as *mut c_void);
    let len = unsafe { GetModuleFileNameW(Some(hinst.into()), &mut buf) } as usize;
    (len > 0 && len < buf.len()).then(|| String::from_utf16_lossy(&buf[..len]))
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    match module_path().map(|p| registration::register(&p)) {
        Some(Ok(())) => S_OK,
        Some(Err(e)) => e.code(),
        None => E_FAIL,
    }
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    match registration::unregister() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }
}
