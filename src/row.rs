//! A tab's row in the sidebar: ❯ and lavender for the current tab, the program dimmed after its name
//! and the status at the right edge, the whole row as many columns in as its group is deep (groups.rs).
use crate::draw::fg;
use crate::model::{self, Tab};

/// The row of `tab` at `width`, `indent` columns in (the indent counts as part of the label).
pub fn tab(tab: &Tab, indent: usize, width: usize, now: i64) -> String {
    let len = |s: &str| s.chars().count();
    // program after the name, status at the edge; a long name is cut to make room for both, and the
    // program left out where fewer than 8 of the name's characters (all of a shorter one) would show
    let (state, kind) = model::status(tab, now);
    let state = if state.is_empty() { state } else { format!(" {state}") };
    let least = indent + 3 + len(&tab.name).min(8) + 1 + len(&tab.program) + len(&state);
    let prog = if tab.program.is_empty() || least > width { String::new() } else { format!(" {}", tab.program) };
    let full = format!("{} {} {}", " ".repeat(indent), if tab.current { '❯' } else { ' ' }, tab.name);
    let right = len(&prog) + len(&state);
    let room = width.saturating_sub(right);
    // a cut ending in a space loses it: no double space before the program
    let label = if len(&full) > room { full.chars().take(room).collect::<String>().trim_end().into() } else { full };
    let rest = " ".repeat(width.saturating_sub(len(&label) + right));
    let style = if tab.current { fg("current") + "\x1b[1m" } else { fg("tab") };
    format!("{style}{label}\x1b[0m{}{prog}{rest}{}{state}\x1b[0m", fg("dim"), fg(kind))
}

#[cfg(test)]
mod tests;
