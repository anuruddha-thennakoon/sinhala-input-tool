# Sinhala Input Tool — Implementation Plan

## Status (2026-10-10)

| Phase | State |
|---|---|
| 0 Foundations | done |
| 1 Transliteration engine | done (mapping from knowledge of Google's behaviour; unverified against the real tool) |
| 2 Dictionary and ranking | done (Sinhala Wikipedia, 152k words) |
| 3 Session, paging, C ABI, harness | done |
| 4 macOS IME | built and launch-tested; not yet used in Word |
| 5 Windows IME | compiles for x64 and x86; CI registers and activates it; not yet used in Word |
| 6 Settings, toggle, tray, About | done in code; untested interactively |
| 7 Packaging | macOS universal .pkg built; Windows MSI verified only by CI; all unsigned |
| 8 Hardening | code-side done (panic guards, re-entrancy, bounded search, robustness tests); pending: your end-to-end testing in Word, then bug fixes |

## 1. Goal

A system-wide Sinhala input method (IME) in the style of Google Input Tools.
The user types romanized Sinhala ("Singlish"); a popup at the caret shows ranked
Sinhala suggestions; the user picks one with the arrow or number keys.

Source requirements (`requirements.txt`):
- Same as the Google Input Tool introduced for Windows 7.
- Installable on **Windows and macOS**.
- Works in **Microsoft Word**.
- Main purpose: type Sinhala, with **suggestions selectable by arrow keys**.
- UI must match `ui sample.png` (candidate popup).

## 2. Decisions made

| Topic | Decision |
|---|---|
| UI scope | Candidate popup only (plus minimal tray/settings). No website, no standalone editor. |
| Architecture | Native IME per OS + shared Rust core |
| Windows | Text Services Framework (TSF) text service |
| macOS | InputMethodKit (IMKit) input method, Swift |
| Suggestions | Offline: transliteration rules + bundled Sinhala word list with frequencies |
| Online/API suggestions | Out of scope |

Rejected: global key hook + overlay (cannot set composition text properly in Word);
Word-only add-in (not system-wide).

## 3. Popup UI spec (from screenshot)

- Floating box anchored below the caret.
- Top row: the typed Latin buffer (e.g. `niladhaari`) with a text cursor.
- Below: numbered list `1.`–`5.` of Sinhala candidates; `6.` is the raw Latin text.
- Row 1 highlighted by default; highlight moves with Up/Down.
- Paging arrows (up/down chevrons) at the bottom for more candidates.
- White background, light grey border, soft shadow, 1px-grey highlight fill.
- Sinhala-capable font (Iskoola Pota / Noto Sans Sinhala / Nirmala UI on Windows;
  Sinhala MN / Noto Sans Sinhala on macOS).
- Committed text appears inline in the document; the in-progress word is shown as
  composition (underlined) text in the host app.

## 4. Architecture

```
 Windows (TSF text service)      macOS (InputMethodKit app)
   ├─ key handling / composition    ├─ key handling / composition
   ├─ popup window (Win32)          ├─ popup window (AppKit)
   └─ calls ──────┐                 └─ calls ──────┐
                  ▼                                ▼
            sinhala-core (Rust, C ABI)
              ├─ transliterator: Singlish → Sinhala variants
              ├─ dictionary: word list + frequencies
              ├─ ranker: dictionary hits first, rule output as fallback
              └─ session: buffer, selection index, paging
```

### Core API (C ABI)
`session_new`, `push_char`, `backspace`, `candidates(page) -> list`,
`move_selection(delta)`, `select(n)`, `commit() -> string`, `cancel`, `session_free`.

### Key handling
- Letters: extend buffer. Up/Down: move highlight. `1`–`6`: pick directly.
- Space/Enter: commit highlighted candidate (Space also inserts a space after).
- Esc: cancel composition. Backspace: edit buffer. PageUp/PageDown or chevrons: paging.
- Any other key/punctuation: commit current selection, then pass the key through.

### Repo layout
```
sinhala-input-tool/
  core/            Rust crate (lib + C header via cbindgen)
  data/            word list sources + build script -> compact dictionary
  windows/         TSF text service + popup + installer (WiX)
  macos/           IMKit app + popup + pkg build
  docs/
  PLAN.md
```

## 5. Phases

Each phase ends with something runnable and verified.

### Phase 0 — Foundations
- Init git repo, workspace layout, CI skeleton (GitHub Actions: Windows + macOS runners).
- Pick Singlish scheme (target Google-style phonetic: `kk→ක්ක`, `aa→ආ`, `dh→ධ`, etc.).
- Choose dictionary source (open Sinhala corpus/wordlist); verify license.
- **Done when:** repo builds an empty core on both OSes in CI.

### Phase 1 — Core transliteration engine (Rust)
- Consonant/vowel/modifier tables, hal-kirima (්), rakaransaya, yansaya, special
  clusters (ක්‍ෂ, ඥ, ග්‍රා…).
- Generate multiple variants per input (e.g. ධ/ද, ළ/ල, ණ/න) as in `niladhaari`.
- Unit tests driven by a table of input → expected Sinhala, including the
  screenshot example.
- **Done when:** `cargo test` passes for a curated test table (≥200 cases).

### Phase 2 — Dictionary and ranking
- Build script: word list + frequencies → compact trie/FST file bundled in the core.
- Ranker: dictionary matches by frequency, then rule-generated variants, then raw Latin.
- Prefix completion for partially typed words.
- **Done when:** the `niladhaari` example yields the same top-5 style list; lookup
  latency < 5 ms on a laptop.

### Phase 3 — Session, selection, paging
- Buffer editing, selection index, paging (5 per page), number-key selection, commit/cancel.
- C ABI + generated header; a small CLI harness to drive it from the terminal.
- **Done when:** the CLI harness reproduces the full key flow end to end.

### Phase 4 — macOS IME
- IMKit app bundle, `Info.plist` input source registration (Sinhala).
- Composition string via `IMKInputController`; candidate popup in AppKit matching spec §3.
- Test in TextEdit, Pages, **Microsoft Word for Mac**, Safari/Chrome.
- **Done when:** typing works in Word for Mac with arrow-key selection.

### Phase 5 — Windows IME
- TSF text service DLL (COM), registration (profile, language `si-LK`), composition
  handling via `ITfContext`/edit sessions.
- Candidate popup window positioned from `ITfContextView::GetTextExt`, matching spec §3.
- Test in Notepad, WordPad, **Microsoft Word**, Edge/Chrome, plus 32-bit and 64-bit hosts.
- **Done when:** typing works in Word on Windows 10/11 with arrow-key selection.

### Phase 6 — Settings and tray
- Minimal: enable/disable, Sinhala ↔ English toggle (hotkey), About.
- **Done when:** toggle persists across restarts on both OSes.

### Phase 7 — Packaging and distribution
- Windows: WiX/MSI installer, registers/unregisters TSF, code-signed.
- macOS: `.pkg` to `/Library/Input Methods`, signed and notarized.
- Uninstallers; install/uninstall verified on clean VMs.
- **Done when:** clean-machine install works and the IME appears in the OS input list.

### Phase 8 — Hardening
- Compatibility pass (Word, Excel, PowerPoint, browsers, terminals), high-DPI,
  multi-monitor popup placement, dark mode, performance, crash safety.
- User docs and a short install guide.

## 6. Testing strategy
- Core: unit tests plus a golden-file table of transliterations.
- Shells: manual checklist per host app (Word first), plus scripted UI tests where possible.
- CI: build and test on Windows and macOS runners for every push.

## 7. Risks
- **TSF complexity** — the biggest risk; mitigate by prototyping composition in Notepad first.
- **Word on Mac IME quirks** — test early (Phase 4), not at the end.
- **Dictionary quality/licence** — decide the source in Phase 0.
- **Signing/notarization** — needs an Apple Developer account and a Windows code-signing cert.
- **Fonts** — ensure fallback if no Sinhala font is installed.

## 8. Open items
- Exact Singlish mapping table (Google-compatible vs. Wikipedia/Sri Lankan standard).
- Which Windows versions to support (suggest 10 and 11; Windows 7 is EOL).
- Product name, icon, and licence.
- Apple Developer account and Windows signing certificate availability.
