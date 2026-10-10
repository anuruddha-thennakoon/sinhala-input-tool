//! Settings file location and access (`%APPDATA%\SinhalaInput\settings.ini`).

use sinhala_core::Settings;
use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("SinhalaInput").join("settings.ini"))
}

pub fn load_enabled() -> bool {
    path().is_none_or(|p| Settings::load(&p).enabled)
}

pub fn save_enabled(enabled: bool) {
    if let Some(p) = path() {
        let _ = Settings { enabled }.save(&p);
    }
}
