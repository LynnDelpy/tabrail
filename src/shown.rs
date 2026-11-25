//! Whether the sidebar's tab is shown: its window is its session's current one and a terminal (not a
//! control client like the sidebars, counted in `session_attached` too) is attached to the session.
//! Hidden, it neither draws nor ticks: `%session-window-changed` names the new current window; a
//! terminal (not a sidebar) attaching, detaching or switching sessions costs one `list-clients` of
//! the terminals, and a `%layout-change` of its own tab one `display` to keep its width (width.rs),
//! and a `resize-pane` if it is off.
use crate::control::Tmux;

#[derive(PartialEq)]
pub enum Wake {
    Shown,  // shown, as before
    Again,  // shown after being hidden: draw at once
    Layout, // hidden, its tab's layout changed: keep the width only
}

pub struct Shown {
    session: String,    // $1
    window: String,     // @4
    current: bool,      // its window is the session's current one
    seen: Option<bool>, // a terminal is attached to the session; None: not asked yet
    layout: bool,       // its tab's layout changed
}

impl Shown {
    pub fn new(session: &str, window: &str) -> Shown {
        Shown { session: session.into(), window: window.into(), current: false, seen: None, layout: false }
    }

    /// Ask tmux which window the session's terminals show: one line each, all the same window.
    fn ask(&mut self, t: &mut Tmux) {
        let terminals = "-f '#{!:#{client_control_mode}}' -F '#{window_id}'";
        self.clients(&t.ask(&format!("list-clients -t {} {terminals}", self.session)));
    }

    /// Follow the notifications that came in (`t.notes`); while the tab is hidden, wait for more
    /// without a tick until it is shown, clicked (shown after all) or its layout changed.
    pub fn wait(&mut self, t: &mut Tmux) -> Wake {
        let mut wake = Wake::Shown;
        while !self.follow(t) && t.clicks.is_empty() {
            if std::mem::take(&mut self.layout) {
                return Wake::Layout;
            }
            if t.notes.is_empty() {
                t.wait(None);
            }
            wake = Wake::Again;
        }
        self.layout = false; // shown: the frame keeps the width
        wake
    }

    fn follow(&mut self, t: &mut Tmux) -> bool {
        let ask = std::mem::take(&mut t.notes).iter().fold(false, |ask, line| self.note(line) || ask);
        if ask || self.seen.is_none() {
            self.ask(t);
        }
        self.is()
    }

    fn is(&self) -> bool { self.current && self.seen == Some(true) }

    /// What `list-clients` replied; with no terminal, the current window is the last one a note named.
    fn clients(&mut self, lines: &[String]) {
        self.seen = Some(!lines.is_empty());
        self.current = lines.iter().any(|w| *w == self.window) || lines.is_empty() && self.current;
    }

    /// One notification line; true if the session's terminals must be asked again.
    fn note(&mut self, line: &str) -> bool {
        let mut w = line.split(' ');
        match w.next() {
            Some("%session-window-changed") if w.next() == Some(self.session.as_str()) => {
                self.current = w.next() == Some(self.window.as_str());
                false
            }
            Some("%layout-change") if w.next() == Some(self.window.as_str()) => {
                self.layout = true;
                false
            }
            // control clients (the sidebars) are named client-PID, terminals by their tty
            Some("%client-session-changed" | "%client-detached") => {
                w.next().is_some_and(|c| !c.starts_with("client-"))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
