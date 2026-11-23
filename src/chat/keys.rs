//! Keys from the terminal in raw mode: UTF-8 text, the editing keys, a bracketed paste (its lines
//! joined with spaces) and the mouse's wheel and left presses (mouse.rs); other keys are dropped.
use crate::Event;
use std::io::Read;
use std::sync::mpsc::Sender;

pub enum Key {
    Text(String),
    Enter,
    Back,
    Left,
    Right,
    Home, // Ctrl-A too
    End,  // Ctrl-E too
    Kill, // Ctrl-U: up to the cursor
    Clear, // Ctrl-C
    Eof,  // Ctrl-D
    PageUp,
    PageDown,
    WheelUp,
    WheelDown,
    Click(usize, usize), // a left press: its row and column, from 0
    Gone, // the terminal closed
}

pub fn read(tx: Sender<Event>) {
    let (mut buf, mut chunk) = (Vec::new(), [0u8; 4096]);
    loop {
        let n = match std::io::stdin().read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        buf.extend_from_slice(&chunk[..n]);
        while let Some((len, key)) = next(&buf) {
            buf.drain(..len);
            if key.is_some_and(|key| tx.send(Event::Key(key)).is_err()) {
                return;
            }
        }
    }
    let _ = tx.send(Event::Key(Key::Gone));
}

/// The first key in `b`: the bytes it takes and the key (None: bytes dropped); None while `b` ends
/// inside a key, which the next read completes.
fn next(b: &[u8]) -> Option<(usize, Option<Key>)> {
    let key = match *b.first()? {
        0x1b if b.starts_with(b"\x1b[200~") => {
            let end = b.windows(6).position(|w| w == b"\x1b[201~")?;
            return Some((end + 6, Some(Key::Text(joined(&b[6..end])))));
        }
        0x1b => match b.get(1)? {
            b'[' | b'O' => {
                let end = 2 + b[2..].iter().position(|c| (0x40..=0x7e).contains(c))?;
                return Some((end + 1, csi(&b[2..=end])));
            }
            _ => return Some((1, None)), // Esc: story 3's
        },
        b'\r' | b'\n' => Key::Enter,
        0x7f | 0x08 => Key::Back,
        0x01 => Key::Home,
        0x05 => Key::End,
        0x15 => Key::Kill,
        0x03 => Key::Clear,
        0x04 => Key::Eof,
        c if c < 0x20 => return Some((1, None)),
        c => {
            let len = match c { 0xc0..=0xdf => 2, 0xe0..=0xef => 3, 0xf0..=0xf7 => 4, _ => 1 };
            return Some(match std::str::from_utf8(b.get(..len)?) {
                Ok(text) => (len, Some(Key::Text(text.into()))),
                Err(_) => (1, None), // a bad byte: the keys after it stay
            });
        }
    };
    Some((1, Some(key)))
}

/// After `ESC [` or `ESC O`: parameters and the final byte.
fn csi(seq: &[u8]) -> Option<Key> {
    Some(match std::str::from_utf8(seq).ok()? {
        "D" => Key::Left,
        "C" => Key::Right,
        "H" | "1~" | "7~" => Key::Home,
        "F" | "4~" | "8~" => Key::End,
        "5~" => Key::PageUp,
        "6~" => Key::PageDown,
        s if s.starts_with('<') => return crate::mouse::sgr(s),
        _ => return None,
    })
}

fn joined(paste: &[u8]) -> String {
    let text: String = String::from_utf8_lossy(paste).chars()
        .map(|c| if c.is_control() && c != '\n' && c != '\r' { ' ' } else { c }).collect();
    text.split(['\r', '\n']).filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join(" ")
}
