//! head.rs: what a group's header row reads at a width, escapes left out.
use super::row;

fn group(name: &str) -> (String, String, String) {
    (name.into(), "#283147".into(), "◆".into())
}

/// The row as it shows: escape sequences dropped.
fn text(name: &str, fold: bool, n: usize, time: Option<&str>, width: usize) -> String {
    let line = row(&group(name), fold, n, time.map(|t| (t.to_string(), "busy")), width).0;
    let mut out = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            chars.by_ref().find(|&c| c == 'm');
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn time_after_the_name_then_the_count() {
    assert_eq!(text("Website", false, 3, Some("▶ 12m"), 26), "▾ ◆ Website ▶ 12m         ");
    assert_eq!(text("Ops", true, 1, Some("▶ 5s"), 26), "▸ ◆ Ops ▶ 5s  1 tab       ");
    assert_eq!(text("Infra", true, 3, None, 26), "▸ ◆ Infra  3 tabs        ▷");
}

#[test]
fn the_count_goes_whole_before_the_name_is_cut() {
    // 26 - 13 for "▸ ◆ Abcdefghi" - 7 for " ▶ 2h0m" = 6: "  3 tabs" does not fit, so none
    assert_eq!(text("Abcdefghi", true, 3, Some("▶ 2h0m"), 26), "▸ ◆ Abcdefghi ▶ 2h0m      ");
    assert_eq!(text("Abcdefghijklmnopqrstuvwxyz0123", true, 3, Some("▶ 2h0m"), 26), "▸ ◆ Abcdefghijklmno ▶ 2h0m");
    assert_eq!(text("Abcdefghijklmnopqrstuvwxyz", true, 3, None, 26), "▸ ◆ Abcdefghijklmnopqrst ▷");
}

#[test]
fn a_cut_ending_in_a_space_loses_it() {
    assert_eq!(text("Abcdefghij", false, 1, Some("▶ 1h0m"), 11), "▾ ◆ ▶ 1h0m ");
}

#[test]
fn an_empty_group_has_no_count() {
    assert_eq!(text("Website", true, 0, Some("⏸ 1m"), 20), "▸ ◆ Website ⏸ 1m    ");
}

#[test]
fn tint_and_bold_go_on_past_the_time() {
    let line = row(&group("A"), false, 1, Some(("▶ 5s".into(), "busy")), 12).0;
    let time = line.find("▶").unwrap();
    assert!(line[..time].ends_with("\x1b[38;2;249;226;175m"));
    assert!(line.contains("\x1b[1m▾ ◆ A "));
    let stopped = row(&group("A"), false, 1, Some(("⏸ 5s".into(), "dim")), 12).0;
    assert!(stopped[..stopped.find("⏸").unwrap()].ends_with("\x1b[38;2;108;112;134m"));
    assert!(line.starts_with("\x1b[48;2;40;49;71m") && line.ends_with("\x1b[0m"));
    assert_eq!(line.matches("\x1b[0m").count(), 1);
}

#[test]
fn the_time_columns_follow_the_cut_name() {
    let span = |name: &str, width| row(&group(name), false, 3, Some(("▶ 2h0m".into(), "busy")), width).1;
    assert_eq!(span("Abcdefghijklmnopqrstuvwxyz0123", 26), Some((21, 26))); // ▾ ◆ Abcdefghijklmno ▶ 2h0m
    assert_eq!(span("Abcdefghij", 11), Some((5, 10))); // ▾ ◆ ▶ 2h0m: the cut's space dropped
    assert_eq!(row(&group("A"), false, 1, None, 26).1, Some((26, 26))); // the ▷
}

#[test]
fn without_a_time_a_start_button_in_the_last_column() {
    assert_eq!(text("Infra", false, 3, None, 26), format!("▾ ◆ Infra{}▷", " ".repeat(16)));
    // the count whole with a space before the ▷, else none; then the name is cut, its space dropped
    assert_eq!(text("Abcdefghijkl", true, 3, None, 26), "▸ ◆ Abcdefghijkl  3 tabs ▷");
    assert_eq!(text("Abcdefghijklm", true, 3, None, 26), "▸ ◆ Abcdefghijklm        ▷");
    assert_eq!(text("Abcdefghij", false, 1, None, 11), "▾ ◆ Abcde ▷");
    assert_eq!(text("Abcdefghij", false, 1, None, 6), "▾ ◆  ▷");
    for (name, fold, width) in [("Infra", false, 26), ("Infra", true, 26), ("Abcdefghijklm", true, 26), ("Ab", false, 11)] {
        assert_eq!(row(&group(name), fold, 3, None, width).1, Some((width, width)), "{name} at {width}");
    }
    let line = row(&group("A"), false, 1, None, 12).0; // dim on the tint, bold on
    assert!(line.starts_with("\x1b[48;2;40;49;71m") && line.ends_with("\x1b[38;2;108;112;134m▷\x1b[0m"));
    assert_eq!(line.matches("\x1b[0m").count(), 1);
}

#[test]
fn no_start_button_with_a_time_or_for_a_name_clean_changes() {
    for time in ["▶ 12m", "⏸ 12m"] {
        assert!(!text("Infra", true, 3, Some(time), 26).contains('▷'), "{time}");
    }
    assert_eq!(text("Q'a", false, 1, None, 12), "▾ ◆ Q'a     ");
    assert_eq!(row(&group("Q'a"), false, 1, None, 12).1, None);
}
