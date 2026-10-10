//! Sinhala transliteration core: Singlish -> Sinhala candidates.

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
