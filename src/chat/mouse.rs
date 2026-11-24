//! The mouse's SGR reports (`ESC [ <` button;column;row, `M` pressed, `m` released): the wheel, and a
//! left press with where it was (for the bar's buttons); releases, drags and other buttons dropped.
use crate::keys::Key;

/// After `ESC [`: the report up to its final byte.
pub fn sgr(report: &str) -> Option<Key> {
    let fields = report.strip_prefix('<')?.strip_suffix('M')?;
    let n: Vec<usize> = fields.split(';').map(|f| f.parse().ok()).collect::<Option<_>>()?;
    match n[..] {
        [64, ..] => Some(Key::WheelUp),
        [65, ..] => Some(Key::WheelDown),
        [0, col, row] if col > 0 && row > 0 => Some(Key::Click(row - 1, col - 1)),
        _ => None,
    }
}
