//! Sinhala transliteration core: Singlish -> Sinhala candidates.

pub mod tables;
mod dictionary;
mod suggest;
mod translit;

pub use dictionary::Dictionary;
pub use suggest::suggest;
pub use translit::transliterate;

/// Placeholder until Phase 1; proves the crate builds on every OS.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn builds() {
        assert_eq!(super::version(), "0.1.0");
    }
}
