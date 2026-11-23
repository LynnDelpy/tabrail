//! The input at the bottom: `❯ ` and the text being typed, wrapped at the pane's full width over as
//! many rows as it needs, the cursor in it; empty, a dim `type to ask`.
use crate::keys::Key;

#[derive(Default)]
pub struct Input {
    text: Vec<char>,
    at: usize, // the cursor, in characters
}

impl Input {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The text for Enter; the input is empty again.
    pub fn take(&mut self) -> String {
        self.at = 0;
        self.text.drain(..).collect()
    }

    pub fn edit(&mut self, key: Key) {
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
    }

    /// At most `max` rows (the rows around the cursor of a longer text, its tail while the cursor is
    /// at the end) and the cursor's row and column in them; `right` dim at the last row's right edge
    /// where it fits.
    pub fn rows(&self, width: usize, max: usize, right: &str) -> (Vec<String>, (usize, usize)) {
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
        if !right.is_empty() && first + shown == all && right.chars().count() <= room {
            let last = rows.len() - 1;
            rows[last] += &format!("{}\x1b[2m{right}\x1b[0m", " ".repeat(room - right.chars().count() + 1));
        }
        (rows, (at / w - first, at % w))
    }
}
