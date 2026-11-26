//! head.rs: what a group's header row reads at a width, escapes left out.
use super::{row, START};
use crate::groups::Group;

fn group(name: &str) -> Group {
    Group { name: name.into(), tint: "#283147".into(), depth: 0 }
}

/// The row as it shows: escape sequences dropped, the start button as ▷.
fn text(name: &str, fold: bool, n: usize, time: Option<&str>, width: usize) -> String {
    let line = row(&group(name), fold, n, time.map(|t| (t.to_string(), "busy")), width).0;
    let mut esc = false; // from \x1b to its m
    let out: String = line.chars().filter(|&c| (!esc && c != '\x1b', esc = (esc || c == '\x1b') && c != 'm').0).collect();
    out.replace(START, "▷")
}

#[test]
fn time_after_the_name_then_the_count() {
    assert_eq!(text("Website", false, 3, Some("▶ 12m"), 26), "▾ Website ▶ 12m         + ");
    assert_eq!(text("Ops", true, 1, Some("▶ 5s"), 26), "▸ Ops ▶ 5s  1 tab       + ");
    assert_eq!(text("Infra", true, 3, None, 26), "▸ Infra  3 tabs       ▷ + ");
    assert_eq!(text("Website", true, 0, Some("⏸ 1m"), 20), "▸ Website ⏸ 1m    + "); // empty: no count
}

#[test]
fn the_count_goes_whole_before_the_name_is_cut() {
    // 26 - 13 for "▸ Abcdefghijk" - 7 for " ▶ 2h0m" - 3 for " + " = 3: "  3 tabs" does not fit, so none
    assert_eq!(text("Abcdefghijk", true, 3, Some("▶ 2h0m"), 26), "▸ Abcdefghijk ▶ 2h0m    + ");
    assert_eq!(text("Abcdefghijklmnopqrstuvwxyz0123", true, 3, Some("▶ 2h0m"), 26), "▸ Abcdefghijklmn ▶ 2h0m + ");
    assert_eq!(text("Abcdefghijklmnopqrstuvwxyz", true, 3, None, 26), "▸ Abcdefghijklmnopqrs ▷ + ");
}

#[test]
fn a_cut_ending_in_a_space_loses_it() {
    assert_eq!(text("Abcdefghij", false, 1, Some("▶ 1h0m"), 12), "▾ ▶ 1h0m  + ");
}

#[test]
fn tint_and_bold_go_on_past_the_time() {
    let line = row(&group("A"), false, 1, Some(("▶ 5s".into(), "busy")), 14).0;
    let time = line.find("▶").unwrap();
    assert!(line[..time].ends_with("\x1b[38;2;249;226;175m"));
    assert!(line.contains("\x1b[1m▾ A "));
    let stopped = row(&group("A"), false, 1, Some(("⏸ 5s".into(), "dim")), 14).0;
    assert!(stopped[..stopped.find("⏸").unwrap()].ends_with("\x1b[38;2;108;112;134m"));
    assert!(line.starts_with("\x1b[48;2;40;49;71m") && line.ends_with("\x1b[38;2;108;112;134m + \x1b[0m"));
    assert_eq!(line.matches("\x1b[0m").count(), 1);
}

#[test]
fn the_time_columns_follow_the_cut_name() {
    let span = |name: &str, width| row(&group(name), false, 3, Some(("▶ 2h0m".into(), "busy")), width).1;
    assert_eq!(span("Abcdefghijklmnopqrstuvwxyz0123", 26), Some((18, 23))); // ▾ Abcdefghijklmn ▶ 2h0m +
    assert_eq!(span("Abcdefghij", 12), Some((3, 8))); // ▾ ▶ 2h0m: the cut's space dropped
    assert_eq!(row(&group("A"), false, 1, None, 26).1, Some((23, 23))); // the ▷, two left of the +
}

#[test]
fn a_plus_one_in_from_the_edge_and_a_start_button_two_left_of_it() {
    assert_eq!(text("Infra", false, 3, None, 26), format!("▾ Infra{}▷ + ", " ".repeat(15)));
    // the count whole with a space before the ▷, else none; then the name is cut, its space dropped
    assert_eq!(text("Abcdefghijk", true, 3, None, 26), "▸ Abcdefghijk  3 tabs ▷ + ");
    assert_eq!(text("Abcdefghijkl", true, 3, None, 26), "▸ Abcdefghijkl        ▷ + ");
    assert_eq!(text("Abcdefghij", false, 1, None, 11), "▾ Abcd ▷ + ");
    assert_eq!(text("Abcdefghij", false, 1, None, 8), "▾ A ▷ + ");
    for (name, fold, width) in [("Infra", false, 26), ("Infra", true, 26), ("Abcdefghijklm", true, 26), ("Ab", false, 11)] {
        let (_, start, plus) = row(&group(name), fold, 3, None, width);
        assert_eq!((start, plus), (Some((width - 3, width - 3)), Some(width - 1)), "{name} at {width}");
    }
    let line = row(&group("A"), false, 1, None, 12).0; // dim on the tint, bold on
    assert!(line.starts_with("\x1b[48;2;40;49;71m") && line.ends_with(&format!("\x1b[38;2;108;112;134m {START} + \x1b[0m")));
    assert_eq!(line.matches("\x1b[0m").count(), 1);
}

#[test]
fn with_a_time_the_plus_stays_default_has_none() {
    let plus = |time: Option<&str>| row(&group("Infra"), true, 3, time.map(|t| (t.into(), "busy")), 26).2;
    assert!([Some("▶ 12m"), Some("⏸ 12m"), None].into_iter().all(|t| plus(t) == Some(25)));
    assert_eq!(text("Default", false, 1, None, 26), format!("▾ Default{}▷   ", " ".repeat(13)));
    assert_eq!(text("Default", false, 1, Some("▶ 5s"), 26), format!("▾ Default ▶ 5s{}", " ".repeat(12)));
    assert_eq!(row(&group("Default"), false, 1, None, 26).1, Some((23, 23)));
    assert!([None, Some(("▶ 5s".into(), "busy"))].into_iter().all(|t| row(&group("Default"), false, 1, t, 26).2.is_none()));
}

#[test]
fn neither_start_button_nor_plus_for_a_name_clean_changes() {
    assert!(!text("Infra", true, 3, Some("▶ 12m"), 26).contains('▷'));
    assert_eq!(text("Q'a", false, 1, None, 12), "▾ Q'a       ");
    assert_eq!(row(&group("Q'a"), false, 1, None, 12).1, None);
    assert_eq!(row(&group("Q'a"), false, 1, Some(("▶ 5s".into(), "busy")), 26).2, None);
}

#[test]
fn a_subgroup_one_column_in_per_level() { // "  ▸ Archiv " is 11 columns: the time at 12-15
    let (line, span, plus) = row(&Group { depth: 2, ..group("Archiv") }, true, 3, Some(("▶ 5s".into(), "busy")), 26);
    assert!(line.contains("\x1b[1m  ▸ Archiv \x1b[") && span == Some((12, 15)) && plus == Some(25), "{line:?} {span:?}");
}
