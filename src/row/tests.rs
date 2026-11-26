//! row.rs: what a tab's row reads at a width and depth, escapes left out.
use super::tab;
use crate::model::{tabs, Tab, FIELDS};

/// A tab named by hand, vim in it, its last command failed with `status`: as model.rs reads it.
fn vim(status: &str) -> Tab {
    let mut f = vec![""; FIELDS.len()];
    for (i, v) in [(0, "@1"), (1, "1"), (2, "abcdefghijklmnop"), (3, "0"), (4, "egov"), (6, "1"), (13, "1"),
                   (14, status), (15, "%1"), (17, "vim"), (18, "/tmp"), (19, "0"), (20, "host")] {
        f[i] = v;
    }
    tabs(&[f.join("\t")]).remove(0)
}

/// The row as it shows: escape sequences dropped.
fn text(tab_: &Tab, depth: usize) -> String {
    let line = tab(tab_, depth, 26, 0);
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
fn the_indent_counts_as_part_of_the_label() {
    let t = vim("1270");
    assert_eq!((t.program.as_str(), t.name.as_str()), ("· vim", "abcdefghijklmnop"));
    // 3 + 8 of the name + " · vim" + " ✗ 1270" = 24: the program stays, the name cut to 10
    assert_eq!(text(&t, 0), "   abcdefghij · vim ✗ 1270");
    // 3 deeper: 27 > 26, as the name would keep only 7: the program goes, the name cut to 13
    assert_eq!(text(&t, 3), "      abcdefghijklm ✗ 1270");
    for depth in [0, 1, 3, 9] {
        assert_eq!(text(&t, depth).chars().count(), 26, "depth {depth}");
    }
}

#[test]
fn a_shorter_status_keeps_the_program_one_level_deeper() {
    // " ✗ 127": 3 + 3 + 8 + 1 + 5 + 6 = 26, not more than the width: exactly 8 of the name show
    assert_eq!(text(&vim("127"), 3), "      abcdefgh · vim ✗ 127");
}
