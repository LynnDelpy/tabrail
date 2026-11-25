//! One control-mode connection to tmux (`tmux -C`). tmux pushes what changed (tab switched, added,
//! renamed, closed, resized, and %paste-buffer-changed from `tabrail-tmux refresh`), and commands go
//! over the same pipe, so nothing is spawned while the sidebar runs.
use crate::input::Click;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

pub enum Event {
    Click(Click),
    Note(String), // a notification line (`%session-window-changed $1 @4`)
    Reply(Vec<String>),
    Gone,
}

pub struct Tmux {
    input: ChildStdin,
    pub rx: Receiver<Event>,
    pub tx: Sender<Event>,
    pub clicks: Vec<Click>,
    pub notes: Vec<String>,
}

impl Tmux {
    /// Attach to `session` without a say in window sizes and without pane output.
    pub fn attach(session: &str) -> std::io::Result<Tmux> {
        let mut child = Command::new("tmux")
            .args(["-C", "attach-session", "-f", "ignore-size,no-output", "-t", session])
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
            .spawn()?;
        let (tx, rx) = channel();
        let (out, events) = (child.stdout.take().expect("stdout is piped"), tx.clone());
        thread::spawn(move || read(out, events));
        Ok(Tmux { input: child.stdin.take().expect("stdin is piped"), rx, tx, clicks: vec![], notes: vec![] })
    }

    /// Send one command (one per line: each gets its own reply) and wait for its reply; clicks and
    /// notifications that come in meanwhile wait in `clicks` and `notes`.
    pub fn ask(&mut self, cmd: &str) -> Vec<String> {
        let _ = writeln!(self.input, "{cmd}").and_then(|_| self.input.flush());
        loop {
            match self.rx.recv() {
                Ok(Event::Reply(lines)) => return lines,
                event => self.keep(event.unwrap_or(Event::Gone)),
            }
        }
    }

    /// Wait for the next notification or click, at most `timeout` (None: as long as it takes); what
    /// piles up meanwhile comes along. True if something came.
    pub fn wait(&mut self, timeout: Option<Duration>) -> bool {
        let first = match timeout.map(|t| self.rx.recv_timeout(t)) {
            Some(Err(RecvTimeoutError::Timeout)) => return false,
            Some(event) => event.ok(),
            None => self.rx.recv().ok(),
        };
        let rest: Vec<Event> = self.rx.try_iter().collect();
        for event in [first.unwrap_or(Event::Gone)].into_iter().chain(rest) {
            self.keep(event);
        }
        true
    }

    fn keep(&mut self, event: Event) {
        match event {
            Event::Click(c) => self.clicks.push(c),
            Event::Note(line) => self.notes.push(line),
            Event::Reply(_) => {}
            Event::Gone => std::process::exit(0),
        }
    }
}

/// Replies to our commands are %begin … %end blocks flagged 1 (the attach itself is flagged 0);
/// every other line starting with % is a notification.
fn read(out: impl Read, events: Sender<Event>) {
    let (mut ours, mut theirs) = (None::<Vec<String>>, false);
    for line in BufReader::new(out).lines().map_while(Result::ok) {
        let end = line.starts_with("%end ") || line.starts_with("%error ");
        if end && ours.is_some() {
            let _ = events.send(Event::Reply(ours.take().unwrap_or_default()));
        } else if let Some(lines) = ours.as_mut() {
            lines.push(line);
        } else if theirs {
            theirs = !end;
        } else if line.starts_with("%begin ") {
            (ours, theirs) = if line.ends_with(" 1") { (Some(vec![]), false) } else { (None, true) };
        } else if line.starts_with("%exit") {
            break;
        } else if line.starts_with('%') && events.send(Event::Note(line)).is_err() {
            return;
        }
    }
    let _ = events.send(Event::Gone);
}
