//! Ranking: dictionary hits first, then completions, then raw rule output.

use crate::dictionary::Dictionary;
use crate::translit::variants;

/// How many rule variants to consider per input.
const VARIANTS: usize = 40;
/// Rule variants whose open form is used for prefix completion.
const COMPLETION_VARIANTS: usize = 8;
const COMPLETIONS_PER_VARIANT: usize = 6;

/// Ranked candidates for the Latin `input`, at most `limit`:
/// 1. exact dictionary words matching any rule variant, most frequent first;
/// 2. dictionary words that start with a variant (mid-word completion),
///    most frequent first;
/// 3. the rule variants themselves, in rule order (covers names/unknown words).
pub fn suggest(input: &str, dict: &Dictionary, limit: usize) -> Vec<String> {
    let vars = variants(input, VARIANTS);
    let mut out: Vec<String> = Vec::new();
    let push = |out: &mut Vec<String>, w: &str| {
        if !out.iter().any(|x| x == w) {
            out.push(w.to_string());
        }
    };

    let mut exact: Vec<(&str, u32)> = vars
        .iter()
        .filter_map(|v| dict.freq(&v.closed).map(|f| (v.closed.as_str(), f)))
        .collect();
    exact.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (w, _) in &exact {
        push(&mut out, w);
    }

    let mut completions: Vec<(&str, u32)> = Vec::new();
    for v in vars.iter().take(COMPLETION_VARIANTS) {
        completions.extend(dict.complete(&v.open, COMPLETIONS_PER_VARIANT));
    }
    completions.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (w, _) in &completions {
        push(&mut out, w);
    }

    for v in &vars {
        push(&mut out, &v.closed);
    }
    out.truncate(limit);
    out
}
