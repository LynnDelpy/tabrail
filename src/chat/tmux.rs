//! tabrail-chat's pane, found by its terminal (the reader command unsets TMUX_PANE), and what the bar
//! asks of `tabrail-reader size`, on a thread of its own and in order: a button's step, or the bar's
//! own height (the newest one queued), asked only while the bar has the keyboard, unzoomed, and the
//! height would change.
use std::collections::VecDeque;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum Ask {
    Rows(usize), // its own height
    Up,
    Down,
}

/// Where the bar sends its asks; a message once this pane is found to be the reader (none in another
/// pane or outside tmux); the bar's height after each button's ask (None: unknown).
pub fn start() -> (Sender<Ask>, Receiver<()>, Receiver<Option<usize>>) {
    let (asks, rx) = mpsc::channel();
    let (tx, found) = mpsc::channel();
    let (done, heights) = mpsc::channel();
    thread::spawn(move || {
        let Some((pane, dir)) = find() else { return };
        let _ = tx.send(());
        let mut queue = VecDeque::new();
        while let Some(ask) = next(&rx, &mut queue) {
            let arg = match ask {
                Ask::Up => "up".to_string(),
                Ask::Down => "down".to_string(),
                Ask::Rows(rows) if takes(&pane, rows) => rows.to_string(),
                Ask::Rows(_) => continue,
            };
            let (mut reader, null) = (Command::new(format!("{dir}/tabrail-reader")), Stdio::null);
            let _ = reader.args(["size", &pane, &arg]).stdin(null()).stdout(null()).stderr(null()).status();
            if matches!(ask, Ask::Up | Ask::Down) {
                let height = tmux(&["display", "-p", "-t", &pane, "#{pane_height}"]);
                let _ = done.send(height.and_then(|h| h.trim().parse().ok()));
            }
        }
    });
    (asks, found, heights)
}

/// The next ask, after all queued: the buttons in order; of its own heights only the newest (an
/// older one is stale by then). None once the bar is gone.
fn next(rx: &Receiver<Ask>, queue: &mut VecDeque<Ask>) -> Option<Ask> {
    if queue.is_empty() {
        queue.push_back(rx.recv().ok()?);
    }
    queue.extend(rx.try_iter());
    let newest = queue.iter().rposition(|ask| matches!(ask, Ask::Rows(_)));
    let mut i = 0;
    queue.retain(|ask| {
        i += 1;
        !matches!(ask, Ask::Rows(_)) || Some(i - 1) == newest
    });
    queue.pop_front()
}

/// This pane and tabrail's folder, once the pane is the reader: tabrail-reader marks it right after
/// it started the reader command, so a few looks over 3 s.
fn find() -> Option<(String, String)> {
    let tty = std::fs::read_link("/proc/self/fd/0").ok()?.to_str()?.to_string();
    for wait in [0, 100, 200, 400, 800, 1600] {
        thread::sleep(Duration::from_millis(wait));
        let Some(panes) = tmux(&["list-panes", "-a", "-F", "#{pane_tty} #{pane_id} #{?@reader,1,0} #{@tabrail_dir}"])
        else { continue };
        let me = panes.lines().find_map(|line| line.strip_prefix(&tty)?.strip_prefix(' ')?.split_once(' '));
        if let Some((pane, dir)) = me.and_then(|(pane, rest)| Some((pane, rest.strip_prefix("1 ")?))) {
            return Some((pane.into(), dir.into()));
        }
    }
    None
}

/// The bar has the keyboard, its tab is not zoomed, and `rows` as tabrail-reader takes them (4 up to
/// a third of the window) are not its height already.
fn takes(pane: &str, rows: usize) -> bool {
    let format = "#{pane_active} #{window_zoomed_flag} #{window_height} #{pane_height}";
    let Some(out) = tmux(&["display", "-p", "-t", pane, format]) else { return false };
    let n: Vec<usize> = out.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    matches!(n[..], [1, 0, window, height] if rows.clamp(4, (window / 3).max(4)) != height)
}

fn tmux(args: &[&str]) -> Option<String> {
    let out = Command::new("tmux").args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}
