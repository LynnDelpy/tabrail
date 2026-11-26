//! The project groups as a tree, from the groups file (groups.conf, or the file in tmux option
//! @tabrail_groups): a group per line that is no comment (its first non-blank character #) and has 3
//! columns or more (name, tint, icon, folders; the icon is not read). A group's parent is the nearest
//! group line above it with less leading whitespace (a space or a tab counts one each), so an indented
//! line with none above is top-level. The one reader of the tree on this side: draw.rs walks it,
//! drag.rs finds by it where a dragged group goes.
use std::ops::Range;

pub struct Group {
    pub name: String,
    pub tint: String,
    pub depth: usize, // 0 top-level, 1 a subgroup of one, …
}

/// The groups in file order, then Default, top-level.
pub fn read(conf: &str) -> Vec<Group> {
    let mut out = Vec::new();
    let mut indents: Vec<usize> = Vec::new(); // of the last group line and its ancestors
    for line in conf.lines().filter(|l| !l.trim_start().starts_with('#')) {
        let c: Vec<&str> = line.split_whitespace().collect();
        if c.len() < 3 {
            continue;
        }
        let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
        while indents.last().is_some_and(|&i| i >= indent) {
            indents.pop();
        }
        out.push(Group { name: c[0].into(), tint: c[1].into(), depth: indents.len() });
        indents.push(indent);
    }
    out.push(Group { name: "Default".into(), tint: String::new(), depth: 0 });
    out
}

/// The indexes of group `i`'s descendants: the groups after it, up to the next one as shallow as it.
pub fn subtree(groups: &[Group], i: usize) -> Range<usize> {
    let depth = groups[i].depth;
    let end = groups[i + 1..].iter().position(|g| g.depth <= depth).map_or(groups.len(), |n| i + 1 + n);
    i + 1..end
}

/// The index of the group named `name`: its first line.
pub fn find(groups: &[Group], name: &str) -> Option<usize> {
    groups.iter().position(|g| g.name == name)
}

/// Group `i`'s parent: the nearest group above it that is shallower; none for a top-level one or Default.
pub fn parent(groups: &[Group], i: usize) -> Option<usize> {
    (0..i).rev().find(|&j| groups[j].depth < groups[i].depth)
}

/// `of` or its ancestor that is a sibling of `src` (has its parent: top-level groups and Default are
/// siblings); none if there is none or that is `src` itself, `of` being in `src`'s tree.
pub fn sibling<'a>(groups: &'a [Group], src: &str, of: &str) -> Option<&'a str> {
    let (s, mut i) = (find(groups, src)?, find(groups, of)?);
    while parent(groups, i) != parent(groups, s) {
        i = parent(groups, i)?;
    }
    (i != s).then(|| groups[i].name.as_str())
}

/// Whether `src` may become a subgroup of `into`: neither is Default, `into` is not in `src`'s tree.
pub fn nests(groups: &[Group], src: &str, into: &str) -> bool {
    let (Some(s), Some(i)) = (find(groups, src), find(groups, into)) else { return false };
    src != "Default" && into != "Default" && !(s..subtree(groups, s).end).contains(&i)
}

#[cfg(test)]
mod tests;
