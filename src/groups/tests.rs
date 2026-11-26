//! groups.rs: depths by the parent rule, and subtrees.
use super::{read, subtree};

/// (name, depth) per group read from `conf`, Default included.
fn tree(conf: &str) -> Vec<(String, usize)> {
    read(conf).into_iter().map(|g| (g.name, g.depth)).collect()
}

fn names(list: &[(&str, usize)]) -> Vec<(String, usize)> {
    list.iter().map(|&(n, d)| (n.to_string(), d)).collect()
}

#[test]
fn a_flat_file_is_all_top_level_then_default() {
    let conf = "# a comment  with  words\nWebsite #283147 ◆ ~/src/web\n\nInfra #3b2f2a ▲\nNotes #2a3b2f\n";
    assert_eq!(tree(conf), names(&[("Website", 0), ("Infra", 0), ("Default", 0)]));
    let g = read(conf);
    assert_eq!((g[0].tint.as_str(), g[0].icon.as_str()), ("#283147", "◆"));
    assert_eq!((g[2].tint.as_str(), g[2].icon.as_str()), ("", "•"));
}

#[test]
fn the_parent_is_the_nearest_line_above_with_less_whitespace() {
    let conf = "egov #1 ★\n  St.Gallen #2 ✚\n    Archiv #3 ◆\n\n  Köniz #4 ✚\n4tw #5 ■\n";
    assert_eq!(tree(conf), names(&[("egov", 0), ("St.Gallen", 1), ("Archiv", 2), ("Köniz", 1), ("4tw", 0),
                                    ("Default", 0)]));
    // an indented comment is no group: Archiv stays St.Gallen's
    let conf = "egov #1 ★\n  # customers of egov\n  St.Gallen #2 ✚\n\t# x y z\n    Archiv #3 ◆\n";
    assert_eq!(tree(conf), names(&[("egov", 0), ("St.Gallen", 1), ("Archiv", 2), ("Default", 0)]));
    // shallower than its sibling but deeper than the parent: still the parent's
    let conf = "a #1 x\n    b #2 x\n  c #3 x\n   d #4 x\n";
    assert_eq!(tree(conf), names(&[("a", 0), ("b", 1), ("c", 1), ("d", 2), ("Default", 0)]));
}

#[test]
fn a_space_or_a_tab_counts_one() {
    let conf = "a #1 x\n\tb #2 x\n \tc #3 x\n  d #4 x\n";
    assert_eq!(tree(conf), names(&[("a", 0), ("b", 1), ("c", 2), ("d", 2), ("Default", 0)])); // " \t" as "  "
}

#[test]
fn an_indented_line_with_no_shallower_one_above_is_top_level() {
    let conf = "   egov #1 ★\n  St.Gallen #2 ✚\n    Archiv #3 ◆\n";
    assert_eq!(tree(conf), names(&[("egov", 0), ("St.Gallen", 0), ("Archiv", 1), ("Default", 0)]));
}

#[test]
fn a_subtree_runs_to_the_next_group_as_shallow() {
    let g = read("egov #1 ★\n  St.Gallen #2 ✚\n    Archiv #3 ◆\n  Köniz #4 ✚\n4tw #5 ■\n");
    assert_eq!(subtree(&g, 0), 1..4);
    assert_eq!(subtree(&g, 1), 2..3);
    assert_eq!(subtree(&g, 2), 3..3);
    assert_eq!(subtree(&g, 4), 5..5);
    assert_eq!(subtree(&g, 5), 6..6); // Default, last
    let g = read("a #1 x\n  b #2 x\n");
    assert_eq!(subtree(&g, 0), 1..2); // up to Default
}
