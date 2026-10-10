//! regsvr32 support: COM class registration plus the TSF language profile and
//! categories. Writes to HKLM, so it needs an elevated process.

use crate::guids::*;
use windows::core::{Result, GUID, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Registry::*;
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::*;

fn clsid_string() -> String {
    format!("{{{:?}}}", CLSID_SINHALA_TIP).to_uppercase()
}

fn wide_z(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn set_default(key: HKEY, name: Option<&str>, value: &str) -> Result<()> {
    let data: Vec<u8> = wide_z(value).iter().flat_map(|w| w.to_le_bytes()).collect();
    let name = name.map(wide_z);
    let name_ptr = name.as_ref().map_or(PCWSTR::null(), |n| PCWSTR(n.as_ptr()));
    unsafe { RegSetValueExW(key, name_ptr, None, REG_SZ, Some(&data)).ok() }
}

fn create_key(path: &str) -> Result<HKEY> {
    let mut key = HKEY::default();
    let path = wide_z(path);
    unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(path.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
        .ok()?;
    }
    Ok(key)
}

pub fn register(dll_path: &str) -> Result<()> {
    let base = format!("SOFTWARE\\Classes\\CLSID\\{}", clsid_string());
    let key = create_key(&base)?;
    set_default(key, None, TIP_DESCRIPTION)?;
    let server = create_key(&format!("{base}\\InprocServer32"))?;
    set_default(server, None, dll_path)?;
    set_default(server, Some("ThreadingModel"), "Apartment")?;
    unsafe {
        let _ = RegCloseKey(server);
        let _ = RegCloseKey(key);

        let profiles: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let desc: Vec<u16> = TIP_DESCRIPTION.encode_utf16().collect();
        // The installer puts sinhala.ico next to the DLLs; without it the profile just has no icon.
        let icon: Vec<u16> = std::path::Path::new(dll_path)
            .parent()
            .map(|dir| dir.join("sinhala.ico"))
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().encode_utf16().collect())
            .unwrap_or_default();
        profiles.RegisterProfile(
            &CLSID_SINHALA_TIP,
            LANGID_SINHALA,
            &GUID_SINHALA_PROFILE,
            &desc,
            &icon,
            0,
            HKL::default(),
            0,
            true,
            0,
        )?;

        let categories: ITfCategoryMgr = CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for cat in categories_to_register() {
            categories.RegisterCategory(&CLSID_SINHALA_TIP, &cat, &CLSID_SINHALA_TIP)?;
        }
    }
    Ok(())
}

pub fn unregister() -> Result<()> {
    unsafe {
        // Best effort: keep going so a half-installed state can still be cleaned up.
        if let Ok(profiles) =
            CoCreateInstance::<_, ITfInputProcessorProfileMgr>(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)
        {
            let _ = profiles.UnregisterProfile(&CLSID_SINHALA_TIP, LANGID_SINHALA, &GUID_SINHALA_PROFILE, 0);
        }
        if let Ok(categories) = CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER) {
            for cat in categories_to_register() {
                let _ = categories.UnregisterCategory(&CLSID_SINHALA_TIP, &cat, &CLSID_SINHALA_TIP);
            }
        }
        let path = wide_z(&format!("SOFTWARE\\Classes\\CLSID\\{}", clsid_string()));
        let status = RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(path.as_ptr()));
        if status != ERROR_SUCCESS && status.0 != 2 {
            // 2 = ERROR_FILE_NOT_FOUND: already gone, fine.
            status.ok()?;
        }
    }
    Ok(())
}

fn categories_to_register() -> [GUID; 3] {
    [GUID_TFCAT_TIP_KEYBOARD, GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER]
}
