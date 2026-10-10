//! Singlish -> ranked Sinhala candidates.

use crate::tables::{lookup, Token};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

const HAL: char = '\u{0DCA}';
const ZWJ: char = '\u{200D}';

/// One rendering of the input. `open` is `closed` without the trailing
/// hal-kirima that a still-unvowelled last consonant gets, so it can be used
/// as a prefix while the user is mid-word.
pub(crate) struct Variant {
    pub closed: String,
    pub open: String,
}

/// Returns up to `limit` unique Sinhala renderings of `input`, best first.
pub fn transliterate(input: &str, limit: usize) -> Vec<String> {
    variants(input, limit).into_iter().map(|v| v.closed).collect()
}

pub(crate) fn variants(input: &str, limit: usize) -> Vec<Variant> {
    if input.is_empty() || limit == 0 {
        return Vec::new();
    }
    let tokens = tokenize(input);
    let mut heap: BinaryHeap<(Reverse<u32>, Vec<usize>)> = BinaryHeap::new();
    let mut seen: HashSet<Vec<usize>> = HashSet::new();
    let mut out: Vec<Variant> = Vec::new();

    push(&mut heap, &mut seen, &tokens, vec![0; tokens.len()]);

    // Options within a token have strictly increasing penalties, so every
    // choice vector is reached via cheaper ones and pops in penalty order.
    let mut budget = limit.saturating_mul(64);
    while let Some((_, mut choice)) = heap.pop() {
        if out.len() >= limit || budget == 0 {
            break;
        }
        budget -= 1;
        choice.reverse(); // heap stores it reversed for tie-breaking
        let (closed, pending) = render(&tokens, &choice);
        if !out.iter().any(|v| v.closed == closed) {
            let open = match closed.strip_suffix(HAL) {
                Some(stem) if pending => stem.to_string(),
                _ => closed.clone(),
            };
            out.push(Variant { closed, open });
        }
        for i in 0..choice.len() {
            if choice[i] + 1 < tokens[i].option_count() {
                let mut next = choice.clone();
                next[i] += 1;
                push(&mut heap, &mut seen, &tokens, next);
            }
        }
    }
    out
}

fn tokenize(input: &str) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let mut matched = None;
        for len in (1..=3).rev() {
            if i + len > chars.len() {
                continue;
            }
            let key: String = chars[i..i + len].iter().collect();
            if let Some(t) = lookup(&key).or_else(|| lookup(&key.to_lowercase())) {
                matched = Some((t, len));
                break;
            }
        }
        match matched {
            Some((t, len)) => {
                tokens.push(t);
                i += len;
            }
            None => {
                tokens.push(Token::Other(chars[i]));
                i += 1;
            }
        }
    }
    tokens
}

fn penalty(tokens: &[Token], choice: &[usize]) -> u32 {
    let last = tokens.len() - 1;
    tokens
        .iter()
        .zip(choice)
        .enumerate()
        .map(|(i, (t, &c))| match t {
            Token::Cons(o) => o[c].p,
            Token::Vowel(o) => {
                if i == last {
                    o[c].pf
                } else {
                    o[c].p
                }
            }
            _ => 0,
        })
        .sum()
}

/// Queue a choice vector. Ties on penalty prefer changes at later positions
/// (word endings are the most ambiguous), hence the reversed vector as key.
fn push(
    heap: &mut BinaryHeap<(Reverse<u32>, Vec<usize>)>,
    seen: &mut HashSet<Vec<usize>>,
    tokens: &[Token],
    choice: Vec<usize>,
) {
    if seen.insert(choice.clone()) {
        let pen = penalty(tokens, &choice);
        let mut rev = choice;
        rev.reverse();
        heap.push((Reverse(pen), rev));
    }
}

/// Returns the text and whether it ended on a consonant awaiting a vowel.
fn render(tokens: &[Token], choice: &[usize]) -> (String, bool) {
    let mut out = String::new();
    let mut pending = false; // a consonant is awaiting a vowel or hal-kirima
    for (tok, &c) in tokens.iter().zip(choice) {
        match tok {
            Token::Cons(opts) => {
                let s = opts[c].s;
                if pending {
                    out.push(HAL);
                    if s == "ර" || s == "ය" {
                        out.push(ZWJ); // rakaransaya / yansaya
                    }
                }
                out.push_str(s);
                pending = true;
            }
            Token::Vowel(opts) => {
                out.push_str(if pending { opts[c].mat } else { opts[c].ind });
                pending = false;
            }
            Token::Mark(m) => {
                out.push_str(m);
                pending = false;
            }
            Token::Other(ch) => {
                if pending {
                    out.push(HAL);
                    pending = false;
                }
                out.push(*ch);
            }
        }
    }
    if pending {
        out.push(HAL);
    }
    (out, pending)
}
