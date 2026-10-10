// Run via macos/build.sh test. Exits non-zero on the first failure.
import Cocoa
import Foundation

var failures = 0
func check(_ got: KeyAction, _ want: KeyAction, _ label: String) {
    if got != want {
        print("FAIL \(label): got \(got), want \(want)")
        failures += 1
    }
}
func r(_ code: UInt16, _ chars: String = "", cmd: Bool = false, ctrl: Bool = false,
       opt: Bool = false, shift: Bool = false, composing: Bool, enabled: Bool = true) -> KeyAction {
    KeyRouter.route(keyCode: code, chars: chars, command: cmd, control: ctrl, option: opt,
                    shift: shift, composing: composing, enabled: enabled)
}

// Idle: only letters start composition.
check(r(0, "a", composing: false), .type("a"), "idle letter")
check(r(0, "N", composing: false), .type("N"), "idle capital")
check(r(49, " ", composing: false), .passThrough, "idle space")
check(r(18, "1", composing: false), .passThrough, "idle digit")
check(r(125, "", composing: false), .passThrough, "idle down")
check(r(51, "", composing: false), .passThrough, "idle backspace")
check(r(36, "\r", composing: false), .passThrough, "idle return")
check(r(53, "", composing: false), .passThrough, "idle escape")

// Composing: the Google-style keys.
check(r(0, "k", composing: true), .type("k"), "composing letter")
check(r(126, "", composing: true), .moveSelection(-1), "up")
check(r(125, "", composing: true), .moveSelection(1), "down")
check(r(116, "", composing: true), .turnPage(-1), "page up")
check(r(121, "", composing: true), .turnPage(1), "page down")
check(r(51, "", composing: true), .backspace, "backspace")
check(r(53, "", composing: true), .cancel, "escape")
check(r(49, " ", composing: true), .commitAndSpace, "space")
check(r(36, "\r", composing: true), .commit, "return")
check(r(76, "\u{3}", composing: true), .commit, "keypad enter")
for d in 1...6 { check(r(18, String(d), composing: true), .pickNumber(d), "digit \(d)") }
check(r(26, "7", composing: true), .commitThenPassThrough, "digit 7 passes")
check(r(29, "0", composing: true), .commitThenPassThrough, "digit 0 passes")
check(r(123, "", composing: true), .commitThenPassThrough, "left arrow")
check(r(43, ",", composing: true), .commitThenPassThrough, "comma")
check(r(48, "\t", composing: true), .commitThenPassThrough, "tab")

// Shortcuts are never swallowed.
check(r(8, "c", cmd: true, composing: false), .passThrough, "cmd-c idle")
check(r(8, "c", cmd: true, composing: true), .commitThenPassThrough, "cmd-c composing")
check(r(0, "a", ctrl: true, composing: false), .passThrough, "ctrl-a idle")
check(r(7, "x", opt: true, composing: true), .commitThenPassThrough, "option-x composing")

// Non-ASCII letters (other keyboard layouts) are not Singlish input.
check(r(0, "ä", composing: false), .passThrough, "non-ascii letter")

// Toggle key (Ctrl+Shift+Space) works in every state.
for composing in [false, true] {
    for enabled in [false, true] {
        check(r(49, " ", ctrl: true, shift: true, composing: composing, enabled: enabled),
              .toggleEnabled, "toggle composing=\(composing) enabled=\(enabled)")
    }
}
check(r(49, " ", ctrl: true, composing: true), .commitThenPassThrough, "ctrl-space is not the toggle")
check(r(49, " ", shift: true, composing: true), .commitAndSpace, "shift-space is not the toggle")
check(r(49, " ", cmd: true, ctrl: true, shift: true, composing: true), .commitThenPassThrough, "cmd-ctrl-shift-space is not the toggle")

// Disabled: everything except the toggle passes through untouched.
for composing in [false, true] {
    for (code, chars) in [(0 as UInt16, "a"), (49, " "), (36, "\r"), (51, ""), (125, ""), (18, "1"), (123, "")] {
        check(r(code, chars, composing: composing, enabled: false), .passThrough, "disabled key \(code) composing=\(composing)")
    }
}

// Caret rectangle validation (what the popup is allowed to be placed at).
func plausible(_ r: NSRect, _ expect: Bool, _ label: String) {
    if CaretLocator.plausible(r) != expect {
        print("FAIL caret \(label): expected plausible=\(expect)")
        failures += 1
    }
}
let screen = NSScreen.screens.first?.frame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
plausible(NSRect(x: screen.midX, y: screen.midY, width: 2, height: 18), true, "on-screen line")
plausible(.zero, false, "empty rect (Word sometimes returns this)")
plausible(NSRect(x: screen.midX, y: screen.midY, width: 2, height: 0), false, "zero height")
plausible(NSRect(x: screen.midX, y: screen.midY, width: 2, height: 5000), false, "absurd height")
plausible(NSRect(x: CGFloat.nan, y: 10, width: 2, height: 18), false, "NaN origin")
plausible(NSRect(x: CGFloat.infinity, y: 10, width: 2, height: 18), false, "infinite origin")
plausible(NSRect(x: screen.maxX + 5000, y: screen.midY, width: 2, height: 18), false, "far off screen")
plausible(NSRect(x: screen.minX - 20, y: screen.midY, width: 2, height: 18), true, "just off the edge is tolerated")

if failures == 0 { print("KeyRouter: all checks passed") }
exit(failures == 0 ? 0 : 1)
