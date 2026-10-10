//! Word-frequency dictionary: a sorted `word<TAB>count` list, binary-searched.

use std::sync::OnceLock;

static EMBEDDED_TSV: &str = include_str!("../data/si-words.tsv");

/// Upper bound on entries scanned for one prefix, to keep lookups fast.
const SCAN_CAP: usize = 50_000;

pub struct Dictionary<'a> {
    /// Sorted by word (code point order == UTF-8 byte order).
    entries: Vec<(&'a str, u32)>,
}

impl<'a> Dictionary<'a> {
    /// Parses `word<TAB>count` lines; malformed lines are skipped and the
    /// result is sorted if the input was not already.
    pub fn from_tsv(text: &'a str) -> Self {
        let mut entries: Vec<(&str, u32)> = text
            .lines()
            .filter_map(|l| {
                let (w, c) = l.split_once('\t')?;
                Some((w, c.trim().parse().ok()?))
            })
            .collect();
        if !entries.windows(2).all(|w| w[0].0 < w[1].0) {
            entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
            entries.dedup_by(|a, b| a.0 == b.0);
        }
        Dictionary { entries }
    }

    /// The dictionary bundled into the binary (built from Sinhala Wikipedia).
    pub fn embedded() -> &'static Dictionary<'static> {
        static D: OnceLock<Dictionary<'static>> = OnceLock::new();
        D.get_or_init(|| Dictionary::from_tsv(EMBEDDED_TSV))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn freq(&self, word: &str) -> Option<u32> {
        self.entries
            .binary_search_by(|e| e.0.cmp(word))
            .ok()
            .map(|i| self.entries[i].1)
    }

    /// Up to `n` most frequent words starting with `prefix` (including an
    /// exact match), most frequent first.
    pub fn complete(&self, prefix: &str, n: usize) -> Vec<(&'a str, u32)> {
        let start = self.entries.partition_point(|e| e.0 < prefix);
        let mut found: Vec<(&str, u32)> = self.entries[start..]
            .iter()
            .take(SCAN_CAP)
            .take_while(|e| e.0.starts_with(prefix))
            .copied()
            .collect();
        found.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        found.truncate(n);
        found
    }
}
