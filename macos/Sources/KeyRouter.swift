// Pure key -> action mapping. No AppKit dependency, so it is unit-testable.

enum KeyAction: Equatable {
    case passThrough
    case type(Character)
    case moveSelection(Int)
    case turnPage(Int)
    case backspace
    case commitAndSpace      // Space: commit highlighted candidate, then a space
    case commit              // Return/Enter: commit highlighted candidate only
    case pickNumber(Int)     // 1...6: commit that row of the current page
    case cancel              // Esc
    case commitThenPassThrough
}

enum KeyCodes {
    static let ret: UInt16 = 36
    static let space: UInt16 = 49
    static let delete: UInt16 = 51
    static let escape: UInt16 = 53
    static let keypadEnter: UInt16 = 76
    static let pageUp: UInt16 = 116
    static let pageDown: UInt16 = 121
    static let down: UInt16 = 125
    static let up: UInt16 = 126
}

enum KeyRouter {
    static func route(keyCode: UInt16, chars: String, command: Bool, control: Bool,
                      option: Bool, composing: Bool) -> KeyAction {
        // Shortcuts (Cmd-C, Ctrl-A, Option-x ...) belong to the app, not the IME.
        if command || control || option {
            return composing ? .commitThenPassThrough : .passThrough
        }
        if let c = chars.first, chars.count == 1, c.isASCII, c.isLetter {
            return .type(c)
        }
        guard composing else { return .passThrough }

        switch keyCode {
        case KeyCodes.up: return .moveSelection(-1)
        case KeyCodes.down: return .moveSelection(1)
        case KeyCodes.pageUp: return .turnPage(-1)
        case KeyCodes.pageDown: return .turnPage(1)
        case KeyCodes.delete: return .backspace
        case KeyCodes.escape: return .cancel
        case KeyCodes.space: return .commitAndSpace
        case KeyCodes.ret, KeyCodes.keypadEnter: return .commit
        default: break
        }
        if chars.count == 1, let d = chars.first?.wholeNumberValue, (1...6).contains(d),
           chars.first!.isASCII {
            return .pickNumber(d)
        }
        // Left/Right, punctuation, Tab, other digits: finish the word, then let the key through.
        return .commitThenPassThrough
    }
}
