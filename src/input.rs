//! Mouse input from the sidebar's terminal: tmux passes clicks on as SGR sequences
//! (ESC [ < button ; column ; row M) once main.rs has switched mouse reporting on.
use crate::control::Event;
use std::io::Read;
use std::sync::mpsc::Sender;

/// A mouse press or release in the sidebar: which button, and the row (1-based).
pub struct Click {
    pub right: bool,
    pub release: bool,
    pub row: usize,
}

/// Presses and releases of the left (button 0) and right (button 2) button become Event::Click.
pub fn read_clicks(tx: Sender<Event>) {
    let (mut buf, mut chunk) = (Vec::new(), [0u8; 256]);
    while let Ok(n @ 1..) = std::io::stdin().read(&mut chunk) {
        buf.extend_from_slice(&chunk[..n]);
        while let Some(end) = buf.iter().position(|&b| b == b'M' || b == b'm') {
            let (seq, press) = (String::from_utf8_lossy(&buf[..end]).to_string(), buf[end] == b'M');
            buf.drain(..=end);
            let n: Vec<usize> = seq.rsplit("[<").next().unwrap_or("").split(';')
                .filter_map(|s| s.parse().ok()).collect();
            if n.len() == 3 && (n[0] == 0 || n[0] == 2) {
                let click = Click { right: n[0] == 2, release: !press, row: n[2] };
                if tx.send(Event::Click(click)).is_err() {
                    return;
                }
            }
        }
    }
}
