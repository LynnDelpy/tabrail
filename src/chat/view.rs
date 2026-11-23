//! The conversation: questions, answers and dim lines (tool calls, errors), wrapped at the pane's
//! width, and the rows a height shows of it, scrolled back from the bottom.

pub enum Entry {
    Question(String),
    Answer(String),
    Dim(String), // one row, cut at the width
}

#[derive(Default)]
pub struct View {
    entries: Vec<Entry>,
    pub back: usize, // rows scrolled back from the bottom; rows() stops it at the top
}

impl View {
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Answer text arriving: onto the answer being written, the view back at the bottom.
    pub fn text(&mut self, text: &str) {
        self.back = 0;
        match self.entries.last_mut() {
            Some(Entry::Answer(answer)) => answer.push_str(text),
            _ => self.entries.push(Entry::Answer(text.into())),
        }
    }

    /// Exactly `height` rows; at the top while the conversation is shorter.
    pub fn rows(&mut self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for entry in &self.entries {
            match entry {
                Entry::Question(q) => lines.extend(wrap(&format!("❯ {q}"), width).iter().map(|l| format!("\x1b[1m{l}\x1b[0m"))),
                Entry::Answer(a) => lines.extend(wrap(a, width)),
                Entry::Dim(d) => lines.push(format!("\x1b[2m{}\x1b[0m", clean(d).into_iter().take(width).collect::<String>())),
            }
        }
        let top = lines.len().saturating_sub(height);
        self.back = self.back.min(top);
        lines.drain(..top - self.back);
        lines.resize(height, String::new());
        lines
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
