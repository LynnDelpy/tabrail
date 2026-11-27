//! drag.rs: a left press and release on a group's + (head.rs's column, 25 at width 26) opens the name
//! field for a subgroup of it; beside it, off it or with another group's row by the release, as before;
//! let go on another group's row, a group drag.
use super::{click, folds, group, send};
use crate::draw::Row;
use std::collections::HashSet;

const SUB: &str = "run-shell -b \"/t/tabrail-name sub 'Website' '' '' p1\"";

#[test]
fn press_and_release_on_the_plus_open_the_name_field() {
    for fold in [false, true] {
        for time in [None, Some("▶ 12m"), Some("⏸ 12m")] {
            let row = group("Website", fold, time);
            assert!(matches!(row, Row::Group(_, _, Some(25))), "folded {fold}, {time:?}");
            assert_eq!(click(&row, 25, 25, fold), [SUB], "folded {fold}, {time:?}");
        }
    }
}

#[test]
fn beside_or_off_the_plus_folds() {
    for fold in [false, true] {
        let row = group("Website", fold, None);
        for (down, up) in [(24, 24), (26, 26), (25, 5), (5, 25), (25, 26), (24, 25), (25, 23)] {
            assert_eq!(click(&row, down, up, fold), folds(fold), "{down}-{up}, folded {fold}");
        }
    }
}

#[test]
fn a_drag_from_the_plus_nests_or_moves_the_group() {
    let none = HashSet::new();
    for (up, verb) in [(25, "nest"), (5, "nest"), (4, "move-group")] { // on Infra's + or name, left of the name
        let mut drag = None;
        assert!(send(&mut drag, (25, 4), &group("Website", false, None), false, &none).is_empty());
        let cmds = send(&mut drag, (up, 9), &group("Infra", false, None), true, &none);
        assert_eq!(cmds, [format!("run-shell -b '/t/tabrail-groups {verb} \"Website\" \"Infra\"'")], "{up}");
    }
}

#[test]
fn another_group_on_the_row_by_the_release_folds_the_pressed_one() {
    let (mut drag, none) = (None, HashSet::new());
    assert!(send(&mut drag, (25, 4), &group("Website", false, None), false, &none).is_empty());
    assert_eq!(send(&mut drag, (25, 4), &group("Infra", false, None), true, &none), folds(false));
}

#[test]
fn a_tab_row_default_and_a_dirty_name() {
    assert_eq!(click(&Row::Tab("@3".into()), 25, 25, false), ["select-window -t @3"]);
    // a name clean() changes: no toggle (no timer items in its menu) and no + (it cannot pass through
    // tmux's command strings), so it folds as a click on its name does; Default has no + either
    let (dirty, bare, default) = (group("Q'a", false, Some("▶ 12m")), group("Q'a", false, None), group("Default", false, None));
    for (row, col) in [(&dirty, 9), (&dirty, 25), (&bare, 23), (&bare, 25), (&default, 25)] {
        assert!(matches!(row, Row::Group(_, _, None)), "{col}");
        assert_eq!(click(row, col, col, false), click(row, 5, 5, false), "{col}");
    }
    assert!(click(&dirty, 9, 9, false)[0].starts_with("set -g @sidebar_fold "));
}
