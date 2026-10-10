# Sinhala Input

A Sinhala input method for Windows and macOS in the style of Google Input Tools. Type Sinhala
phonetically ("Singlish"), and suggestions appear next to the cursor; choose with the arrow keys.
Works in any app, including Microsoft Word.

    niladhaari  ->  1. නිලධාරී   2. නිලධාරි   3. නිළධාරි   4. නිළධාරී   5. නිලධාරීන්   6. niladhaari

## Using it

| Key | Action |
|---|---|
| letters | type Singlish (`ammaa` -> අම්මා, `graama` -> ග්‍රාම, `shrii` -> ශ්‍රී) |
| ↑ / ↓ | move through suggestions (continues onto the next page) |
| PageUp / PageDown | previous / next page of suggestions |
| 1 – 6 | pick that row (6 is the text exactly as typed) |
| Space | commit the highlighted suggestion and add a space |
| Enter | commit the highlighted suggestion |
| Backspace / Esc | edit the typed text / cancel |
| Ctrl+Shift+Space | turn Sinhala typing on or off (without switching keyboards) |

Capital letters give retroflex sounds: `T D N L S` -> ට ඩ ණ ළ ෂ. `x` -> ං, `H` -> ඃ.

## Install

Both installers are **unsigned**, so the OS will warn about an unknown publisher.

**macOS** (Apple Silicon and Intel, macOS 12+)
1. Build or download `SinhalaInput-<version>.pkg` and run it (asks for your password), or from a
   checkout run `macos/build.sh && macos/install.sh` for a per-user install.
2. If the pkg warns it is from an unidentified developer: right-click it, Open.
3. System Settings > Keyboard > Input Sources > Edit > + > Sinhala > "Sinhala". Log out and in if it is not listed.

**Windows 10 / 11**
1. Run `SinhalaInput-<version>-x64.msi`. If SmartScreen warns: More info > Run anyway.
2. Settings > Time & language > Language & region > add Sinhala, then enable the "Sinhala Input" keyboard.
3. Switch to it with Win+Space.

To remove: macOS `macos/uninstall.sh`; Windows uninstall "Sinhala Input" in Settings > Apps.

## Building from source

| Part | Where | Build |
|---|---|---|
| Transliteration, suggestions, sessions (Rust, C ABI) | `core/` | `cargo test --workspace` |
| Word list (Sinhala Wikipedia) | `data/` | `data/build_dictionary.py` |
| macOS input method (Swift, InputMethodKit) | `macos/` | `macos/build.sh`, `macos/package.sh` |
| Windows text service (Rust, TSF) | `windows/` | see `windows/README.md`; MSI in `windows/installer/` |

Pushing a tag such as `v0.1.0` builds both installers in CI and attaches them to a GitHub release.
See `PLAN.md` for the design and phases.

## Licence

The code is MIT-licensed (see `LICENSE`).

## Credits and licence of the word list

`core/data/si-words.tsv` is derived from the Sinhala Wikipedia (text by Wikipedia contributors,
CC BY-SA 4.0) and is shared under the same licence. See `core/data/ATTRIBUTION.md`.
