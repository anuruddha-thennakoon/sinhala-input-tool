use sinhala_core::{suggest, Dictionary};
use std::time::Instant;

const SMALL: &str = "නිලධාරි\t10\nනිලධාරී\t5000\nනිළධාරි\t1\nමම\t100\nමමත්\t5\nමමත\t2\n";

fn small() -> Dictionary<'static> {
    Dictionary::from_tsv(SMALL)
}

#[test]
fn much_more_frequent_word_beats_rule_order() {
    let got = suggest("niladhaari", &small(), 5);
    // නිලධාරී (freq 5000) beats නිලධාරි (freq 10) even though the rules prefer නිලධාරි.
    assert_eq!(&got[..3], ["නිලධාරී", "නිලධාරි", "නිළධාරි"]);
}

#[test]
fn similar_frequencies_keep_rule_order() {
    // Real counts from the word list: නිලධාරි 310, නිලධාරී 404. The rules' own best guess
    // wins when frequencies are close, which also matches the reference UI's first row.
    let d = Dictionary::from_tsv("නිලධාරි\t310\nනිලධාරී\t404\n");
    assert_eq!(&suggest("niladhaari", &d, 3)[..2], ["නිලධාරි", "නිලධාරී"]);
}

#[test]
fn prefix_completion_while_typing() {
    // "niladh" renders to the open prefix "නිලධ", completing to the dictionary words.
    let got = suggest("niladh", &small(), 5);
    assert_eq!(got[0], "නිලධාරී");
    assert!(got.contains(&"නිලධාරි".to_string()));
}

#[test]
fn completions_follow_exact_hits() {
    let got = suggest("mama", &small(), 5);
    assert_eq!(got[0], "මම");
    assert!(got.iter().position(|w| w == "මමත්").unwrap() > 0);
}

#[test]
fn rule_fallback_when_dictionary_misses() {
    let got = suggest("kopi", &small(), 5);
    assert_eq!(got[0], "කොපි");
}

#[test]
fn limit_empty_unique() {
    let d = small();
    assert!(suggest("", &d, 5).is_empty());
    assert!(suggest("mama", &d, 0).is_empty());
    let got = suggest("niladhaari", &d, 8);
    assert!(got.len() <= 8);
    let mut u = got.clone();
    u.sort();
    u.dedup();
    assert_eq!(u.len(), got.len());
}

#[test]
fn unsorted_input_is_handled() {
    let d = Dictionary::from_tsv("ම\t1\nඅ\t2\n");
    assert_eq!(d.freq("අ"), Some(2));
    assert_eq!(d.freq("ම"), Some(1));
    assert_eq!(d.freq("ක"), None);
}

#[test]
fn embedded_dictionary_is_sane() {
    let d = Dictionary::embedded();
    assert!(d.len() > 100_000);
    assert!(d.freq("සහ").unwrap() > 1000);
    assert_eq!(suggest("saha", d, 3)[0], "සහ");
    assert_eq!(suggest("athara", d, 3)[0], "අතර");
}

#[test]
fn embedded_short_inputs_prefer_the_plain_letter() {
    let d = Dictionary::embedded();
    for (typed, want) in [("ka", "ක"), ("i", "ඉ"), ("a", "අ"), ("u", "උ"), ("da", "ද"), ("ma", "ම")] {
        assert_eq!(suggest(typed, d, 3)[0], want, "typed {typed:?}");
    }
    // The reference UI's first candidate for niladhaari.
    assert_eq!(suggest("niladhaari", d, 3)[0], "නිලධාරි");
}

#[test]
fn embedded_lookup_is_fast() {
    let d = Dictionary::embedded(); // load once, outside the timing
    let n = 50;
    let t = Instant::now();
    for _ in 0..n {
        std::hint::black_box(suggest("niladhaari", d, 6));
    }
    let per_call = t.elapsed() / n;
    // Plan target is <5 ms in release; debug builds get generous headroom.
    assert!(per_call.as_millis() < 100, "{per_call:?} per call");
}
