//! The terminal: raw keys (stty, as the sidebar does), the alternate screen, mouse reports (for the
//! wheel) and bracketed paste on, autowrap off (a wide character cannot push a row onto the next);
//! its size from the kernel; all of it back on the way out.
use std::ffi::c_ulong;
use std::io::Write;
use std::process::{Command, Stdio};

#[repr(C)]
struct Winsize {
    rows: u16,
    cols: u16,
    xpixel: u16,
    ypixel: u16,
}

extern "C" {
    fn ioctl(fd: i32, request: c_ulong, ...) -> i32;
}

const TIOCGWINSZ: c_ulong = 0x5413; // Linux

pub fn enter() {
    let _ = Command::new("stty").args(["raw", "-echo"]).stdin(Stdio::inherit()).status();
    write("\x1b[?1049h\x1b[?7l\x1b[?1000h\x1b[?1006h\x1b[?2004h\x1b[2J");
}

pub fn leave() {
    write("\x1b[?2004l\x1b[?1006l\x1b[?1000l\x1b[?7h\x1b[0m\x1b[?25h\x1b[?1049l");
    let _ = Command::new("stty").arg("sane").stdin(Stdio::inherit()).status();
}

pub fn write(text: &str) {
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes()).and_then(|_| out.flush());
}

/// Columns and rows, asked each time: a resize needs no signal handler.
pub fn size() -> (usize, usize) {
    let mut ws = Winsize { rows: 0, cols: 0, xpixel: 0, ypixel: 0 };
    let ok = unsafe { ioctl(1, TIOCGWINSZ, &mut ws as *mut Winsize) } == 0;
    if ok && ws.cols > 0 && ws.rows > 0 { (ws.cols.into(), ws.rows.into()) } else { (80, 4) }
}
