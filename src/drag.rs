//! The left button. Pressing and letting go on the same tab goes there; a press on a group's row folds
//! or unfolds it. Letting go over another tab puts the pressed tab right there, in that tab's group;
//! over a group's row it joins that group at its end. Nothing switches before the release: the
//! release must reach this same sidebar, which only the current tab shows.
use crate::control::Click;
use crate::draw::Row;
use crate::menu::clean;
use crate::model::Tab;
use std::collections::HashSet;

const REFRESH: &str = "set-buffer -b sidebar-refresh x"; // the other sidebars redraw on it

/// tmux commands for a left press or release; `drag` holds the pressed tab between the two.
pub fn left(click: &Click, row: Option<&Row>, tabs: &[Tab], folded: &HashSet<String>,
            drag: &mut Option<(usize, String)>) -> Vec<String> {
    if click.release {
        return match drag.take() {
            Some((from, src)) if from != click.row => drop_on(&src, row, tabs),
            Some((_, src)) => vec![format!("select-window -t {src}")],
            None => vec![],
        };
    }
    *drag = None;
    match row {
        Some(Row::Tab(id)) => {
            *drag = Some((click.row, id.clone()));
            vec![]
        }
        Some(Row::Group(name)) => {
            let mut f = folded.clone();
            if !f.remove(name) {
                f.insert(name.clone());
            }
            let f: Vec<&str> = f.iter().map(String::as_str).filter(|s| !s.is_empty()).collect();
            vec![format!("set -g @sidebar_fold '{}'", f.join("|")), REFRESH.into()]
        }
        _ => vec![],
    }
}

fn drop_on(src: &str, row: Option<&Row>, tabs: &[Tab]) -> Vec<String> {
    let Some(moved) = tabs.iter().find(|t| t.id == src) else { return vec![] };
    // the tab to land next to, and the group to join
    let (next_to, group, after) = match row {
        Some(Row::Tab(id)) if id != src => match tabs.iter().find(|t| &t.id == id) {
            Some(t) => (Some(t), t.group.clone(), t.index > moved.index),
            None => return vec![],
        },
        Some(Row::Group(g)) => (tabs.iter().filter(|t| &t.group == g && t.id != src).last(), g.clone(), true),
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
