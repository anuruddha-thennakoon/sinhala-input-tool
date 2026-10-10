//! Try the ranker: `cargo run --release -p sinhala-core --example suggest -- niladhaari saha`
use sinhala_core::{suggest, Dictionary};
use std::time::Instant;

fn main() {
    let t = Instant::now();
    let dict = Dictionary::embedded();
    println!("dictionary: {} words, loaded in {:?}", dict.len(), t.elapsed());
    for word in std::env::args().skip(1) {
        let t = Instant::now();
        let got = suggest(&word, dict, 6);
        println!("{word:<14} {:?}  [{:?}]", got, t.elapsed());
    }
}
