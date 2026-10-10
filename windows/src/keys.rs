//! Virtual-key -> action mapping. Pure logic with no Windows dependency, so it is
//! unit-tested on any OS. Mirrors macos/Sources/KeyRouter.swift.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    PassThrough,
    Type(char),
    MoveSelection(i32),
    TurnPage(i32),
    Backspace,
    /// Space: commit the highlighted candidate, then insert a space.
    CommitAndSpace,
    /// Return: commit the highlighted candidate only.
    Commit,
    /// 1..=6: commit that row of the current page.
    PickNumber(u32),
    Cancel,
    CommitThenPassThrough,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps: bool,
}

pub mod vk {
    pub const BACK: u32 = 0x08;
    pub const RETURN: u32 = 0x0D;
    pub const ESCAPE: u32 = 0x1B;
    pub const SPACE: u32 = 0x20;
    pub const PRIOR: u32 = 0x21; // Page Up
    pub const NEXT: u32 = 0x22; // Page Down
    pub const UP: u32 = 0x26;
    pub const DOWN: u32 = 0x28;
}

pub fn route(vk_code: u32, mods: Mods, composing: bool) -> KeyAction {
    // Shortcuts (Ctrl-C, Alt-F4, AltGr combos) belong to the app, not the IME.
    if mods.ctrl || mods.alt {
        return if composing {
            KeyAction::CommitThenPassThrough
        } else {
            KeyAction::PassThrough
        };
    }
    if (0x41..=0x5A).contains(&vk_code) {
        let c = vk_code as u8 as char;
        let upper = mods.shift ^ mods.caps;
        return KeyAction::Type(if upper { c } else { c.to_ascii_lowercase() });
    }
    if !composing {
        return KeyAction::PassThrough;
    }
    match vk_code {
        vk::UP => return KeyAction::MoveSelection(-1),
        vk::DOWN => return KeyAction::MoveSelection(1),
        vk::PRIOR => return KeyAction::TurnPage(-1),
        vk::NEXT => return KeyAction::TurnPage(1),
        vk::BACK => return KeyAction::Backspace,
        vk::ESCAPE => return KeyAction::Cancel,
        vk::SPACE => return KeyAction::CommitAndSpace,
        vk::RETURN => return KeyAction::Commit,
        _ => {}
    }
    // Digits 1-6 pick a row (top row, unshifted, or numpad).
    let digit = match vk_code {
        0x31..=0x36 if !mods.shift => Some(vk_code - 0x30),
        0x61..=0x66 => Some(vk_code - 0x60),
        _ => None,
    };
    if let Some(d) = digit {
        return KeyAction::PickNumber(d);
    }
    // Left/Right, punctuation, Tab, other digits: finish the word, let the key through.
    KeyAction::CommitThenPassThrough
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyAction::*;

    fn r(vk: u32, composing: bool) -> KeyAction {
        route(vk, Mods::default(), composing)
    }

    #[test]
    fn idle_only_letters_start_composition() {
        assert_eq!(r(0x41, false), Type('a'));
        assert_eq!(r(0x5A, false), Type('z'));
        assert_eq!(r(vk::SPACE, false), PassThrough);
        assert_eq!(r(0x31, false), PassThrough);
        assert_eq!(r(vk::DOWN, false), PassThrough);
        assert_eq!(r(vk::BACK, false), PassThrough);
        assert_eq!(r(vk::RETURN, false), PassThrough);
        assert_eq!(r(vk::ESCAPE, false), PassThrough);
    }

    #[test]
    fn case_follows_shift_xor_caps() {
        let m = |shift, caps| Mods { shift, caps, ..Mods::default() };
        assert_eq!(route(0x4E, m(true, false), false), Type('N'));
        assert_eq!(route(0x4E, m(false, true), false), Type('N'));
        assert_eq!(route(0x4E, m(true, true), false), Type('n'));
        assert_eq!(route(0x4E, m(false, false), false), Type('n'));
    }

    #[test]
    fn composing_keys() {
        assert_eq!(r(0x4B, true), Type('k'));
        assert_eq!(r(vk::UP, true), MoveSelection(-1));
        assert_eq!(r(vk::DOWN, true), MoveSelection(1));
        assert_eq!(r(vk::PRIOR, true), TurnPage(-1));
        assert_eq!(r(vk::NEXT, true), TurnPage(1));
        assert_eq!(r(vk::BACK, true), Backspace);
        assert_eq!(r(vk::ESCAPE, true), Cancel);
        assert_eq!(r(vk::SPACE, true), CommitAndSpace);
        assert_eq!(r(vk::RETURN, true), Commit);
    }

    #[test]
    fn number_keys_pick_rows_one_to_six() {
        for d in 1..=6u32 {
            assert_eq!(r(0x30 + d, true), PickNumber(d));
            assert_eq!(r(0x60 + d, true), PickNumber(d)); // numpad
        }
        assert_eq!(r(0x37, true), CommitThenPassThrough);
        assert_eq!(r(0x30, true), CommitThenPassThrough);
        assert_eq!(r(0x67, true), CommitThenPassThrough);
    }

    #[test]
    fn shifted_digits_are_punctuation_not_picks() {
        let shift = Mods { shift: true, ..Mods::default() };
        assert_eq!(route(0x31, shift, true), CommitThenPassThrough); // '!'
    }

    #[test]
    fn other_keys_finish_the_word_and_pass_through() {
        assert_eq!(r(0x25, true), CommitThenPassThrough); // Left
        assert_eq!(r(0x27, true), CommitThenPassThrough); // Right
        assert_eq!(r(0x09, true), CommitThenPassThrough); // Tab
        assert_eq!(r(0xBC, true), CommitThenPassThrough); // ','
    }

    #[test]
    fn shortcuts_are_never_swallowed() {
        let ctrl = Mods { ctrl: true, ..Mods::default() };
        let alt = Mods { alt: true, ..Mods::default() };
        assert_eq!(route(0x43, ctrl, false), PassThrough);
        assert_eq!(route(0x43, ctrl, true), CommitThenPassThrough);
        assert_eq!(route(0x46, alt, false), PassThrough);
        assert_eq!(route(vk::SPACE, ctrl, true), CommitThenPassThrough);
    }
}
