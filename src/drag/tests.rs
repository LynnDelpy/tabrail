//! drag.rs: a left press and release on a group's row, on its time or ▷ (head.rs's columns) or beside it.
use super::left;
use crate::draw::Row;
use crate::head;
use crate::input::Click;
use crate::menu::Place;
use std::collections::HashSet;

const P: Place<'static> = Place { pane: "%1", helper: "/t/tabrail-act", mover: "", namer: "", timer: "/t/tabrail-timer",
                                  colourer: "" };
const TOGGLE: &str = "run-shell -b \"/t/tabrail-timer toggle 'Website'\"";

/// `name`'s header row at width 26 with three tabs, `fold`ed or not, as draw makes it.
fn group(name: &str, fold: bool, time: Option<&str>) -> Row {
    let g = (name.to_string(), "#283147".to_string(), "◆".to_string());
    Row::Group(name.into(), head::row(&g, fold, 3, time.map(|t| (t.to_string(), "busy")), 26).1)
}

/// What a left press or release at column `col` of row `y` (holding `row`) sends to tmux.
fn send(drag: &mut Option<(usize, usize, Row)>, (col, y): (usize, usize), row: &Row, release: bool,
        folded: &HashSet<String>) -> Vec<String> {
    left(&Click { right: false, release, col, row: y }, Some(row), &[], folded, &P, drag)
}

/// A press at column `down` and a release at `up` on row 4: the press sends nothing.
fn click(row: &Row, down: usize, up: usize, fold: bool) -> Vec<String> {
    let folded: HashSet<String> = if fold { ["Website".to_string()].into() } else { HashSet::new() };
    let mut drag = None;
    assert!(send(&mut drag, (down, 4), row, false, &folded).is_empty(), "the press sends nothing");
    send(&mut drag, (up, 4), row, true, &folded)
}

/// Website's fold: added to @sidebar_fold while unfolded, removed while `fold`ed.
fn folds(fold: bool) -> [String; 2] {
    [format!("set -g @sidebar_fold '{}'", if fold { "" } else { "Website" }), "set-buffer -b sidebar-refresh x".into()]
}

#[test]
fn the_time_is_columns_13_to_17() {
    // ▾ ◆ Website ▶ 12m: ▾=1, ◆=3, Website=5-11, space 12, ▶ 12m=13-17; folded the count comes after
    assert!(matches!(group("Website", false, Some("▶ 12m")), Row::Group(_, Some((13, 17)))));
    assert!(matches!(group("Website", true, Some("▶ 12m")), Row::Group(_, Some((13, 17)))));
    assert!(matches!(group("Website", false, None), Row::Group(_, Some((25, 25))))); // ▷ one in from the edge
}

#[test]
fn press_and_release_on_the_time_toggle_it_folded_or_not() {
    for fold in [false, true] {
        let row = group("Website", fold, Some("▶ 12m"));
        for (down, up) in [(13, 13), (17, 17), (13, 17), (17, 13), (15, 15)] {
            assert_eq!(click(&row, down, up, fold), [TOGGLE], "{down}-{up}, folded {fold}");
        }
        assert_eq!(click(&group("Website", fold, Some("⏸ 12m")), 14, 14, fold), [TOGGLE]); // resume
        assert_eq!(click(&group("Website", fold, None), 25, 25, fold), [TOGGLE]); // never timed: the ▷ starts
    }
}

#[test]
fn beside_the_time_folds_as_before() {
    for fold in [false, true] {
        let row = group("Website", fold, Some("▶ 12m"));
        // one beside each end, the name, the space after, off the time on either end
        for (down, up) in [(12, 12), (18, 18), (5, 5), (22, 22), (13, 5), (5, 13), (17, 18), (12, 13)] {
            assert_eq!(click(&row, down, up, fold), folds(fold), "{down}-{up}, folded {fold}");
        }
        for (down, up) in [(13, 13), (24, 24), (26, 26), (25, 5), (5, 25)] { // no time: where one goes, beside and off the ▷
            assert_eq!(click(&group("Website", fold, None), down, up, fold), folds(fold), "{down}-{up}, no time");
        }
    }
}

#[test]
fn the_columns_count_as_at_the_press() {
    let none = HashSet::new();
    // ▶ 59s (13-17) at the press, ▶ 1m (13-16) by the release: column 17 still toggles
    let mut drag = None;
    assert!(send(&mut drag, (17, 4), &group("Website", false, Some("▶ 59s")), false, &none).is_empty());
    assert_eq!(send(&mut drag, (17, 4), &group("Website", false, Some("▶ 1m")), true, &none), [TOGGLE]);
    // no time at the press, another group's time on the row by the release: folds, starts nothing
    let mut drag = None;
    assert!(send(&mut drag, (13, 4), &group("Website", false, None), false, &none).is_empty());
    assert_eq!(send(&mut drag, (13, 4), &group("Infra", false, Some("▶ 12m")), true, &none), folds(false));
}

#[test]
fn a_drag_from_the_time_moves_the_group() {
    let (mut drag, none) = (None, HashSet::new());
    assert!(send(&mut drag, (14, 4), &group("Website", false, Some("▶ 12m")), false, &none).is_empty());
    let cmds = send(&mut drag, (14, 9), &group("Infra", false, Some("▶ 3m")), true, &none);
    assert_eq!(cmds, ["run-shell -b '/t/tabrail-act move-group \"Website\" \"Infra\"'"]);
}

#[test]
fn a_tab_row_and_a_dirty_name() {
    assert_eq!(click(&Row::Tab("@3".into()), 24, 24, false), ["select-window -t @3"]);
    // no toggle, as no timer items in its menu: folds as a click on its name does; without a time no ▷ at 25
    let (dirty, bare) = (group("Q'a", false, Some("▶ 12m")), group("Q'a", false, None));
    for (row, col) in [(&dirty, 9), (&bare, 25)] { assert_eq!(click(row, col, col, false), click(row, 5, 5, false)); }
    assert!(click(&dirty, 9, 9, false)[0].starts_with("set -g @sidebar_fold "));
}
