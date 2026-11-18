//! One control-mode connection to tmux (`tmux -C`). tmux pushes what changed (tab switched, added,
//! renamed, closed, resized, and %paste-buffer-changed from `tabrail-tmux refresh`), and commands go
//! over the same pipe, so nothing is spawned while the sidebar runs.
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

/// A mouse press or release in the sidebar: which button, and the row (1-based).
pub struct Click {
    pub right: bool,
    pub release: bool,
    pub row: usize,
}

pub enum Event {
    Click(Click),
    Changed,
    Reply(Vec<String>),
    Gone,
}

pub struct Tmux {
    input: ChildStdin,
    pub rx: Receiver<Event>,
    pub tx: Sender<Event>,
    pub clicks: Vec<Click>,
}

impl Tmux {
    /// Attach to `session` without a say in window sizes and without pane output.
    pub fn attach(session: &str) -> std::io::Result<Tmux> {
        let mut child = Command::new("tmux")
            .args(["-C", "attach-session", "-f", "ignore-size,no-output", "-t", session])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let (tx, rx) = channel();
        let (out, events) = (child.stdout.take().expect("stdout is piped"), tx.clone());
        thread::spawn(move || read(out, events));
        Ok(Tmux { input: child.stdin.take().expect("stdin is piped"), rx, tx, clicks: Vec::new() })
    }

    /// Send one command (one per line: each gets its own reply) and wait for its reply; clicks that
    /// come in meanwhile wait in `clicks`.
    pub fn ask(&mut self, cmd: &str) -> Vec<String> {
        let _ = writeln!(self.input, "{cmd}").and_then(|_| self.input.flush());
        loop {
            match self.rx.recv() {
                Ok(Event::Reply(lines)) => return lines,
                Ok(Event::Click(c)) => self.clicks.push(c),
                Ok(Event::Changed) => {}
                Ok(Event::Gone) | Err(_) => std::process::exit(0),
            }
        }
    }

    /// Wait for the next change, a click, or `timeout`; changes that pile up count once. True if
    /// something happened.
    pub fn wait(&mut self, timeout: Duration) -> bool {
        let woke = match self.rx.recv_timeout(timeout) {
            Ok(Event::Click(c)) => {
                self.clicks.push(c);
                true
            }
            Ok(Event::Gone) | Err(RecvTimeoutError::Disconnected) => std::process::exit(0),
            Ok(_) => true,
            Err(RecvTimeoutError::Timeout) => false,
        };
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Click(c) => self.clicks.push(c),
                Event::Gone => std::process::exit(0),
                _ => {}
            }
        }
        woke
    }
}

/// Replies to our commands are %begin … %end blocks flagged 1 (the attach itself is flagged 0);
/// every other line starting with % is a notification.
fn read(out: impl Read, events: Sender<Event>) {
    let (mut ours, mut theirs) = (None::<Vec<String>>, false);
    for line in BufReader::new(out).lines().map_while(Result::ok) {
        let end = line.starts_with("%end ") || line.starts_with("%error ");
        if let Some(lines) = ours.as_mut() {
            if end {
                let _ = events.send(Event::Reply(ours.take().unwrap_or_default()));
            } else {
                lines.push(line);
            }
        } else if theirs {
            theirs = !end;
        } else if line.starts_with("%begin ") {
            (ours, theirs) = if line.ends_with(" 1") { (Some(vec![]), false) } else { (None, true) };
        } else if line.starts_with("%exit") {
            break;
        } else if line.starts_with('%') && events.send(Event::Changed).is_err() {
            return;
        }
    }
    let _ = events.send(Event::Gone);
}
