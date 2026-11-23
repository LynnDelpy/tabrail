//! The input at the bottom: `❯ ` and the text being typed, wrapped at the pane's full width over as
//! many rows as it needs, the cursor in it; empty, a dim `type to ask`. At its last row's right end a
//! dim `… 12s` while a turn runs, and in the reader bar the buttons `▲ ▼` after it (bar.rs).
use crate::keys::Key;
use std::time::Instant;

#[derive(Default)]
pub struct Input {
    text: Vec<char>,
    at: usize,         // the cursor, in characters
    pub changed: bool, // the text changed since the bar's height was last looked at (bar.rs)
}

impl Input {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The text for Enter; the input is empty again.
    pub fn take(&mut self) -> String {
        (self.at, self.changed) = (0, self.changed || !self.text.is_empty());
        self.text.drain(..).collect()
    }

    pub fn edit(&mut self, key: Key) {
        let len = self.text.len();
        match key {
            Key::Text(t) => {
                for c in t.chars().filter(|c| !c.is_control()) {
                    self.text.insert(self.at, c);
                    self.at += 1;
                }
            }
            Key::Back if self.at > 0 => {
                self.at -= 1;
                self.text.remove(self.at);
            }
            Key::Left => self.at = self.at.saturating_sub(1),
            Key::Right => self.at = (self.at + 1).min(self.text.len()),
            Key::Home => self.at = 0,
            Key::End => self.at = self.text.len(),
            Key::Kill => {
                self.text.drain(..self.at);
                self.at = 0;
            }
            Key::Clear => drop(self.take()),
            _ => {}
        }
        self.changed |= self.text.len() != len;
    }

    /// All its rows at `width`, however many the screen shows.
    pub fn height(&self, width: usize) -> usize {
        (self.text.len() + 2) / width.max(1) + 1
    }

    /// At most `max` rows (the rows around the cursor of a longer text, its tail while the cursor is
    /// at the end) and the cursor's row and column in them. At the last row's right edge where it
    /// fits: the timer of a turn running since `turn`, then with `buttons` ▲ ▼ (they first where
    /// both do not fit); and the column of ▲ when they are drawn.
    pub fn rows(&self, width: usize, max: usize, turn: Option<Instant>, buttons: bool)
                -> (Vec<String>, (usize, usize), Option<usize>) {
        let w = width.max(1);
        let cells: Vec<char> = "❯ ".chars().chain(self.text.iter().copied()).collect();
        let (all, at) = (cells.len() / w + 1, self.at + 2);
        let shown = all.min(max.max(1));
        let first = (all - shown).min(at / w);
        let mut rows: Vec<String> = (first..first + shown).map(|r| cells.iter().skip(r * w).take(w).collect()).collect();
        if self.text.is_empty() {
            let hint: String = "type to ask".chars().take(w.saturating_sub(2)).collect();
            rows[0] = format!("❯ \x1b[2m{hint}\x1b[0m");
        }
        let used = if self.text.is_empty() { 13 } else { (cells.len() - (all - 1) * w).min(w) };
        let room = w.saturating_sub(used + 1);
        let took = |s: u64| if s < 60 { format!("… {s}s") } else { format!("… {}m{:02}s", s / 60, s % 60) };
        let timer = turn.map(|t| took(t.elapsed().as_secs())).unwrap_or_default();
        let dim = |t: &str| format!("\x1b[2m{t}\x1b[0m");
        // what goes there and its width: the timer, the buttons after it, or alone where both do not fit
        let right = match (timer.chars().count(), buttons) {
            (0, false) => None,
            (t, false) => Some((dim(&timer), t)),
            (t, true) if t > 0 && t + 4 <= room => Some((format!("{} ▲ ▼", dim(&timer)), t + 4)),
            (_, true) => Some(("▲ ▼".to_string(), 3)),
        };
        let cursor = (at / w - first, at % w);
        match right.filter(|&(_, n)| n <= room && first + shown == all) {
            Some((text, n)) => {
                rows[shown - 1] += &format!("{}{text}", " ".repeat(room - n + 1));
                (rows, cursor, buttons.then(|| w - 3))
            }
            None => (rows, cursor, None),
        }
    }
}
