//! Singlish -> Sinhala mapping tables (Google Input Tools style phonetic scheme).
//!
//! Each Latin token maps to a ranked list of options. Within a list, penalties
//! must be strictly increasing: the search in `translit` relies on it.

/// Consonant option: Sinhala text ending in a base consonant, plus its penalty.
pub struct COpt {
    pub s: &'static str,
    pub p: u32,
}

/// Vowel option: independent form, matra (dependent sign), penalty, and the
/// penalty when the vowel is the last token of the word (`pf`).
pub struct VOpt {
    pub ind: &'static str,
    pub mat: &'static str,
    pub p: u32,
    pub pf: u32,
}

#[derive(Clone, Copy)]
pub enum Token {
    Cons(&'static [COpt]),
    Vowel(&'static [VOpt]),
    /// Anusvara / visarga: attaches directly, no hal-kirima.
    Mark(&'static str),
    /// Anything else (digits, punctuation, spaces): passed through.
    Other(char),
}

impl Token {
    pub fn option_count(&self) -> usize {
        match self {
            Token::Cons(o) => o.len(),
            Token::Vowel(o) => o.len(),
            _ => 1,
        }
    }
}

macro_rules! cons {
    ($(($s:expr, $p:expr)),+ $(,)?) => {
        Some(Token::Cons(&[$(COpt { s: $s, p: $p }),+]))
    };
}

macro_rules! vow {
    ($(($i:expr, $m:expr, $p:expr, $pf:expr)),+ $(,)?) => {
        Some(Token::Vowel(&[$(VOpt { ind: $i, mat: $m, p: $p, pf: $pf }),+]))
    };
}

// Matras:  ා \u{0DCF}  ැ \u{0DD0}  ෑ \u{0DD1}  ි \u{0DD2}  ී \u{0DD3}  ු \u{0DD4}
//          ූ \u{0DD6}  ෙ \u{0DD9}  ේ \u{0DDA}  ෛ \u{0DDB}  ො \u{0DDC}  ෝ \u{0DDD}  ෞ \u{0DDE}
// Hal-kirima ් \u{0DCA}, ZWJ \u{200D}.

/// Exact-case lookup of one Latin token (1 to 3 chars).
pub fn lookup(key: &str) -> Option<Token> {
    match key {
        // --- consonant clusters / digraphs ---
        "ksh" => cons!(("ක්\u{200D}ෂ", 0)),
        "chh" => cons!(("ඡ", 0)),
        "ndh" => cons!(("න්ධ", 0)),
        "kh" => cons!(("ඛ", 0)),
        "gh" => cons!(("ඝ", 0)),
        "ch" => cons!(("ච", 0), ("ඡ", 3)),
        "jh" => cons!(("ඣ", 0)),
        "Th" => cons!(("ඨ", 0)),
        "Dh" => cons!(("ඪ", 0)),
        "th" => cons!(("ත", 0), ("ථ", 2)),
        "dh" => cons!(("ධ", 0)),
        "ph" => cons!(("ඵ", 0), ("ෆ", 3)),
        "bh" => cons!(("භ", 0)),
        "sh" => cons!(("ශ", 0), ("ෂ", 2)),
        "Sh" => cons!(("ෂ", 0)),
        "nd" => cons!(("ඳ", 0), ("න්ද", 2)),
        "ng" => cons!(("ඟ", 0), ("න්ග", 2)),
        "mb" => cons!(("ඹ", 0), ("ම්බ", 2)),
        "gn" => cons!(("ඥ", 0)),
        // --- single consonants ---
        "k" => cons!(("ක", 0)),
        "g" => cons!(("ග", 0)),
        "c" => cons!(("ච", 0)),
        "j" => cons!(("ජ", 0)),
        "T" => cons!(("ට", 0)),
        "D" => cons!(("ඩ", 0)),
        "N" => cons!(("ණ", 0)),
        "t" => cons!(("ත", 0), ("ට", 2)),
        "d" => cons!(("ද", 0), ("ඩ", 2)),
        "n" => cons!(("න", 0), ("ණ", 5)),
        "p" => cons!(("ප", 0)),
        "b" => cons!(("බ", 0)),
        "m" => cons!(("ම", 0)),
        "y" => cons!(("ය", 0)),
        "r" => cons!(("ර", 0)),
        "l" => cons!(("ල", 0), ("ළ", 1)),
        "L" => cons!(("ළ", 0)),
        "v" | "w" => cons!(("ව", 0)),
        "s" => cons!(("ස", 0), ("ශ", 3), ("ෂ", 4)),
        "S" => cons!(("ෂ", 0)),
        "h" => cons!(("හ", 0)),
        "f" => cons!(("ෆ", 0)),
        "G" => cons!(("ඟ", 0)),
        "B" => cons!(("ඹ", 0)),
        // --- vowels: (independent, matra, penalty, penalty-if-word-final) ---
        "aae" => vow!(("ඈ", "\u{0DD1}", 0, 0)),
        "aa" => vow!(("ආ", "\u{0DCF}", 0, 0)),
        "ae" => vow!(("ඇ", "\u{0DD0}", 0, 0), ("ඈ", "\u{0DD1}", 3, 3)),
        "au" => vow!(("ඖ", "\u{0DDE}", 0, 0)),
        "ai" => vow!(("ඓ", "\u{0DDB}", 0, 0)),
        "ii" => vow!(("ඊ", "\u{0DD3}", 0, 0)),
        "ee" => vow!(("ඊ", "\u{0DD3}", 0, 0), ("ඒ", "\u{0DDA}", 2, 2)),
        "ei" => vow!(("ඒ", "\u{0DDA}", 0, 0)),
        "uu" => vow!(("ඌ", "\u{0DD6}", 0, 0)),
        "oo" => vow!(("ඌ", "\u{0DD6}", 0, 0), ("ඕ", "\u{0DDD}", 2, 2)),
        "oe" => vow!(("ඕ", "\u{0DDD}", 0, 0)),
        "a" => vow!(("අ", "", 0, 0), ("ආ", "\u{0DCF}", 3, 3)),
        "i" => vow!(("ඉ", "\u{0DD2}", 0, 0), ("ඊ", "\u{0DD3}", 4, 1)),
        "u" => vow!(("උ", "\u{0DD4}", 0, 0), ("ඌ", "\u{0DD6}", 4, 1)),
        "e" => vow!(("එ", "\u{0DD9}", 0, 0), ("ඒ", "\u{0DDA}", 4, 2)),
        "o" => vow!(("ඔ", "\u{0DDC}", 0, 0), ("ඕ", "\u{0DDD}", 4, 2)),
        // --- marks ---
        "x" => Some(Token::Mark("\u{0D82}")),
        "H" => Some(Token::Mark("\u{0D83}")),
        _ => None,
    }
}
