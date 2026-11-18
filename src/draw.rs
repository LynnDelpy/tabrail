//! One screen of the sidebar: groups on the terminal's own background with a faint tint behind each
//! group's row, ❯ and lavender for the current tab, status on the right, the time at the bottom.
//! Colours: Catppuccin Mocha.
use crate::model::{self, Tab};
use std::collections::HashSet;

pub enum Row {
    Nothing,
    Tab(String),
    Group(String),
}

fn fg(key: &str) -> String {
    let hex = match key {
        "text" => "cdd6f4", "current" => "b4befe", "tab" => "a6adc8", "busy" => "f9e2af",
        "wait" | "fail" => "f38ba8", "done" => "a6e3a1", _ => "6c7086",
    };
    rgb(38, hex)
}

fn rgb(layer: u8, hex: &str) -> String {
    let c = |i: usize| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    format!("\x1b[{layer};2;{};{};{}m", c(0), c(2), c(4))
}

/// Cut or pad to exactly `width` characters.
fn fit(text: &str, width: usize) -> String {
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

pub fn screen(tabs: &[Tab], groups: &[(String, String, String)], folded: &HashSet<String>,
              clock: &str, (width, height): (usize, usize), now: i64) -> Screen {
    let mut lines = vec![String::new()];
    let mut rows = vec![Row::Nothing, Row::Nothing];
    for (name, tint, icon) in groups {
        let members: Vec<&Tab> = tabs.iter().filter(|t| &t.group == name).collect();
        if members.is_empty() {
            continue;
        }
        let fold = folded.contains(name);
        let mut head = format!(" {} {icon} {name}", if fold { "▸" } else { "▾" });
        if fold {
            head += &format!("  {} tab{}", members.len(), if members.len() == 1 { "" } else { "s" });
        }
        let tint = tint.strip_prefix('#').map(|h| rgb(48, h)).unwrap_or_default();
        lines.push(format!("{tint}{}\x1b[1m{}\x1b[0m", fg("text"), fit(&head, width)));
        rows.push(Row::Group(name.clone()));
        for tab in if fold { vec![] } else { members } {
            // the status right after the name; a long name is cut to make room for it
            let (state, kind) = model::status(tab, now);
            let mark = if tab.current { '❯' } else { ' ' };
            let gap = if state.is_empty() { 0 } else { 1 };
            let room = width.saturating_sub(state.chars().count() + gap);
            let label: String = format!("  {mark} {} {}", tab.index, tab.name).chars().take(room).collect();
            let rest = width.saturating_sub(label.chars().count() + gap + state.chars().count());
            let style = if tab.current { fg("current") + "\x1b[1m" } else { fg("tab") };
            lines.push(format!("{style}{label}\x1b[0m{}{}{state}\x1b[0m{}", " ".repeat(gap), fg(kind), " ".repeat(rest)));
            rows.push(Row::Tab(tab.id.clone()));
        }
        lines.push(String::new());
        rows.push(Row::Nothing);
    }
    lines.resize(height.saturating_sub(1), String::new());
    lines.push(format!("{}{}\x1b[0m", fg("dim"), fit(&format!("{clock:^width$}"), width)));
    Screen { lines, rows }
}
