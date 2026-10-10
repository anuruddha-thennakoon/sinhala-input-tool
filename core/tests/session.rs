use sinhala_core::{Dictionary, Session};

const SMALL: &str = "නිලධාරි\t10\nනිලධාරී\t50\nනිළධාරි\t1\nමම\t100\n";

fn typed(word: &str) -> Session<'static> {
    let d: &'static Dictionary<'static> = Box::leak(Box::new(Dictionary::from_tsv(SMALL)));
    let mut s = Session::new(d);
    for c in word.chars() {
        s.push_char(c);
    }
    s
}

fn embedded(word: &str) -> Session<'static> {
    let mut s = Session::new(Dictionary::embedded());
    for c in word.chars() {
        s.push_char(c);
    }
    s
}

#[test]
fn typing_builds_buffer_and_popup_rows() {
    let s = typed("niladhaari");
    assert!(s.is_composing());
    assert_eq!(s.buffer(), "niladhaari");
    let v = s.view();
    assert_eq!(v.rows.len(), 6); // 5 Sinhala + raw Latin, as in the screenshot
    assert_eq!(v.rows[0], "නිලධාරී");
    assert_eq!(v.rows[5], "niladhaari");
    assert_eq!(v.selected, 0);
    assert_eq!((v.page, v.page_count > 1), (0, true));
}

#[test]
fn arrows_move_and_clamp() {
    let mut s = typed("niladhaari");
    s.move_selection(1);
    assert_eq!(s.view().selected, 1);
    s.move_selection(-1);
    s.move_selection(-1); // clamp at top
    assert_eq!(s.view().selected, 0);
    s.move_selection(-100); // still clamped
    assert_eq!(s.view().selected, 0);
}

#[test]
fn down_arrow_flows_into_next_page() {
    let mut s = embedded("ka");
    assert!(s.view().page_count > 1);
    for _ in 0..5 {
        s.move_selection(1);
    }
    let v = s.view();
    assert_eq!((v.page, v.selected), (1, 0));
    s.move_selection(-1);
    let v = s.view();
    assert_eq!((v.page, v.selected), (0, 4));
}

#[test]
fn page_turning() {
    let mut s = embedded("ka");
    let first_page = s.view().rows;
    s.turn_page(1);
    let v = s.view();
    assert_eq!((v.page, v.selected), (1, 0));
    assert_ne!(v.rows, first_page);
    s.turn_page(-1);
    assert_eq!(s.view().rows, first_page);
    s.turn_page(-1); // clamp
    assert_eq!(s.view().page, 0);
    s.turn_page(1000); // clamp to last page
    let last = s.view();
    assert_eq!(last.page, last.page_count - 1);
}

#[test]
fn commit_returns_selected_and_ends_composition() {
    let mut s = typed("niladhaari");
    s.move_selection(1);
    assert_eq!(s.commit().as_deref(), Some("නිලධාරි"));
    assert!(!s.is_composing());
    assert_eq!(s.buffer(), "");
    assert_eq!(s.commit(), None);
}

#[test]
fn number_keys_pick_rows_including_raw() {
    let mut s = typed("niladhaari");
    assert_eq!(s.select_number(6).as_deref(), Some("niladhaari"));
    assert!(!s.is_composing());

    let mut s = typed("niladhaari");
    assert_eq!(s.select_number(2).as_deref(), Some("නිලධාරි"));

    let mut s = typed("niladhaari");
    assert_eq!(s.select_number(0), None);
    assert_eq!(s.select_number(7), None);
    assert!(s.is_composing(), "bad number must not end composition");
}

#[test]
fn number_keys_apply_to_current_page() {
    let mut s = embedded("ka");
    s.turn_page(1);
    let want = s.view().rows[0].clone();
    assert_eq!(s.select_number(1), Some(want));
}

#[test]
fn backspace_edits_then_reports_empty() {
    let mut s = typed("mamaa");
    assert!(s.backspace());
    assert_eq!(s.buffer(), "mama");
    assert_eq!(s.view().rows[0], "මම");
    for _ in 0..4 {
        assert!(s.backspace());
    }
    assert!(!s.is_composing());
    assert!(!s.backspace(), "empty buffer: key should pass through");
}

#[test]
fn any_key_resets_selection() {
    let mut s = typed("niladhaar");
    s.move_selection(2);
    s.push_char('i');
    assert_eq!(s.view().selected, 0);
}

#[test]
fn cancel_clears() {
    let mut s = typed("mama");
    s.cancel();
    assert!(!s.is_composing());
    assert!(s.view().rows.is_empty());
}

#[test]
fn unknown_word_shows_rules_and_raw() {
    let s = typed("kopi");
    let v = s.view();
    assert_eq!(v.rows[0], "කොපි");
    assert_eq!(v.rows.last().unwrap(), "kopi");
}
