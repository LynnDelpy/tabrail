//! The left button, acting on the release. Pressing and letting go on the same tab goes there, on the
//! same group's row folds or unfolds it, both on its time or the ▷ a row without one has (the columns
//! draw reports) stops or starts its timer (tabrail-timer toggle), both on its + opens the name field
//! for a new subgroup of it (tabrail-name sub: this sidebar's control client has no screen for the
//! popup, so tabrail-name finds the terminal showing its tab by its pane). Letting go over another tab
//! puts the pressed tab right there, in that tab's group; over a group's row it joins that group at
//! its end. A group's row let go over another row moves the group with its tree in the groups file,
//! nested into a group or among its siblings (group.rs). Nothing switches before the release: the
//! release must reach this same sidebar, which only the current tab shows.
use crate::draw::Row;
use crate::groups::Group;
use crate::input::Click;
use crate::menu::{clean, Place};
use crate::model::Tab;
use std::collections::HashSet;

const REFRESH: &str = "set-buffer -b sidebar-refresh x"; // the other sidebars redraw on it

/// tmux commands for a left press or release; `drag` holds the pressed row, its column and what was
/// there between the two.
pub fn left(click: &Click, row: Option<&Row>, tabs: &[Tab], groups: &[Group], folded: &HashSet<String>, p: &Place,
            drag: &mut Option<(usize, usize, Row)>) -> Vec<String> {
    if !click.release {
        *drag = match row {
            Some(r @ (Row::Tab(_) | Row::Group(..))) => Some((click.row, click.col, r.clone())),
            _ => None,
        };
        return vec![];
    }
    let on = |col: usize, time: Option<(usize, usize)>| time.is_some_and(|(a, b)| (a..=b).contains(&col));
    let same = |name: &str| matches!(row, Some(Row::Group(g, ..)) if g == name); // the pressed group's row
    match drag.take() {
        Some((from, _, Row::Tab(src))) if from != click.row => drop_on(&src, row, tabs),
        Some((_, _, Row::Tab(src))) => vec![format!("select-window -t {src}")],
        // both on the time or ▷ as it was at the press; a name clean() changes folds, as its menu has no
        // timer items (the cleaned name would key another group's timer); tabrail-timer redraws all
        Some((from, col, Row::Group(src, time, _))) if from == click.row && on(col, time) && on(click.col, time)
            && clean(&src) == src => vec![format!("run-shell -b \"{} toggle '{src}'\"", p.timer)],
        // both on the + as it was at the press, on that group's row (no + on Default or a name clean()
        // changes); no CLIENT: tabrail-name finds the terminal by the pane, %12 written p12
        Some((from, col, Row::Group(src, _, Some(plus)))) if from == click.row && col == plus && click.col == plus
            && same(&src) => vec![format!("run-shell -b \"{} sub '{src}' '' '' {}\"", p.namer, p.pane.replace('%', "p"))],
        Some((from, _, Row::Group(src, ..))) if from == click.row => fold(&src, folded),
        Some((_, _, Row::Group(src, ..))) => group::drop_on(&src, (row, click.col), tabs, groups, p.grouper),
        _ => vec![],
    }
}

fn fold(name: &str, folded: &HashSet<String>) -> Vec<String> {
    let mut f = folded.clone();
    if !f.remove(name) {
        f.insert(name.to_string());
    }
    let f: Vec<&str> = f.iter().map(String::as_str).filter(|s| !s.is_empty()).collect();
    vec![format!("set -g @sidebar_fold '{}'", f.join("|")), REFRESH.into()]
}

fn drop_on(src: &str, row: Option<&Row>, tabs: &[Tab]) -> Vec<String> {
    let Some(moved) = tabs.iter().find(|t| t.id == src) else { return vec![] };
    // the tab to land next to, and the group to join
    let (next_to, group, after) = match row {
        Some(Row::Tab(id)) if id != src => match tabs.iter().find(|t| &t.id == id) {
            Some(t) => (Some(t), t.group.clone(), t.index > moved.index),
            None => return vec![],
        },
        Some(Row::Group(g, ..)) => (tabs.iter().filter(|t| &t.group == g && t.id != src).last(), g.clone(), true),
        _ => return vec![],
    };
    let mut cmds = vec![format!("set -w -t {src} @group '{}'", clean(&group))];
    if let Some(t) = next_to {
        cmds.push(format!("move-window {} -s {src} -t {}", if after { "-a" } else { "-b" }, t.id));
        cmds.push(format!("move-window -r -t {src}")); // numbers 1, 2, 3 … again
    }
    cmds.push(REFRESH.into());
    cmds
}

mod group;
#[cfg(test)]
mod tests;
