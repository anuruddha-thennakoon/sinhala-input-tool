//! Ranking: dictionary hits first, then completions, then raw rule output.

use crate::dictionary::Dictionary;
use crate::translit::variants;

/// How many rule variants to consider per input.
const VARIANTS: usize = 40;
/// Rule variants whose open form is used for prefix completion.
const COMPLETION_VARIANTS: usize = 8;
const COMPLETIONS_PER_VARIANT: usize = 6;

/// How strongly the rules' own preference counts against word frequency. Each penalty
/// point costs a factor of e^1.5 (about 4.5x) in frequency, so a clearly more common
/// word still wins, but near-ties and noisy rare tokens (a lone ඊ in a list) do not
/// outrank the engine's best guess.
const RULE_WEIGHT: f64 = 1.5;

fn score(freq: u32, penalty: u32) -> f64 {
    (f64::from(freq) + 1.0).ln() - RULE_WEIGHT * f64::from(penalty)
}

/// Sorts by score (best first), then alphabetically so the order is deterministic.
fn rank(items: &mut [(&str, f64)]) {
    items.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
}

/// Ranked candidates for the Latin `input`, at most `limit`:
/// 1. exact dictionary words matching any rule variant;
/// 2. dictionary words that start with a variant (mid-word completion);
/// 3. the rule variants themselves, in rule order (covers names/unknown words).
///
/// Within 1 and 2, order is `ln(frequency) - RULE_WEIGHT * rule penalty`.
pub fn suggest(input: &str, dict: &Dictionary, limit: usize) -> Vec<String> {
    let vars = variants(input, VARIANTS);
    let mut out: Vec<String> = Vec::new();
    let push = |out: &mut Vec<String>, w: &str| {
        if !out.iter().any(|x| x == w) {
            out.push(w.to_string());
        }
    };

    let mut exact: Vec<(&str, f64)> = vars
        .iter()
        .filter_map(|v| dict.freq(&v.closed).map(|f| (v.closed.as_str(), score(f, v.penalty))))
        .collect();
    rank(&mut exact);
    for (w, _) in &exact {
        push(&mut out, w);
    }

    let mut completions: Vec<(&str, f64)> = Vec::new();
    for v in vars.iter().take(COMPLETION_VARIANTS) {
        completions.extend(
            dict.complete(&v.open, COMPLETIONS_PER_VARIANT)
                .into_iter()
                .map(|(w, f)| (w, score(f, v.penalty))),
        );
    }
    rank(&mut completions);
    for (w, _) in &completions {
        push(&mut out, w);
    }

    for v in &vars {
        push(&mut out, &v.closed);
    }
    out.truncate(limit);
    out
}
