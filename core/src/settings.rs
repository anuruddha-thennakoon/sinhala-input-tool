//! User settings, shared by both platforms: a tiny `key=value` file.
//!
//! Unknown keys and garbage are ignored and a missing or unreadable file means
//! defaults, so a bad settings file can never stop someone from typing.

use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Sinhala typing on. When off, the input method passes every key through.
    pub enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { enabled: true }
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "enabled" {
                    match value.trim() {
                        "true" => s.enabled = true,
                        "false" => s.enabled = false,
                        _ => {}
                    }
                }
            }
        }
        s
    }

    pub fn to_ini(&self) -> String {
        format!("enabled={}\n", self.enabled)
    }

    /// Reads settings from `path`; defaults if the file is missing or unreadable.
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .map(|t| Settings::parse(&t))
            .unwrap_or_default()
    }

    /// Writes settings to `path`, creating parent directories. The write goes to a
    /// temporary file first and is renamed into place, so a crash cannot leave a
    /// half-written file.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("ini.tmp");
        std::fs::write(&tmp, self.to_ini())?;
        std::fs::rename(&tmp, path)
    }
}

/// Text for the About box on both platforms. Carries the attribution that the
/// CC BY-SA 4.0 licence of the bundled word list requires.
pub fn about_text() -> String {
    format!(
        "Sinhala Input {}\n\n\
         Type Sinhala phonetically (Singlish) and pick suggestions with the arrow keys.\n\n\
         Toggle Sinhala typing: Ctrl+Shift+Space\n\n\
         Word list derived from Sinhala Wikipedia (https://si.wikipedia.org), text by \
         Wikipedia contributors, licensed under CC BY-SA 4.0 \
         (https://creativecommons.org/licenses/by-sa/4.0/). This derived list is shared \
         under the same licence.",
        env!("CARGO_PKG_VERSION")
    )
}
