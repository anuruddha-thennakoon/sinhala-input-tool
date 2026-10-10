//! Opt-in debug log. Enabled by creating `%LOCALAPPDATA%\SinhalaInput\enable-log`;
//! output goes to `%LOCALAPPDATA%\SinhalaInput\debug.log`. Off by default.

use std::io::Write;
use std::path::PathBuf;

fn dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("SinhalaInput"))
}

pub fn log(msg: impl AsRef<str>) {
    let Some(dir) = dir() else { return };
    if !dir.join("enable-log").exists() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("debug.log"))
    {
        let _ = writeln!(f, "[{}] {}", std::process::id(), msg.as_ref());
    }
}
