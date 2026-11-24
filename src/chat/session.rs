//! The claude behind the chat, across processes: started with the caller's options; after it ended
//! or a turn failed, the next question starts it again with `--resume <session>` (as it was started
//! under `--no-session-persistence`; on a new conversation when one ended before it named its session).
//! Events from a process before the running one are dropped.
use crate::claude::Claude;
use crate::stream::Out;
use crate::Event;
use std::sync::mpsc::Sender;
use std::thread;

pub struct Session {
    opts: Vec<String>, // the caller's claude options
    tx: Sender<Event>,
    claude: Option<Claude>,
    gen: u32,        // the running process's number
    id: String,      // the conversation's id
    named: bool,     // the running process reported its session id
    new: bool,       // without an id: start a new conversation, not the caller's -c or -r
    pub ended: bool, // claude ended or a turn failed: the next question starts it again
}

impl Session {
    pub fn new(opts: Vec<String>, tx: Sender<Event>) -> Session {
        Session { opts, tx, claude: None, gen: 0, id: String::new(), named: false, new: false, ended: true }
    }

    /// Err: why claude did not start.
    pub fn start(&mut self) -> Result<(), String> {
        if let Some(old) = self.claude.take() {
            thread::spawn(move || drop(old)); // it ends on its own thread: the keys and the screen go on
        }
        (self.gen, self.named) = (self.gen + 1, false);
        let opts = match (self.id.as_str(), self.new) {
            _ if self.opts.iter().any(|o| o == "--no-session-persistence") => self.opts.clone(),
            ("", false) => self.opts.clone(),
            ("", true) => options(&self.opts, None),
            (id, _) => options(&self.opts, Some(id)),
        };
        let started = Claude::start(&opts, self.gen, self.tx.clone());
        self.ended = started.is_err();
        self.claude = Some(started.map_err(|e| format!("claude did not start: {e}"))?);
        Ok(())
    }

    pub fn ask(&mut self, text: &str) -> Result<(), String> {
        if self.ended {
            self.start()?;
        }
        if let Some(claude) = &mut self.claude {
            claude.ask(text);
        }
        Ok(())
    }

    /// The running process's events; its session id is kept for --resume, and forgotten when it
    /// ended without naming one (its conversation could not be resumed).
    pub fn heard(&mut self, gen: u32, out: Out) -> Option<Out> {
        match (gen == self.gen, &out) {
            (false, _) => return None,
            (true, Out::Session(id)) => (self.id, self.named) = (id.clone(), true),
            (true, Out::Exit(_)) if !self.named => (self.id, self.new) = (String::new(), true),
            _ => {}
        }
        Some(out)
    }
}

/// The caller's options without their `--continue`, `-c`, `--resume`, `-r` and `--session-id` (with
/// its value: the next option, if it is no flag), and with `--resume <session>` when there is one.
fn options(opts: &[String], session: Option<&str>) -> Vec<String> {
    let (mut out, mut it) = (Vec::new(), opts.iter().peekable());
    while let Some(o) = it.next() {
        match o.as_str() {
            "--continue" | "-c" => {}
            "--resume" | "-r" | "--session-id" => drop(it.next_if(|v| !v.starts_with('-'))),
            o if o.starts_with("--resume=") || o.starts_with("--session-id=") => {}
            _ => out.push(o.clone()),
        }
    }
    out.extend(session.into_iter().flat_map(|s| ["--resume".into(), s.into()]));
    out
}
