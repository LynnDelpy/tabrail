//! drag.rs: a group's row let go over another row. On a group's header at or right of its name (column
//! depth + 5, head.rs: the indent, the arrow and the icon with a space after each) the dragged group
//! becomes that one's last subgroup (tabrail-groups nest); left of the name or on a tab's row it moves
//! among its siblings next to the row's group or the ancestor of it that is one (groups.rs;
//! tabrail-groups move-group): above it dragged up, below its tree dragged down. Either way with its
//! whole tree. No sibling there, Default dragged or nested into, or a group nested into its own tree:
//! nothing runs.
use crate::draw::Row;
use crate::groups::{self, Group};
use crate::menu::clean;
use crate::model::Tab;

/// The tmux command for group `src` let go over `row` at column `col`, if any.
pub fn drop_on(src: &str, (row, col): (Option<&Row>, usize), tabs: &[Tab], groups: &[Group], grouper: &str)
               -> Vec<String> {
    let (onto, header) = match row {
        Some(Row::Group(g, ..)) => (g.as_str(), true),
        Some(Row::Tab(id)) => match tabs.iter().find(|t| &t.id == id) {
            Some(t) => (t.group.as_str(), false),
            None => return vec![],
        },
        _ => return vec![],
    };
    let nest = header && groups.iter().find(|g| g.name == onto).is_some_and(|g| col >= g.depth + 5);
    let to = if nest { groups::nests(groups, src, onto).then_some(("nest", onto)) }
             else { groups::sibling(groups, src, onto).map(|o| ("move-group", o)) };
    match to {
        // only names that pass into the quotes unchanged: a cleaned name could match another group
        Some((verb, onto)) if src != "Default" && [src, onto].iter().all(|n| clean(n) == *n && !n.contains('`')) =>
            vec![format!("run-shell -b '{grouper} {verb} \"{src}\" \"{onto}\"'")],
        _ => vec![],
    }
}
