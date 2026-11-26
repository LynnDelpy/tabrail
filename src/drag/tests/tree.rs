//! drag.rs, group.rs: a group's row let go over another row. On a header at or right of its name it
//! nests into that group, left of the name or on a tab's row it moves next to the sibling there;
//! no sibling there, Default and the group's own tree send nothing.
use super::P;
use crate::draw::Row;
use crate::groups::read;
use crate::input::Click;
use crate::model::{tabs, Tab, FIELDS};
use crate::{drag::left, head};
use std::collections::HashSet;

const TREE: &str = "egov #1 ★\n  St.Gallen #2 ✚\n    Archiv #3 ◆\n  Köniz #4 ✚\n4tw #5 ■\nQ'a #6 ●\n";

/// Tab `id` (@N) in `group`, as model.rs reads it.
fn tab(id: &str, group: &str) -> Tab {
    let mut f = vec![""; FIELDS.len()];
    for (i, v) in [(0, id), (1, &id[1..]), (2, "x"), (4, group)] {
        f[i] = v;
    }
    tabs(&[f.join("\t")]).remove(0)
}

fn header(name: &str) -> Row {
    Row::Group(name.into(), None, None)
}

fn on_tab(id: &str) -> Row {
    Row::Tab(id.into())
}

/// What group `src`'s header, pressed on row 2, sends let go over `row` at column `col` on row 9; tabs
/// @1 in egov, @2 in 4tw, @3 in Default; egov folded.
fn drop(src: &str, row: Row, col: usize) -> Vec<String> {
    let (groups, folded) = (read(TREE), HashSet::from(["egov".to_string()]));
    let tabs = [tab("@1", "egov"), tab("@2", "4tw"), tab("@3", "Default")];
    let (mut drag, click) = (None, |release, col, row| Click { right: false, release, col, row });
    assert!(left(&click(false, 5, 2), Some(&header(src)), &tabs, &groups, &folded, &P, &mut drag).is_empty());
    left(&click(true, col, 9), Some(&row), &tabs, &groups, &folded, &P, &mut drag)
}

fn cmd(verb: &str, src: &str, onto: &str) -> Vec<String> {
    vec![format!("run-shell -b '/t/tabrail-groups {verb} \"{src}\" \"{onto}\"'")]
}

#[test]
fn a_header_s_name_starts_at_column_depth_plus_3() {
    for g in read(TREE) { // head.rs's layout, as group.rs reads it
        let line = head::row(&g, false, 1, None, 40).0;
        let shown: String = line.split("\x1b[1m").nth(1).unwrap_or("").chars().skip(g.depth + 2).collect();
        assert!(shown.starts_with(&g.name), "{} at depth {}: {shown}", g.name, g.depth);
    }
}

#[test]
fn left_of_a_name_or_on_a_tab_row_it_moves_next_to_the_sibling_there() {
    assert_eq!(drop("4tw", on_tab("@1"), 20), cmd("move-group", "4tw", "egov")); // up: on egov's tab
    assert_eq!(drop("egov", header("4tw"), 2), cmd("move-group", "egov", "4tw")); // down: left of 4tw's name
    assert_eq!(drop("4tw", header("St.Gallen"), 3), cmd("move-group", "4tw", "egov")); // its name from 4
    assert_eq!(drop("4tw", header("Archiv"), 1), cmd("move-group", "4tw", "egov"));
    assert_eq!(drop("Köniz", header("Archiv"), 4), cmd("move-group", "Köniz", "St.Gallen")); // name from 5
    assert_eq!(drop("4tw", on_tab("@3"), 20), cmd("move-group", "4tw", "Default"));
    assert_eq!(drop("4tw", header("Default"), 2), cmd("move-group", "4tw", "Default"));
}

#[test]
fn at_or_right_of_a_header_s_name_it_nests() {
    for col in [4, 5, 25] {
        assert_eq!(drop("4tw", header("St.Gallen"), col), cmd("nest", "4tw", "St.Gallen"), "{col}");
    }
    assert_eq!(drop("Köniz", header("egov"), 3), cmd("nest", "Köniz", "egov")); // its own parent's
    assert_eq!(drop("4tw", header("egov"), 3), cmd("nest", "4tw", "egov"));
    assert_eq!(drop("Köniz", header("4tw"), 3), cmd("nest", "Köniz", "4tw"));
    assert_eq!(drop("Archiv", header("egov"), 20), cmd("nest", "Archiv", "egov"));
    assert_eq!(drop("4tw", header("Archiv"), 5), cmd("nest", "4tw", "Archiv")); // depth 2: name from 5
}

#[test]
fn no_sibling_default_its_own_tree_or_a_dirty_name_send_nothing() {
    for (src, row, col) in [("Köniz", on_tab("@2"), 20), ("Köniz", header("4tw"), 2), ("Köniz", on_tab("@3"), 20),
                            ("Köniz", on_tab("@1"), 20), ("egov", header("St.Gallen"), 4), ("egov", header("Archiv"), 2),
                            ("egov", on_tab("@1"), 20), ("St.Gallen", header("Archiv"), 9), ("4tw", header("Default"), 3),
                            ("Default", header("egov"), 3), ("Default", header("egov"), 2), ("Default", on_tab("@2"), 3),
                            ("Q'a", on_tab("@1"), 20), ("4tw", header("Q'a"), 2), ("4tw", header("Q'a"), 9),
                            ("4tw", Row::Nothing, 5), ("4tw", on_tab("@9"), 5)] {
        assert!(drop(src, row.clone(), col).is_empty(), "{src} at {col}");
    }
}
