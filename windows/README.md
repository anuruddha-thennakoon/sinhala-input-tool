# Windows text service (TSF)

Rust cdylib `sinhala_tsf.dll`: a Text Services Framework input method that links the
`sinhala-core` library directly.

- `src/keys.rs`     virtual-key -> action routing (pure, unit-tested on any OS)
- `src/service.rs`  the text service: key events, composition, edit sessions
- `src/popup.rs`    GDI candidate popup (Google Input Tools style)
- `src/langbar.rs`  tray / language-bar button: on-off state, menu, About
- `src/config.rs`   settings file (`%APPDATA%\SinhalaInput\settings.ini`)
- `src/dll.rs`      COM entry points + class factory
- `src/registration.rs`  regsvr32: CLSID, TSF profile (si-LK), categories
- `exports.def`     undecorated exports (needed for the 32-bit build)

## Build

    cargo build --release -p sinhala-tsf --target x86_64-pc-windows-msvc
    cargo build --release -p sinhala-tsf --target i686-pc-windows-msvc   # 32-bit apps

## Install for testing (elevated PowerShell)

    powershell -ExecutionPolicy Bypass -File windows\install.ps1

Then add Sinhala under Settings > Time & language > Language & region, enable the
"Sinhala Input" keyboard, and switch to it with Win+Space. Remove with `uninstall.ps1`.

## Settings and toggle

Ctrl+Shift+Space turns Sinhala typing on or off without switching keyboards (when off, every
key passes straight through). The tray / language-bar button shows the state ("සි" on blue = on,
"EN" on grey = off); its menu has the same switch and an About box. The state is saved in
`%APPDATA%\SinhalaInput\settings.ini` and re-read when a window gains focus, so it is shared
between apps and survives restarts.

## Debug log

Create `%LOCALAPPDATA%\SinhalaInput\enable-log` (an empty file); key events and lifecycle
calls are appended to `%LOCALAPPDATA%\SinhalaInput\debug.log`. Delete the file to turn it off.

## Known gaps (Phase 5)

- Not verified on a real Windows machine yet; CI only checks build + register/activate/unregister.
- Composition text is not underlined (no display-attribute provider yet).
- Plain popup window rather than TSF candidate-list UI elements, so Store/UWP apps and the
  Windows secure desktop will not show it.
- No installer icon (Phase 7). No on-screen toast on toggle; the tray icon is the indicator.
- Toggling from the tray menu abandons a word in progress (no document context to commit into).
