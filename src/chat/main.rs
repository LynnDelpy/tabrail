//! tabrail-chat [claude options…]: a Claude Code frontend for a pane of a few rows (the reader bar).
//! It runs `claude -p` over stream-json in the caller's folder and environment with their options,
//! and draws the tail of the conversation above the input at the pane's full width.
mod bar;
mod chat;
mod claude;
mod input;
mod json;
mod keys;
mod mouse;
mod session;
mod stream;
mod term;
mod tmux;
mod view;

use std::sync::mpsc;
use std::time::Duration;
use std::{env, iter, thread};

pub enum Event {
    Key(keys::Key),
    Claude(u32, stream::Out), // from the claude process of that number
}

fn main() {
    let (tx, rx) = mpsc::channel();
    term::enter();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        term::leave();
        hook(info);
        std::process::exit(101);
    }));
    let keys = tx.clone();
    thread::spawn(move || keys::read(keys));
    let mut chat = chat::Chat::new(env::args().skip(1).collect(), tx);
    // the rows on screen and its size, where the cursor was put
    let (mut shown, mut drawn, mut cursor) = (Vec::new(), (0, 0), (0, 0));
    loop {
        let size = term::size();
        let (rows, at) = chat.frame(size);
        if drawn != size {
            (shown, drawn) = (Vec::new(), size); // tmux reflowed the old rows: repaint them all
        }
        let text = changes(&mut shown, rows);
        if !text.is_empty() || at != cursor {
            cursor = at;
            term::write(&format!("\x1b[?25l{text}\x1b[{};{}H\x1b[?25h", at.0 + 1, at.1 + 1));
        }
        let Ok(first) = rx.recv_timeout(Duration::from_millis(200)) else { continue };
        for event in iter::once(first).chain(rx.try_iter()) {
            let go_on = match event {
                Event::Key(key) => chat.key(key),
                Event::Claude(gen, out) => {
                    chat.claude(gen, out);
                    true
                }
            };
            if !go_on {
                term::leave();
                drop(chat); // ends claude
                return;
            }
        }
    }
}

/// What to write to turn `shown` into `rows`: only the rows that changed (as the sidebar's
/// draw::changes), each cleared first: with autowrap off, an erase after a full row would take
/// its last character.
fn changes(shown: &mut Vec<String>, rows: Vec<String>) -> String {
    let mut text = String::new();
    if shown.len() != rows.len() {
        shown.clear();
        text.push_str("\x1b[2J");
    }
    for (i, row) in rows.iter().enumerate() {
        if shown.get(i) != Some(row) {
            text += &format!("\x1b[{};1H\x1b[2K{row}", i + 1);
        }
    }
    *shown = rows;
    text
}
