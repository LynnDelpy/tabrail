//! A group's header row: one column in per level (groups.rs), fold arrow, icon and name, the group's
//! timer right after the name, then a folded group's tab count, bold on the group's tint to the row's
//! end. At the end a dim + one column in from the edge adds a subgroup (none for Default), and without
//! a timer a dim ▷ two left of it starts one; neither for a name clean() changes (it cannot travel
//! through tmux's command strings: drag.rs folds there). The count goes first (shown whole or not at
//! all), then the name is cut; never the time, the ▷ or the +.
use crate::draw::{fg, fit, rgb};
use crate::groups::Group;
use crate::menu::clean;

/// The ▷: Nerd Font nf-cod-play, an outlined triangle centred on the text (the terminal's MesloLGS NF
/// draws its own ▷ on the baseline, too low); one column in tmux.
pub const START: &str = "\u{eb2c}";

/// The row at `width` for a group of `n` tabs (its subgroups' included), `fold`ed or not; `time` the
/// timer's label and colour (timer.rs). The time's colour sets the foreground alone: the tint and bold
/// go on past it. With it the first and last column of the time (1-based, one per char, its marker
/// included) for a click, without one the ▷'s column as both; then the +'s column.
pub fn row(Group { name, tint, icon, depth }: &Group, fold: bool, n: usize, time: Option<(String, &str)>,
           width: usize) -> (String, Option<(usize, usize)>, Option<usize>) {
    let len = |s: &str| s.chars().count();
    let clean = clean(name) == *name;
    let start = if time.is_none() && clean { START } else { "" };
    let plus = if name == "Default" { " " } else { "+" }; // Default's column blank: its ▷ where the others' is
    // never cut: a space and the ▷, a space, the + and the pad off the pane's border
    let end = if !clean { String::new() } else if start.is_empty() { format!(" {plus} ") } else { format!(" {start} {plus} ") };
    let (time, kind) = time.unwrap_or_default();
    let gap = if time.is_empty() { "" } else { " " }; // in the name's colour: the time's starts at its marker
    let right = len(&time) + len(&end);
    let full = format!("{}{} {icon} {name}", " ".repeat(*depth), if fold { "▸" } else { "▾" });
    let room = width.saturating_sub(len(gap) + right);
    // a cut ending in a space loses it: no double space before the time
    let head = if len(&full) > room { full.chars().take(room).collect::<String>().trim_end().into() } else { full };
    let head = head + gap;
    let count = if fold && n > 0 { format!("  {n} tab{}", if n == 1 { "" } else { "s" }) } else { String::new() };
    let room = width.saturating_sub(len(&head) + right);
    let count = fit(if len(&count) <= room { &count } else { "" }, room);
    let tint = tint.strip_prefix('#').map(|h| rgb(48, h)).unwrap_or_default();
    let at = len(&head) + len(&time) + len(&count); // the end's first column, less one
    let span = (!time.is_empty()).then(|| (len(&head) + 1, len(&head) + len(&time)));
    let span = span.or((!start.is_empty()).then_some((at + 2, at + 2)));
    let plus = (!end.is_empty() && plus == "+").then_some(at + len(&end) - 1);
    let (text, dim) = (fg("text"), fg("dim"));
    (format!("{tint}{text}\x1b[1m{head}{}{time}{text}{count}{dim}{end}\x1b[0m", fg(kind)), span, plus)
}

#[cfg(test)]
mod tests;
