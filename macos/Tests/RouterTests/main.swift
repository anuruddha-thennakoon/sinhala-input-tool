// Run via macos/build.sh test. Exits non-zero on the first failure.
import Foundation

var failures = 0
func check(_ got: KeyAction, _ want: KeyAction, _ label: String) {
    if got != want {
        print("FAIL \(label): got \(got), want \(want)")
        failures += 1
    }
}
func r(_ code: UInt16, _ chars: String = "", cmd: Bool = false, ctrl: Bool = false,
       opt: Bool = false, composing: Bool) -> KeyAction {
    KeyRouter.route(keyCode: code, chars: chars, command: cmd, control: ctrl, option: opt, composing: composing)
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

if failures == 0 { print("KeyRouter: all checks passed") }
exit(failures == 0 ? 0 : 1)
