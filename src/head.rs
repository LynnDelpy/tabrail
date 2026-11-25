//! A group's header row: fold arrow, icon and name, the group's timer right after the name, then a
//! folded group's tab count, bold on the group's tint to the row's end. The time is never cut: the
//! count goes first (shown whole or not at all), then the name is cut.
use crate::draw::{fg, fit, rgb};

/// The row at `width` for a group of `n` tabs, `fold`ed or not; `time` the timer's label and colour
/// (timer.rs). The time's colour sets the foreground alone: the tint and bold go on past it. With it
/// the first and last column of the time (1-based, one per char, its marker included) for a click.
pub fn row((name, tint, icon): &(String, String, String), fold: bool, n: usize, time: Option<(String, &str)>,
           width: usize) -> (String, Option<(usize, usize)>) {
    let len = |s: &str| s.chars().count();
    let (time, kind) = time.unwrap_or_default();
    let gap = if time.is_empty() { "" } else { " " }; // in the name's colour: the time's starts at its marker
    let full = format!("{} {icon} {name}", if fold { "▸" } else { "▾" });
    let room = width.saturating_sub(len(gap) + len(&time));
    // a cut ending in a space loses it: no double space before the time
    let head = if len(&full) > room { full.chars().take(room).collect::<String>().trim_end().into() } else { full };
    let head = head + gap;
    let count = if fold && n > 0 { format!("  {n} tab{}", if n == 1 { "" } else { "s" }) } else { String::new() };
    let room = width.saturating_sub(len(&head) + len(&time));
    let count = fit(if len(&count) <= room { &count } else { "" }, room);
    let tint = tint.strip_prefix('#').map(|h| rgb(48, h)).unwrap_or_default();
    let span = (!time.is_empty()).then(|| (len(&head) + 1, len(&head) + len(&time)));
    (format!("{tint}{}\x1b[1m{head}{}{time}{}{count}\x1b[0m", fg("text"), fg(kind), fg("text")), span)
}

#[cfg(test)]
mod tests;
