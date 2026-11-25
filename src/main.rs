//! tabrail, the tmux sidebar, running in the left pane of every tab (`tabrail-tmux ensure` starts it):
//! all tabs of the session grouped by project (groups.conf, or the file in tmux option @tabrail_groups) and what each is doing. Left button: go to a tab,
//! fold a group, drag a tab or group (drag.rs); right button: a menu (menu.rs). While its tab is shown
//! (shown.rs) it redraws when tmux reports a change over its control connection, and every second for
//! the timers and the clock; hidden, it draws nothing and asks tmux only what shown.rs needs.
mod control;
mod drag;
mod draw;
mod head;
mod input;
mod menu;
mod model;
mod name;
mod program;
mod shown;
mod ssh;
mod timer;
mod width;

use control::Tmux;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, thread};

fn main() {
    let pane = env::var("TMUX_PANE").expect("runs inside a tmux pane");
    let dir = env::current_exe().ok().and_then(|p| Some(p.parent()?.parent()?.parent()?.to_path_buf()))
        .expect("binary lives in tabrail/target/release");
    let [helper, mover, namer, timer] = ["act", "move", "name", "timer"].map(|f| format!("{}/tabrail-{f}", dir.display()));
    let ids = Command::new("tmux").args(["display", "-p", "-t", &pane, "#{session_id} #{pid} #{window_id}"])
        .output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let ids: Vec<&str> = ids.split(' ').chain(["0", ""]).collect();
    let (session, mut menus, mut view) = (ids[0], menu::Files::new(ids[1], &pane), shown::Shown::new(ids[0], ids[2]));
    let _ = Command::new("stty").args(["-icanon", "-echo"]).stdin(Stdio::inherit()).status();
    let mut t = Tmux::attach(&session).expect("tmux -C attach");
    let tx = t.tx.clone();
    thread::spawn(move || input::read_clicks(tx));
    t.ask(&format!("set -p -t {pane} @sidebar 1"));
    let place = menu::Place { pane: &pane, helper: &helper, mover: &mover, namer: &namer, timer: &timer };
    // the tabs' own panes only: tmux reads /proc (current command and path) for these alone
    let list = format!("list-panes -s -t {session} -f '#{{!:#{{@sidebar}}#{{@reader}}#{{@reader_slot}}}}' -F '{}'",
                       model::FIELDS.map(|f| format!("#{{{f}}}")).join("\t"));
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]2;sidebar\x1b\\\x1b[?25l\x1b[?1000h\x1b[?1006h");
    // the row, column and tab or group a left press started on; the lines on screen and their size
    let (mut drag, mut shown, mut drawn) = (None, Vec::new(), (0, 0));
    // while tmux shows a right-click menu over the sidebar, drawing would paint over it: paused from
    // the right press until the next change or click after the menu's first moment, 10 s at most
    let mut menu_open: Option<Instant> = None;
    let mut keep = width::Width::default(); // one width for every tab's sidebar
    loop {
        let wake = view.wait(&mut t);
        if wake != shown::Wake::Shown { // hidden meanwhile: its menu closed; a shrink may have cut lines
            (menu_open, drawn) = (None, (0, 0)); // so the next frame repaints them all
        }
        let head = t.ask(&format!("display -p -t {pane} '%H:%M  %a %d.%m.\t#{{@sidebar_fold}}\t\
                                   #{{pane_width}}\t#{{pane_height}}\t#{{@tabrail_groups}}\t{}'", width::FORMAT));
        let h: Vec<&str> = head.first().map(|l| l.split('\t').collect()).unwrap_or_default();
        let folded: HashSet<String> = h.get(1).unwrap_or(&"").split('|').map(str::to_string).collect();
        let size: (usize, usize) = (h.get(2).and_then(|v| v.parse().ok()).unwrap_or(26),
                                    h.get(3).and_then(|v| v.parse().ok()).unwrap_or(40));
        if keep.keep(&mut t, &pane, size.0, h.get(5..).unwrap_or_default()) || wake == shown::Wake::Layout {
            continue; // draw at the new width; hidden: no frame
        }
        let tabs = model::tabs(&t.ask(&list));
        let conf = h.get(4).filter(|p| !p.is_empty()).map(PathBuf::from).unwrap_or(dir.join("groups.conf"));
        let groups = model::groups(&fs::read_to_string(conf).unwrap_or_default());
        let timers = timer::Timers::read(); // every redraw: tabrail-timer changes it from any sidebar
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        let screen = draw::screen(&tabs, &groups, &folded, &timers, h.first().unwrap_or(&""), size, now);
        menus.write(size.1, &screen.rows, &tabs, &folded, &timers, &place);
        if menu_open.is_none() {
            if drawn != size {
                (shown, drawn) = (Vec::new(), size); // tmux reflowed the old lines: repaint them all
            }
            let _ = out.write_all(draw::changes(&mut shown, screen.lines).as_bytes()).and_then(|_| out.flush());
        }
        for click in std::mem::take(&mut t.clicks) {
            if click.right && !click.release {
                // tmux's right press moved the keyboard here: give it back; its menu opens on release
                t.ask(&format!("last-pane -t {pane}"));
                menu_open = Some(Instant::now());
            } else if !click.right {
                for cmd in drag::left(&click, screen.rows.get(click.row), &tabs, &folded, &place, &mut drag) {
                    t.ask(&cmd);
                }
            }
        }
        let woke = t.wait(Some(Duration::from_secs(1)));
        if let Some(since) = menu_open {
            let age = since.elapsed();
            if (woke && age > Duration::from_millis(500)) || age > Duration::from_secs(10) {
                menu_open = None;
            }
        }
    }
}
