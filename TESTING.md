# End-to-end test checklist

Run this on each platform after installing (see `README.md`). Note anything that differs and
send it back; each item is something automated tests cannot see.

## Install

- [ ] Installer runs (macOS: `macos/build.sh && macos/install.sh`, or the `.pkg`; Windows: the MSI from CI).
- [ ] "Sinhala" shows up as an input source / keyboard, with the blue "සි" icon.
- [ ] Switching to it works (mac: input menu; Windows: Win+Space).

## Typing (try in Word, then Notepad/TextEdit, then a browser text box)

- [ ] Typing `niladhaari` shows the popup next to the cursor with 1. නිලධාරි 2. නිලධාරී 3. නිළධාරි ... 6. niladhaari.
- [ ] The typed text is underlined in the document while composing.
- [ ] ↓ / ↑ move the highlight; PageDown / PageUp turn pages; the page arrows grey out at the ends.
- [ ] Space commits the highlighted word and adds a space. Enter commits without a newline.
- [ ] Keys 1–6 pick that row. 6 inserts the Latin text as typed.
- [ ] Backspace edits the typed text; Esc cancels and removes it.
- [ ] `graama`, `shrii`, `vidyaalaya`, `kaththa`, `ammaa` give sensible Sinhala (conjuncts look right).
- [ ] Everyday words: `oyaa`, `mokada`, `mama`, `eka`, `mage`, `lankaa`. Note any that are wrong or missing.
- [ ] Popup follows the cursor on a second line, near the bottom of the screen (it should flip above), and on a second monitor.
- [ ] Ctrl/Cmd shortcuts (copy, paste, undo) still work, also in the middle of a word.
- [ ] Clicking elsewhere mid-word commits the word instead of losing it (Windows).
- [ ] Typing fast does not drop letters or lag.

## Toggle and menu

- [ ] Ctrl+Shift+Space turns Sinhala typing off (letters type as English) and on again.
- [ ] macOS: a "Sinhala typing: ON/OFF" message appears near the cursor.
- [ ] The input menu (mac) / tray button (Windows) shows the state and has "About".
- [ ] The setting survives quitting and reopening the app, and a restart.
- [ ] About shows the version and the Wikipedia attribution.

## Uninstall

- [ ] macOS `macos/uninstall.sh`; Windows: uninstall "Sinhala Input" in Settings > Apps. The input source disappears after sign-out.

## Reporting a problem

- Say what you typed, what you expected, which app, and which OS version.
- Windows: create the empty file `%LOCALAPPDATA%\SinhalaInput\enable-log`, reproduce, and send `debug.log` from the same folder.
- macOS: Console.app > Crash Reports > `SinhalaInput`; or the Finder > `~/Library/Logs/DiagnosticReports`.
