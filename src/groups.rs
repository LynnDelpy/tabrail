//! The project groups as a tree, from the groups file (groups.conf, or the file in tmux option
//! @tabrail_groups): a group per line that is no comment (its first non-blank character #) and has 3
//! columns or more (name, tint, icon, folders). A group's parent is the nearest group line above it
//! with less leading whitespace (a space or a tab counts one each), so an indented line with none above
//! is top-level. The one reader of the tree on this side: draw.rs walks it, drag.rs will move whole
//! subtrees by it.
use std::ops::Range;

pub struct Group {
    pub name: String,
    pub tint: String,
    pub icon: String,
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
        out.push(Group { name: c[0].into(), tint: c[1].into(), icon: c[2].into(), depth: indents.len() });
        indents.push(indent);
    }
    out.push(Group { name: "Default".into(), tint: String::new(), icon: "•".into(), depth: 0 });
    out
}

/// The indexes of group `i`'s descendants: the groups after it, up to the next one as shallow as it.
pub fn subtree(groups: &[Group], i: usize) -> Range<usize> {
    let depth = groups[i].depth;
    let end = groups[i + 1..].iter().position(|g| g.depth <= depth).map_or(groups.len(), |n| i + 1 + n);
    i + 1..end
}

#[cfg(test)]
mod tests;
