use sinhala_core::{suggest, Dictionary};
use std::time::Instant;

const SMALL: &str = "නිලධාරි\t10\nනිලධාරී\t50\nනිළධාරි\t1\nමම\t100\nමමත්\t5\nමමත\t2\n";

fn small() -> Dictionary<'static> {
    Dictionary::from_tsv(SMALL)
}

#[test]
fn exact_hits_rank_by_frequency_not_rule_order() {
    let got = suggest("niladhaari", &small(), 5);
    // නිලධාරී (freq 50) beats නිලධාරි (freq 10) even though the rules prefer නිලධාරි.
    assert_eq!(&got[..3], ["නිලධාරී", "නිලධාරි", "නිළධාරි"]);
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
