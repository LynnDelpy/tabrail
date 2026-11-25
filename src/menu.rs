//! Right-click menus: one per sidebar row (a tab: new tab, rename, duplicate, move to group, new
//! group, close; a group: new tab, new group, rename, fold, start or stop and reset its timer; empty
//! space: new tab, new group), as tmux display-menu commands at the mouse. A menu only takes clicks
//! when tmux opens it from a mouse key binding, so the sidebar keeps them as files
//! ($XDG_RUNTIME_DIR/tabrail/SERVERPID-PANE-ROW.conf) and tmux.conf's right-button binding runs the
//! one under the mouse. Item commands are tmux { } blocks; menus are rounded as set in tmux.conf.
use crate::draw::Row;
use crate::model::Tab;
use crate::timer::Timers;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{env, fs};

pub struct Place<'a> {
    pub pane: &'a str,   // the sidebar's pane, for the session of a new tab
    pub helper: &'a str, // tabrail-act
    pub mover: &'a str,  // tabrail-move
    pub namer: &'a str,  // tabrail-name
    pub timer: &'a str,  // tabrail-timer
}

/// One sidebar's menu files and what each holds.
pub struct Files {
    prefix: PathBuf, // the tmux server's pid in the name, as pane ids repeat across servers
    written: Vec<String>,
}

impl Files {
    pub fn new(server: &str, pane: &str) -> Files {
        let dir = PathBuf::from(env::var("XDG_RUNTIME_DIR").unwrap_or("/tmp".into())).join("tabrail");
        let _ = fs::create_dir_all(&dir);
        Files { prefix: dir.join(format!("{server}-{pane}")), written: Vec::new() }
    }

    /// Write the files whose menu changed, one per terminal row (`mouse_y`; `rows` counts from 1).
    pub fn write(&mut self, height: usize, rows: &[Row], tabs: &[Tab], folded: &HashSet<String>, timers: &Timers,
                 p: &Place) {
        self.written.resize(height, String::new());
        for (y, file) in self.written.iter_mut().enumerate() {
            let text = for_row(rows.get(y + 1), tabs, folded, timers, p);
            if *file != text && fs::write(format!("{}-{y}.conf", self.prefix.display()), &text).is_ok() {
                *file = text;
            }
        }
    }
}

/// Names go into quoted tmux strings: drop what could end the quote or the command.
pub fn clean(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '\'' | '"' | '{' | '}' | ';' | '\\' | '$' | '#')).collect()
}

/// The menu for a row of the sidebar.
pub fn for_row(row: Option<&Row>, tabs: &[Tab], folded: &HashSet<String>, timers: &Timers, p: &Place) -> String {
    let h = p.helper;
    let new_tab = |group: &str| format!("'New tab' n {{ run-shell -b '{h} new {} {}' }}", clean(group), p.pane);
    // names are typed in tabrail-name's popup: KIND TARGET CLIENT and the name it starts with
    let namer = |kind: &str, target: &str, text: &str| format!("{{ run-shell -b \"{} {kind} '{target}' \
                                                                '#{{client_name}}' '{text}'\" }}", p.namer);
    // a new group: a tab (@id) moves into it, from elsewhere a new tab opens in it (the sidebar's pane,
    // %12 written p12 as tabrail-act new-group takes it)
    let new_group = |target: &str| format!("'New group…' g {}", namer("new", &target.replace('%', "p"), ""));
    let (title, items) = match row {
        Some(Row::Tab(id)) => {
            let tab = tabs.iter().find(|t| &t.id == id);
            let name = clean(tab.map(|t| t.name.as_str()).unwrap_or(""));
            (name.clone(), vec![
                new_tab(tab.map(|t| t.group.as_str()).unwrap_or("Default")),
                "''".into(),
                format!("Rename r {}", namer("tab", id, &name)),
                format!("Duplicate d {{ run-shell -b '{h} dup {id}' }}"),
                format!("'Move to group…' m {{ run-shell -b \"{} {id} '#{{client_name}}'\" }}", p.mover),
                new_group(id),
                "''".into(),
                // tabrail-tmux close: the reader pane moves on before the tab closes
                format!("Close x {{ confirm-before -p 'Close {name}? (y/n)' \
                         {{ run-shell -b '#{{@tabrail_dir}}/tabrail-tmux close {id}' }} }}"),
            ])
        }
        Some(Row::Group(raw)) => {
            let name = clean(raw);
            let mut items = vec![new_tab(&name), new_group(p.pane)];
            if name != "Default" {
                items.push(format!("'Rename group' r {}", namer("group", &name, &name)));
            }
            let fold = if folded.contains(&name) { "Unfold" } else { "Fold" };
            items.push(format!("{fold} f {{ run-shell -b '{h} fold {name}' }}"));
            if name == *raw { // a cleaned name would key another group's timer
                let timer = |verb: &str| format!("{{ run-shell -b \"{} {verb} '{name}'\" }}", p.timer);
                let (label, verb) = if timers.running(&name) == Some(true) { ("Stop", "stop") } else { ("Start", "start") };
                items.extend(["''".into(), format!("'{label} timer' t {}", timer(verb))]);
                items.extend(timers.running(&name).map(|_| format!("'Reset timer' 0 {}", timer("reset"))));
            }
            (name, items)
        }
        _ => ("Tabs".into(), vec![new_tab("Default"), new_group(p.pane)]),
    };
    format!("display-menu -x M -y M -T '#[align=centre] {title} ' {}\n", items.join(" "))
}
