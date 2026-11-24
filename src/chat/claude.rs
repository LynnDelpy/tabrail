//! One claude process: `${TABRAIL_CHAT_CLAUDE:-claude} -p` over stream-json in the caller's folder
//! and environment with the given options. A thread reads its lines into events for the main loop,
//! tagged with the process's number (session.rs drops those of a process replaced).
use crate::json;
use crate::stream::{self, Out};
use crate::Event;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{mpsc, mpsc::Sender, Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

const ARGS: &[&str] = &["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose",
                        "--include-partial-messages", "--permission-prompts", "none"];

extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

pub struct Claude {
    child: Child,
    stdin: ChildStdin,
}

impl Claude {
    pub fn start(opts: &[String], gen: u32, tx: Sender<Event>) -> std::io::Result<Claude> {
        let program = env::var("TABRAIL_CHAT_CLAUDE").ok().filter(|p| !p.is_empty()).unwrap_or("claude".into());
        let mut child = Command::new(program).args(ARGS).args(opts)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let (out, err) = (child.stdout.take().expect("piped"), child.stderr.take().expect("piped"));
        let tail = Arc::new(Mutex::new(String::new())); // the last line on stderr
        let (closed, wait) = mpsc::channel::<()>();
        let last = tail.clone();
        thread::spawn(move || {
            for line in BufReader::new(err).split(b'\n').map_while(Result::ok) {
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if let (false, Ok(mut last)) = (line.is_empty(), last.lock()) {
                    *last = line;
                }
            }
            drop(closed); // stderr closed: its last line is in
        });
        thread::spawn(move || {
            read(out, gen, &tx);
            // the last lines on stderr, unless a child of claude keeps it open
            let _ = wait.recv_timeout(Duration::from_secs(1));
            let tail = tail.lock().map(|t| t.clone()).unwrap_or_default();
            let _ = tx.send(Event::Claude(gen, Out::Exit(tail)));
        });
        let mut claude = Claude { stdin: child.stdin.take().expect("piped"), child };
        claude.send(r#"{"type":"control_request","request_id":"init","request":{"subtype":"initialize"}}"#);
        Ok(claude)
    }

    pub fn ask(&mut self, text: &str) {
        let content = json::quote(text);
        self.send(&format!(r#"{{"type":"user","message":{{"role":"user","content":{content}}}}}"#));
    }

    fn send(&mut self, line: &str) {
        let _ = self.stdin.write_all(format!("{line}\n").as_bytes());
    }
}

impl Drop for Claude {
    /// SIGTERM first, so that claude can end what it started; SIGKILL after a second.
    fn drop(&mut self) {
        unsafe { kill(self.child.id() as i32, 15) };
        for _ in 0..20 {
            if !matches!(self.child.try_wait(), Ok(None)) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read(out: impl Read, gen: u32, tx: &Sender<Event>) {
    let mut session = String::new();
    for line in BufReader::new(out).split(b'\n').map_while(Result::ok) {
        let Some(v) = json::parse(&String::from_utf8_lossy(&line)) else { continue }; // a bad line: skipped
        for out in stream::outs(&v, &mut session) {
            if tx.send(Event::Claude(gen, out)).is_err() {
                return;
            }
        }
    }
}
