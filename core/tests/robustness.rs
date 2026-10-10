//! The core runs inside other people's apps: it must never panic and never be slow,
//! whatever is typed. Inputs here are deliberately hostile.
use sinhala_core::{suggest, transliterate, Dictionary, Session};
use std::time::{Duration, Instant};

/// Tiny deterministic generator so failures are reproducible without a dependency.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

fn random_string(rng: &mut Lcg, len: usize, alphabet: &[char]) -> String {
    (0..len).map(|_| alphabet[rng.next() as usize % alphabet.len()]).collect()
}

#[test]
fn random_latin_never_panics() {
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let mut rng = Lcg(1);
    let dict = Dictionary::embedded();
    for _ in 0..500 {
        let len = 1 + rng.next() as usize % 24;
        let word = random_string(&mut rng, len, &alphabet);
        let out = suggest(&word, dict, 30);
        assert!(!out.is_empty(), "no candidates for {word:?}");
        assert!(out.iter().all(|c| !c.is_empty()));
    }
}

#[test]
fn hostile_unicode_never_panics() {
    let alphabet: Vec<char> = "a\u{0}\u{7f}ක්\u{200d}\u{200b}é😀日本\u{301}\r\n\t 123!?'\"\\{}[]ZzNnTtDdLlSs"
        .chars()
        .collect();
    let mut rng = Lcg(2);
    for _ in 0..500 {
        let len = 1 + rng.next() as usize % 16;
        let word = random_string(&mut rng, len, &alphabet);
        let _ = transliterate(&word, 10);
        let _ = suggest(&word, Dictionary::embedded(), 10);
        let mut s = Session::new(Dictionary::embedded());
        for c in word.chars() {
            s.push_char(c);
        }
        let _ = s.view();
        let _ = s.commit();
    }
}

#[test]
fn only_non_letters_is_fine() {
    for input in ["1", "!!!", "   ", "\u{200d}", "😀", "123 456"] {
        let out = transliterate(input, 5);
        assert_eq!(out.len(), 1, "{input:?}");
        assert_eq!(out[0], input, "non-letters pass through unchanged");
    }
}

#[test]
fn very_long_input_stays_fast() {
    let dict = Dictionary::embedded();
    for len in [200, 2000] {
        let word = "kaththa".repeat(len / 7);
        let t = Instant::now();
        let out = suggest(&word, dict, 30);
        assert!(!out.is_empty());
        assert!(t.elapsed() < Duration::from_millis(500), "len {len} took {:?}", t.elapsed());
    }
}

#[test]
fn long_typing_session_stays_fast() {
    let mut s = Session::new(Dictionary::embedded());
    let t = Instant::now();
    for c in "niladhaarii".repeat(20).chars() {
        s.push_char(c); // recomputes candidates on every key
        let _ = s.view();
    }
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
}

#[test]
fn session_survives_arbitrary_key_sequences() {
    let mut rng = Lcg(3);
    let mut s = Session::new(Dictionary::embedded());
    for _ in 0..5000 {
        match rng.next() % 9 {
            0..=3 => s.push_char((b'a' + (rng.next() % 26) as u8) as char),
            4 => {
                s.backspace();
            }
            5 => s.move_selection(rng.next() as i32 % 40 - 20),
            6 => s.turn_page(rng.next() as i32 % 10 - 5),
            7 => {
                let _ = s.select_number(rng.next() as usize % 9);
            }
            _ => {
                let _ = s.commit();
            }
        }
        let v = s.view();
        assert!(v.selected <= v.rows.len(), "selection out of range: {v:?}");
        assert!(v.page < v.page_count.max(1));
    }
}

#[test]
fn extreme_deltas_do_not_overflow() {
    let mut s = Session::new(Dictionary::embedded());
    for c in "ka".chars() {
        s.push_char(c);
    }
    s.move_selection(i32::MAX);
    s.move_selection(i32::MIN);
    s.turn_page(i32::MAX);
    s.turn_page(i32::MIN);
    assert_eq!(s.view().page, 0);
}
