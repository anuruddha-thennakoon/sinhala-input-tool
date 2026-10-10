use sinhala_core::transliterate;

fn top(input: &str) -> String {
    transliterate(input, 1)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// The exact candidate list shown in docs/ui-sample.png for `niladhaari`.
#[test]
fn screenshot_example_niladhaari() {
    assert_eq!(
        transliterate("niladhaari", 5),
        vec!["නිලධාරි", "නිලධාරී", "නිළධාරි", "නිළධාරී", "නිලාධාරි"]
    );
}

#[test]
fn words() {
    let cases = [
        ("mama", "මම"),
        ("amma", "අම්ම"),
        ("ammaa", "අම්මා"),
        ("akka", "අක්ක"),
        ("saha", "සහ"),
        ("eka", "එක"),
        ("oya", "ඔය"),
        ("mage", "මගෙ"),
        ("bath", "බත්"),
        ("paasala", "පාසල"),
        ("karuNaa", "කරුණා"),
        ("lankaa", "ලන්කා"),
        ("sinhala", "සින්හල"),
        ("thaaththaa", "තාත්තා"),
        ("dhaarmika", "ධාර්මික"),
        ("gnaanaa", "ඥානා"),
        ("angaa", "අඟා"),
    ];
    for (input, want) in cases {
        assert_eq!(top(input), want, "input: {input}");
    }
}

#[test]
fn conjuncts_rakaransaya_yansaya() {
    // consonant + ZWJ-joined ra / ya
    assert_eq!(top("graama"), "ග්\u{200D}රාම");
    assert_eq!(top("krama"), "ක්\u{200D}රම");
    assert_eq!(top("shrii"), "ශ්\u{200D}රී");
    assert_eq!(top("vidyaalaya"), "විද්\u{200D}යාලය");
    assert_eq!(top("ksha"), "ක්\u{200D}ෂ");
}

#[test]
fn marks_and_passthrough() {
    assert_eq!(top("kax"), "කං");
    assert_eq!(top("kaH"), "කඃ");
    assert_eq!(top("bath!"), "බත්!");
    assert_eq!(top("bath 1"), "බත් 1");
}

#[test]
fn independent_vowels() {
    let cases = [
        ("a", "අ"), ("aa", "ආ"), ("ae", "ඇ"), ("aae", "ඈ"), ("i", "ඉ"), ("ii", "ඊ"),
        ("u", "උ"), ("uu", "ඌ"), ("e", "එ"), ("ei", "ඒ"), ("o", "ඔ"), ("oe", "ඕ"),
        ("ai", "ඓ"), ("au", "ඖ"),
    ];
    for (input, want) in cases {
        assert_eq!(top(input), want, "input: {input}");
    }
}

#[test]
fn limit_empty_and_uniqueness() {
    assert!(transliterate("", 5).is_empty());
    assert!(transliterate("mama", 0).is_empty());
    assert_eq!(transliterate("niladhaari", 3).len(), 3);
    let all = transliterate("niladhaari", 30);
    let mut dedup = all.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(all.len(), dedup.len(), "candidates must be unique");
}

/// Every consonant x every vowel: the top candidate is base + matra.
/// Expected values are written out here independently of the engine's tables.
#[test]
fn consonant_vowel_matrix() {
    let consonants = [
        ("k", "ක"), ("kh", "ඛ"), ("g", "ග"), ("gh", "ඝ"), ("c", "ච"), ("ch", "ච"),
        ("chh", "ඡ"), ("j", "ජ"), ("jh", "ඣ"), ("T", "ට"), ("Th", "ඨ"), ("D", "ඩ"),
        ("Dh", "ඪ"), ("N", "ණ"), ("t", "ත"), ("th", "ත"), ("d", "ද"), ("dh", "ධ"),
        ("n", "න"), ("p", "ප"), ("ph", "ඵ"), ("b", "බ"), ("bh", "භ"), ("m", "ම"),
        ("y", "ය"), ("r", "ර"), ("l", "ල"), ("L", "ළ"), ("v", "ව"), ("w", "ව"),
        ("s", "ස"), ("sh", "ශ"), ("S", "ෂ"), ("Sh", "ෂ"), ("h", "හ"), ("f", "ෆ"),
        ("nd", "ඳ"), ("ng", "ඟ"), ("mb", "ඹ"), ("gn", "ඥ"),
    ];
    let vowels = [
        ("a", ""), ("aa", "\u{0DCF}"), ("ae", "\u{0DD0}"), ("aae", "\u{0DD1}"),
        ("i", "\u{0DD2}"), ("ii", "\u{0DD3}"), ("ee", "\u{0DD3}"), ("ei", "\u{0DDA}"),
        ("u", "\u{0DD4}"), ("uu", "\u{0DD6}"), ("oo", "\u{0DD6}"), ("e", "\u{0DD9}"),
        ("o", "\u{0DDC}"), ("oe", "\u{0DDD}"), ("ai", "\u{0DDB}"), ("au", "\u{0DDE}"),
    ];
    for (cl, cs) in consonants {
        for (vl, vs) in vowels {
            let input = format!("{cl}{vl}");
            assert_eq!(top(&input), format!("{cs}{vs}"), "input: {input}");
        }
    }
}
