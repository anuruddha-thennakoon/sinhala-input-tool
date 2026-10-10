//! Drive a typing session from the terminal, one command per line.
//!   cargo run --release -p sinhala-core --example harness
//!   printf 'type niladhaari\ndown\nspace\n' | cargo run -q --release -p sinhala-core --example harness
//!
//! Commands: type <latin> | down | up | pgdn | pgup | bs | space | enter |
//!           1..6 (pick row) | esc | quit
use sinhala_core::{Dictionary, Session};
use std::io::{self, BufRead};

fn show(doc: &str, s: &Session) {
    println!("  document: {doc}{}", if s.is_composing() { format!("[{}]", s.buffer()) } else { String::new() });
    let v = s.view();
    if v.rows.is_empty() {
        return;
    }
    println!("  ┌ {}", s.buffer());
    for (i, row) in v.rows.iter().enumerate() {
        println!("  │{}{}. {row}", if i == v.selected { '▶' } else { ' ' }, i + 1);
    }
    println!("  └ page {}/{}", v.page + 1, v.page_count);
}

fn main() {
    let mut s = Session::new(Dictionary::embedded());
    let mut doc = String::new();
    for line in io::stdin().lock().lines().map_while(Result::ok) {
        let line = line.trim();
        println!("> {line}");
        let (cmd, arg) = line.split_once(' ').unwrap_or((line, ""));
        match cmd {
            "type" => arg.chars().for_each(|c| s.push_char(c)),
            "down" => s.move_selection(1),
            "up" => s.move_selection(-1),
            "pgdn" => s.turn_page(1),
            "pgup" => s.turn_page(-1),
            "bs" => {
                if !s.backspace() {
                    doc.pop(); // passes through to the document
                }
            }
            "space" | "enter" => {
                if let Some(t) = s.commit() {
                    doc.push_str(&t);
                }
                doc.push(if cmd == "space" { ' ' } else { '\n' });
            }
            "esc" => s.cancel(),
            "quit" => break,
            n if n.len() == 1 && n.as_bytes()[0].is_ascii_digit() => {
                if let Some(t) = s.select_number(n.parse().unwrap()) {
                    doc.push_str(&t);
                }
            }
            _ => println!("  ?"),
        }
        show(&doc, &s);
    }
}
