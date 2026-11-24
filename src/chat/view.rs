//! The conversation: questions, answers and dim lines (errors), wrapped at the pane's
//! width, and the rows a height shows of it, scrolled back from the bottom.
use crate::keys::Key;

pub enum Entry {
    Question(String),
    Answer(String),
    Dim(String), // one row, cut at the width
}

#[derive(Default)]
pub struct View {
    entries: Vec<Entry>,
    pub back: usize,   // rows scrolled back from the bottom; rows() stops it at the top
    pub changed: bool, // an entry or answer text came since the bar's height was last looked at (bar.rs)
}

impl View {
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.changed = true;
    }

    /// Answer text arriving: onto the answer being written, the view back at the bottom.
    pub fn text(&mut self, text: &str) {
        (self.back, self.changed) = (0, true);
        match self.entries.last_mut() {
            Some(Entry::Answer(answer)) => answer.push_str(text),
            _ => self.entries.push(Entry::Answer(text.into())),
        }
    }

    /// A tool call: the answer so far was narration; only answers show, no tool calls (Lynn, 2026-11-24).
    pub fn narration(&mut self) {
        self.changed |= matches!(self.entries.last(), Some(Entry::Answer(_))) && self.entries.pop().is_some();
    }

    /// The wheel 3 rows, PgUp and PgDn a `page` of rows, back from the bottom or towards it.
    pub fn scroll(&mut self, key: Key, page: usize) {
        self.back = match key {
            Key::WheelUp => self.back + 3,
            Key::WheelDown => self.back.saturating_sub(3),
            Key::PageUp => self.back + page,
            _ => self.back.saturating_sub(page),
        };
    }

    /// Exactly `height` rows; at the top while the conversation is shorter.
    pub fn rows(&mut self, width: usize, height: usize) -> Vec<String> {
        let mut lines: Vec<String> = self.entries.iter().flat_map(|entry| lines(entry, width)).collect();
        let top = lines.len().saturating_sub(height);
        self.back = self.back.min(top);
        lines.drain(..top - self.back);
        lines.resize(height, String::new());
        lines
    }

    /// The rows of the latest question and what came after it (its answer), at `width`.
    pub fn latest(&self, width: usize) -> usize {
        let from = self.entries.iter().rposition(|e| matches!(e, Entry::Question(_))).unwrap_or(0);
        self.entries[from..].iter().map(|entry| lines(entry, width).len()).sum()
    }
}

fn lines(entry: &Entry, width: usize) -> Vec<String> {
    match entry {
        Entry::Question(q) => wrap(&format!("❯ {q}"), width).iter().map(|l| format!("\x1b[1m{l}\x1b[0m")).collect(),
        Entry::Answer(a) => wrap(a, width),
        Entry::Dim(d) => vec![format!("\x1b[2m{}\x1b[0m", clean(d).into_iter().take(width).collect::<String>())],
    }
}

/// Tabs as four spaces, other control characters (escapes in an answer) left out.
fn clean(text: &str) -> Vec<char> {
    text.replace('\t', "    ").chars().filter(|c| !c.is_control()).collect()
}

/// Paragraphs (split at newlines, blank ones left out: no row is spent on nothing) broken at the
/// last space before the width; a longer word is broken where the width ends.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let w = width.max(1);
    let mut rows = Vec::new();
    for para in text.split('\n').filter(|p| !p.trim().is_empty()) {
        let (c, mut s) = (clean(para), 0);
        while c.len() - s > w {
            let room = &c[s..=s + w];
            let words = |i: &usize| room[..*i].iter().any(|&ch| ch != ' ');
            let cut = room.iter().rposition(|&ch| ch == ' ').filter(words).unwrap_or(w);
            rows.push(room[..cut].iter().collect::<String>().trim_end().to_string());
            s += cut;
            while c.get(s) == Some(&' ') {
                s += 1;
            }
        }
        if s < c.len() {
            rows.push(c[s..].iter().collect());
        }
    }
    rows
}
