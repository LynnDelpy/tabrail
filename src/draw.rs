//! One screen of the sidebar: groups on the terminal's own background with a faint tint behind each
//! group's row and its timer after its name (head.rs), ❯ and lavender for the current tab, the program
//! dimmed after its name and the status at the right edge, the clock at the bottom.
//! Colours: Catppuccin Mocha.
use crate::head;
use crate::model::{self, Tab};
use crate::timer::Timers;
use std::collections::HashSet;

pub enum Row {
    Nothing,
    Tab(String),
    Group(String),
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

pub fn screen(tabs: &[Tab], groups: &[(String, String, String)], folded: &HashSet<String>, timers: &Timers,
              clock: &str, (width, height): (usize, usize), now: i64) -> Screen {
    let mut lines = vec![String::new()];
    let mut rows = vec![Row::Nothing, Row::Nothing];
    let len = |s: &str| s.chars().count();
    for group in groups {
        let name = &group.0;
        let members: Vec<&Tab> = tabs.iter().filter(|t| &t.group == name).collect();
        // a group without tabs shows only while it has a timer: its time and menu stay in reach
        if members.is_empty() && timers.running(name).is_none() {
            continue;
        }
        let fold = folded.contains(name);
        lines.push(head::row(group, fold, members.len(), timers.label(name, now), width));
        rows.push(Row::Group(name.clone()));
        for tab in if fold { vec![] } else { members } {
            // program after the name, status at the edge; a long name is cut to make room for both, and the
            // program left out where fewer than 8 of the name's characters (all of a shorter one) would show
            let (state, kind) = model::status(tab, now);
            let state = if state.is_empty() { state } else { format!(" {state}") };
            let least = 3 + len(&tab.name).min(8) + 1 + len(&tab.program) + len(&state);
            let prog = if tab.program.is_empty() || least > width { String::new() } else { format!(" {}", tab.program) };
            let full = format!(" {} {}", if tab.current { '❯' } else { ' ' }, tab.name);
            let right = len(&prog) + len(&state);
            let room = width.saturating_sub(right);
            // a cut ending in a space loses it: no double space before the program
            let label = if len(&full) > room { full.chars().take(room).collect::<String>().trim_end().into() } else { full };
            let rest = " ".repeat(width.saturating_sub(len(&label) + right));
            let style = if tab.current { fg("current") + "\x1b[1m" } else { fg("tab") };
            lines.push(format!("{style}{label}\x1b[0m{}{prog}{rest}{}{state}\x1b[0m", fg("dim"), fg(kind)));
            rows.push(Row::Tab(tab.id.clone()));
        }
        lines.push(String::new());
        rows.push(Row::Nothing);
    }
    lines.resize(height.saturating_sub(1), String::new());
    lines.push(format!("{}{}\x1b[0m", fg("dim"), fit(&format!("{clock:^width$}"), width)));
    Screen { lines, rows }
}
