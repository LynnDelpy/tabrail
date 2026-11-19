//! tabrail, the tmux sidebar, running in the left pane of every tab (`tabrail-tmux ensure` starts it):
//! all tabs of the session grouped by project (groups.conf, or the file in tmux option @tabrail_groups) and what each is doing. Left button: go to a tab,
//! fold a group, drag a tab (drag.rs); right button: a menu (menu.rs). Redraws when tmux reports a
//! change over its control connection, and every second for the timers and the clock.
mod control;
mod drag;
mod draw;
mod input;
mod menu;
mod model;
mod name;
mod program;
mod ssh;
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
    let helper = dir.join("tabrail-act").display().to_string();
    let mover = dir.join("tabrail-move").display().to_string();
    let namer = dir.join("tabrail-name").display().to_string();
    let ids = Command::new("tmux").args(["display", "-p", "-t", &pane, "#{session_id} #{pid}"])
        .output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let (session, server) = ids.split_once(' ').unwrap_or((&ids, "0"));
    // menu files: the tmux server's pid in the name, as pane ids repeat across servers
    let menus = PathBuf::from(env::var("XDG_RUNTIME_DIR").unwrap_or("/tmp".into())).join("tabrail");
    let _ = fs::create_dir_all(&menus);
    let _ = Command::new("stty").args(["-icanon", "-echo"]).stdin(Stdio::inherit()).status();
    let mut t = Tmux::attach(&session).expect("tmux -C attach");
    let tx = t.tx.clone();
    thread::spawn(move || input::read_clicks(tx));
    t.ask(&format!("set -p -t {pane} @sidebar 1"));
    let place = menu::Place { pane: &pane, helper: &helper, mover: &mover, namer: &namer };
    let fields = model::FIELDS.map(|f| format!("#{{{f}}}")).join("\t");
    let mut out = std::io::stdout();
    let _ = write!(out, "\x1b]2;sidebar\x1b\\\x1b[?25l\x1b[?1000h\x1b[?1006h");
    // the tab a left press started on; the lines on screen and their size; the menu file for each row
    let (mut drag, mut shown, mut drawn, mut written) = (None, Vec::new(), (0, 0), Vec::<String>::new());
    // while tmux shows a right-click menu over the sidebar, drawing would paint over it: paused from
    // the right press until the next change or click after the menu's first moment, 10 s at most
    let mut menu_open: Option<Instant> = None;
    let mut keep = width::Width::default(); // one width for every tab's sidebar
    loop {
        let head = t.ask(&format!("display -p -t {pane} '%H:%M  %a %d.%m.\t#{{@sidebar_fold}}\t\
                                   #{{pane_width}}\t#{{pane_height}}\t#{{@tabrail_groups}}\t{}'", width::FORMAT));
        let h: Vec<&str> = head.first().map(|l| l.split('\t').collect()).unwrap_or_default();
        let folded: HashSet<String> = h.get(1).unwrap_or(&"").split('|').map(str::to_string).collect();
        let size: (usize, usize) = (h.get(2).and_then(|v| v.parse().ok()).unwrap_or(26),
                                    h.get(3).and_then(|v| v.parse().ok()).unwrap_or(40));
        if keep.keep(&mut t, &pane, size.0, h.get(5..).unwrap_or_default()) {
            continue; // draw at the new width
        }
        let tabs = model::tabs(&t.ask(&format!("list-panes -s -t {session} -F '{fields}'")));
        let conf = h.get(4).filter(|p| !p.is_empty()).map(PathBuf::from).unwrap_or(dir.join("groups.conf"));
        let groups = model::groups(&fs::read_to_string(conf).unwrap_or_default());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        let screen = draw::screen(&tabs, &groups, &folded, h.first().unwrap_or(&""), size, now);
        written.resize(size.1, String::new());
        for (y, file) in written.iter_mut().enumerate() {
            let text = menu::for_row(screen.rows.get(y + 1), &tabs, &folded, &place);
            if *file != text && fs::write(menus.join(format!("{server}-{pane}-{y}.conf")), &text).is_ok() {
                *file = text;
            }
        }
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
                for cmd in drag::left(&click, screen.rows.get(click.row), &tabs, &folded, &mut drag) {
                    t.ask(&cmd);
                }
            }
        }
        let woke = t.wait(Duration::from_secs(1));
        if let Some(since) = menu_open {
            let age = since.elapsed();
            if (woke && age > Duration::from_millis(500)) || age > Duration::from_secs(10) {
                menu_open = None;
            }
        }
    }
}
