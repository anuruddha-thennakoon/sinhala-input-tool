use sinhala_core::{about_text, Settings};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("sinhala-core-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn default_is_enabled() {
    assert!(Settings::default().enabled);
}

#[test]
fn parse_variants() {
    assert!(Settings::parse("enabled=true\n").enabled);
    assert!(!Settings::parse("enabled=false\n").enabled);
    assert!(!Settings::parse("# comment\nenabled = false\n").enabled);
    assert!(!Settings::parse("other=1\r\nenabled=false\r\n").enabled); // CRLF + unknown key
    assert!(Settings::parse("").enabled); // empty file: defaults
    assert!(Settings::parse("enabled=maybe").enabled); // garbage value: defaults
    assert!(Settings::parse("\u{0}\u{1}garbage").enabled);
}

#[test]
fn serialize_round_trips() {
    for enabled in [true, false] {
        let s = Settings { enabled };
        assert_eq!(Settings::parse(&s.to_ini()), s);
    }
}

#[test]
fn save_then_load_persists_across_instances() {
    let dir = temp_dir("persist");
    let path = dir.join("nested").join("settings.ini"); // parent dirs do not exist yet
    assert!(Settings::load(&path).enabled, "missing file means defaults");

    Settings { enabled: false }.save(&path).unwrap();
    assert!(!Settings::load(&path).enabled);

    Settings { enabled: true }.save(&path).unwrap(); // overwrite
    assert!(Settings::load(&path).enabled);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn corrupt_file_falls_back_to_defaults() {
    let dir = temp_dir("corrupt");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("settings.ini");
    std::fs::write(&path, [0xff, 0xfe, 0x00, 0x80]).unwrap(); // not UTF-8
    assert!(Settings::load(&path).enabled);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn about_text_has_version_and_wikipedia_attribution() {
    let t = about_text();
    assert!(t.contains(env!("CARGO_PKG_VERSION")));
    assert!(t.contains("Wikipedia"));
    assert!(t.contains("CC BY-SA 4.0"));
}
