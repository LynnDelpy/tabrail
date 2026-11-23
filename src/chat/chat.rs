//! What the chat does with keys and with claude's events, and the screen it shows: the
//! conversation above the input; in the reader bar, the bar's height (bar.rs).
use crate::bar::Bar;
use crate::input::Input;
use crate::keys::Key;
use crate::session::Session;
use crate::stream::Out;
use crate::view::{Entry, View};
use crate::Event;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub struct Chat {
    claude: Session,
    turn: Option<Instant>, // since when a turn runs
    view: View,
    input: Input,
    page: usize, // conversation rows on the screen, for PgUp and PgDn
    bar: Bar,
}

impl Chat {
    /// claude starts at once: its start-up is over by the first question.
    pub fn new(opts: Vec<String>, tx: Sender<Event>) -> Chat {
        let (view, input, bar) = (View::default(), Input::default(), Bar::new());
        let mut chat = Chat { claude: Session::new(opts, tx), turn: None, view, input, page: 1, bar };
        if let Err(e) = chat.claude.start() {
            chat.view.push(Entry::Dim(e));
        }
        chat
    }

    /// false: quit.
    pub fn key(&mut self, key: Key) -> bool {
        match key {
            key @ (Key::WheelUp | Key::WheelDown | Key::PageUp | Key::PageDown) => self.view.scroll(key, self.page),
            Key::Click(row, col) => self.bar.click(row, col),
            Key::Gone => return false,
            Key::Eof if self.input.is_empty() => return false,
            Key::Enter => {
                self.view.back = 0;
                self.send();
            }
            key => {
                self.view.back = 0;
                self.input.edit(key);
            }
        }
        true
    }

    /// While a turn runs, Enter sends nothing: the text waits in the input.
    fn send(&mut self) {
        if self.turn.is_some() {
            return;
        }
        let text = self.input.take();
        if text.trim().is_empty() {
            return;
        }
        self.view.push(Entry::Question(text.clone()));
        self.bar.sent();
        match self.claude.ask(&text) {
            Ok(()) => self.turn = self.turn.or(Some(Instant::now())),
            Err(e) => self.view.push(Entry::Dim(e)),
        }
    }

    pub fn claude(&mut self, gen: u32, out: Out) {
        match self.claude.heard(gen, out) {
            Some(Out::Text(text)) => self.view.text(&text),
            Some(Out::Tool(line)) => self.view.push(Entry::Dim(line)),
            Some(Out::End(None)) => self.turn = None,
            Some(Out::End(Some(error))) => self.failed(format!("claude failed: {error}")),
            Some(Out::Exit(tail)) if tail.is_empty() => self.failed("claude ended".into()),
            Some(Out::Exit(tail)) => self.failed(format!("claude ended: {tail}")),
            Some(Out::Session(_)) | None => {}
        }
    }

    /// One dim line, once: claude exiting after a failed turn adds none.
    fn failed(&mut self, line: String) {
        if !self.claude.ended {
            self.view.push(Entry::Dim(line));
        }
        (self.turn, self.claude.ended) = (None, true);
    }

    /// The screen at `(width, height)`: its rows and where the cursor goes in them.
    pub fn frame(&mut self, (width, height): (usize, usize)) -> (Vec<String>, (usize, usize)) {
        let (input, (row, col)) = self.bar.input(&self.input, (width, height), self.turn);
        self.page = height.saturating_sub(input.len()).max(1);
        let mut rows = self.view.rows(width, height.saturating_sub(input.len()));
        let cursor = (rows.len() + row, col);
        rows.extend(input);
        self.bar.fit((width, height), self.turn.is_some(), &mut self.input, &mut self.view);
        (rows, cursor)
    }
}
