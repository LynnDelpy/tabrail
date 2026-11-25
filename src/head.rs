//! A group's header row: fold arrow, icon and name, the group's timer right after the name, then a
//! folded group's tab count, bold on the group's tint to the row's end. Without a timer a dim ▷ one
//! column in from the edge starts one (none for a name clean() changes: no timer, drag.rs folds there).
//! Neither the time nor the ▷ is cut: the count goes first (shown whole or not at all), then the name.
use crate::draw::{fg, fit, rgb};
use crate::menu::clean;

/// The ▷: Nerd Font nf-cod-play, an outlined triangle centred on the text (the terminal's MesloLGS NF
/// draws its own ▷ on the baseline, too low); one column in tmux.
pub const START: &str = "\u{eb2c}";

/// The row at `width` for a group of `n` tabs, `fold`ed or not; `time` the timer's label and colour
/// (timer.rs). The time's colour sets the foreground alone: the tint and bold go on past it. With it
/// the first and last column of the time (1-based, one per char, its marker included) for a click,
/// without one the ▷'s column as both.
pub fn row((name, tint, icon): &(String, String, String), fold: bool, n: usize, time: Option<(String, &str)>,
           width: usize) -> (String, Option<(usize, usize)>) {
    let len = |s: &str| s.chars().count();
    let start = if time.is_none() && clean(name) == *name { START } else { "" };
    let pad = if start.is_empty() { "" } else { " " }; // the ▷ off the pane's border
    let (time, kind) = time.unwrap_or_default();
    let gap = if time.is_empty() { "" } else { " " }; // in the name's colour: the time's starts at its marker
    let right = len(&time) + 2 * len(start) + len(pad); // never cut: the time, or a space, the ▷ and its pad
    let full = format!("{} {icon} {name}", if fold { "▸" } else { "▾" });
    let room = width.saturating_sub(len(gap) + right);
    // a cut ending in a space loses it: no double space before the time
    let head = if len(&full) > room { full.chars().take(room).collect::<String>().trim_end().into() } else { full };
    let head = head + gap;
    let count = if fold && n > 0 { format!("  {n} tab{}", if n == 1 { "" } else { "s" }) } else { String::new() };
    let room = width.saturating_sub(len(&head) + right);
    let count = fit(if len(&count) <= room { &count } else { "" }, room + len(start));
    let tint = tint.strip_prefix('#').map(|h| rgb(48, h)).unwrap_or_default();
    let span = |at: usize, s: &str| (!s.is_empty()).then(|| (at + 1, at + len(s)));
    let span = span(len(&head), &time).or(span(len(&head) + len(&count), start));
    let (text, dim) = (fg("text"), fg("dim"));
    (format!("{tint}{text}\x1b[1m{head}{}{time}{text}{count}{dim}{start}{pad}\x1b[0m", fg(kind)), span)
}

#[cfg(test)]
mod tests;
