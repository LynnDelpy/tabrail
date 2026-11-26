//! One screen of the sidebar: the group tree (groups.rs) on the terminal's own background, each level
//! one column further right, a faint tint behind each group's row and its timer after its name
//! (head.rs), a parent's own tabs (row.rs) above its subgroups, the clock at the bottom.
//! Colours: Catppuccin Mocha.
use crate::groups::{self, Group};
use crate::model::Tab;
use crate::timer::Timers;
use crate::{head, row};
use std::collections::HashSet;

pub enum Row {
    Nothing,
    Tab(String),
    Group(String, Option<(usize, usize)>), // and the time's or ▷'s first and last column (head.rs), if shown
}

pub(crate) fn fg(key: &str) -> String {
    let hex = match key {
        "text" => "cdd6f4", "current" => "b4befe", "tab" => "a6adc8", "busy" => "f9e2af",
        "wait" | "fail" => "f38ba8", "done" => "a6e3a1", _ => "6c7086",
    };
    rgb(38, hex)
}

pub(crate) fn rgb(layer: u8, hex: &str) -> String {
    let c = |i: usize| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    format!("\x1b[{layer};2;{};{};{}m", c(0), c(2), c(4))
}

/// Cut or pad to exactly `width` characters.
pub(crate) fn fit(text: &str, width: usize) -> String {
    let cut: String = text.chars().take(width).collect();
    format!("{cut:<width$}")
}

pub struct Screen {
    pub lines: Vec<String>,
    pub rows: Vec<Row>,
}

/// What to write to turn `shown` into `lines`: only the lines that differ, so a menu tmux shows
/// over the sidebar is not painted over by lines that stayed the same.
pub fn changes(shown: &mut Vec<String>, lines: Vec<String>) -> String {
    let mut text = String::new();
    if shown.len() != lines.len() {
        shown.clear();
        text.push_str("\x1b[2J");
    }
    for (i, line) in lines.iter().enumerate() {
        if shown.get(i) != Some(line) {
            text += &format!("\x1b[{};1H{line}\x1b[K", i + 1);
        }
    }
    *shown = lines;
    text
}

pub fn screen(tabs: &[Tab], groups: &[Group], folded: &HashSet<String>, timers: &Timers,
              clock: &str, (width, height): (usize, usize), now: i64) -> Screen {
    let mut lines = vec![String::new()];
    let mut rows = vec![Row::Nothing, Row::Nothing];
    let n: Vec<usize> = groups.iter().map(|g| tabs.iter().filter(|t| t.group == g.name).count()).collect();
    let mut i = 0;
    while i < groups.len() {
        let (group, end) = (&groups[i], groups::subtree(groups, i).end);
        // a group shows while it or one below it holds a tab or has a timer: its time and menu stay in
        // reach; a folded one hides the groups below it and counts their tabs with its own
        if (i..end).all(|j| n[j] == 0 && timers.running(&groups[j].name).is_none()) {
            i = end;
            continue;
        }
        if group.depth == 0 && lines.len() > 1 { // one blank line after each top-level tree
            lines.push(String::new());
            rows.push(Row::Nothing);
        }
        let fold = folded.contains(&group.name);
        let (line, time) = head::row(group, fold, n[i..end].iter().sum(), timers.label(&group.name, now), width);
        lines.push(line);
        rows.push(Row::Group(group.name.clone(), time));
        for tab in tabs.iter().filter(|t| !fold && t.group == group.name) {
            lines.push(row::tab(tab, group.depth, width, now));
            rows.push(Row::Tab(tab.id.clone()));
        }
        i = if fold { end } else { i + 1 };
    }
    if lines.len() > 1 {
        lines.push(String::new());
        rows.push(Row::Nothing);
    }
    lines.resize(height.saturating_sub(1), String::new());
    lines.push(format!("{}{}\x1b[0m", fg("dim"), fit(&format!("{clock:^width$}"), width)));
    Screen { lines, rows }
}
