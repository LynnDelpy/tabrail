//! What the sidebar shows: the session's tabs (one `list-panes` reply), the project groups from
//! groups.conf, and what each tab is doing.
use crate::{name, program};
use std::collections::BTreeMap;

pub const FIELDS: [&str; 22] = [
    "window_id", "window_index", "window_name", "window_active", "@group", "@sidebar",
    "pane_active", "@claude", "@claude_since", "@claude_took", "@finished",
    "pane_command_running", "pane_command_start_time", "pane_command_end_time",
    "pane_command_status", "pane_id", "pane_title", "pane_current_command", "pane_current_path",
    "automatic-rename", "host_short", "@reader",
];

pub struct Tab {
    pub id: String,
    pub index: u32,
    pub name: String,
    pub program: String, // after the name: `· vim`, `✳` for claude, or nothing
    pub current: bool,
    pub group: String,
    f: Vec<String>,
}

/// One Tab per window from `list-panes -F` lines (FIELDS joined by tabs): its content pane, the
/// active one if there are several; sidebar and reader panes are skipped.
pub fn tabs(lines: &[String]) -> Vec<Tab> {
    let mut by_window: BTreeMap<u32, Tab> = BTreeMap::new();
    for line in lines {
        let f: Vec<String> = line.split('\t').map(str::to_string).collect();
        if f.len() < FIELDS.len() || f[5] == "1" || f[21] == "1" {
            continue;
        }
        let index = f[1].parse().unwrap_or(0);
        if by_window.contains_key(&index) && f[6] != "1" {
            continue;
        }
        let group = if f[4].is_empty() { "Default".into() } else { f[4].clone() };
        let name = name::label(&name::Facts {
            window: &f[2], renamed: f[19] == "0", claude: f[17] == "claude",
            title: &f[16], command: &f[17], running: f[11] == "1", path: &f[18], host: &f[20],
        });
        let program = program::mark(&name, &f[17]);
        let tab = Tab { id: f[0].clone(), index, name, program, current: f[3] == "1", group, f };
        by_window.insert(index, tab);
    }
    by_window.into_values().collect()
}

/// (name, tint, icon) per line of groups.conf, then Default.
pub fn groups(conf: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = conf
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split_whitespace().map(str::to_string).collect::<Vec<_>>())
        .filter(|c| c.len() >= 3)
        .map(|c| (c[0].clone(), c[1].clone(), c[2].clone()))
        .collect();
    out.push(("Default".into(), String::new(), "•".into()));
    out
}

fn ago(seconds: i64) -> String {
    let s = seconds.max(0);
    match s {
        0..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        _ => format!("{}h{}m", s / 3600, s % 3600 / 60),
    }
}

/// (text, kind) right of a tab; kind picks the colour: busy, wait, done, fail, dim.
pub fn status(tab: &Tab, now: i64) -> (String, &'static str) {
    let num = |i: usize| tab.f[i].parse::<i64>().unwrap_or(now);
    let icon = |s: &str| match s {
        "writing" => "✎", "running" => "⚙", "reading" => "◎", "searching" => "⌕",
        "delegating" => "⧉", _ => "…",
    };
    // Claude's state only while claude runs: a crashed Claude leaves its @claude behind
    let claude = if tab.f[17] == "claude" { tab.f[7].as_str() } else { "" };
    match claude {
        "waiting" => return ("● waits".into(), "wait"),
        "done" => return (format!("✓ {}", tab.f[9]), "done"),
        "idle" => return ("◇".into(), "dim"),
        "" => {}
        state => return (format!("{} {}", icon(state), ago(now - num(8))), "busy"),
    }
    if tab.f[11] == "1" && now - num(12) >= 3 {
        return (format!("⏱ {}", ago(now - num(12))), "busy");
    }
    if tab.f[10] == "1" {
        return ("✓".into(), "done");
    }
    if !tab.f[13].is_empty() && !matches!(tab.f[14].as_str(), "" | "0") {
        return (format!("✗ {}", tab.f[14]), "fail");
    }
    (String::new(), "dim")
}
