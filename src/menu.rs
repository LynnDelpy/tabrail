//! Right-click menus: one per sidebar row (a tab: new tab, rename, duplicate, move to group, new
//! group, close; a group: new tab, new group, rename, fold; empty space: new tab, new group), as tmux
//! display-menu commands at the mouse. A menu only takes clicks when tmux opens it from a mouse key
//! binding, so the sidebar keeps them as files ($XDG_RUNTIME_DIR/tabrail/SERVERPID-PANE-ROW.conf)
//! and tmux.conf's right-button binding runs the one under the mouse. Item commands are tmux { }
//! blocks; menus are rounded as set in tmux.conf.
use crate::draw::Row;
use crate::model::Tab;
use std::collections::HashSet;

pub struct Place<'a> {
    pub pane: &'a str,   // the sidebar's pane, for the session of a new tab
    pub helper: &'a str, // tabrail-act
    pub mover: &'a str,  // tabrail-move
    pub namer: &'a str,  // tabrail-name
}

/// Names go into quoted tmux strings: drop what could end the quote or the command.
pub fn clean(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '\'' | '"' | '{' | '}' | ';' | '\\' | '$' | '#')).collect()
}

/// The menu for a row of the sidebar.
pub fn for_row(row: Option<&Row>, tabs: &[Tab], folded: &HashSet<String>, p: &Place) -> String {
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
        Some(Row::Group(name)) => {
            let name = clean(name);
            let mut items = vec![new_tab(&name), new_group(p.pane)];
            if name != "Default" {
                items.push(format!("'Rename group' r {}", namer("group", &name, &name)));
            }
            let fold = if folded.contains(&name) { "Unfold" } else { "Fold" };
            items.push(format!("{fold} f {{ run-shell -b '{h} fold {name}' }}"));
            (name, items)
        }
        _ => ("Tabs".into(), vec![new_tab("Default"), new_group(p.pane)]),
    };
    format!("display-menu -x M -y M -T '#[align=centre] {title} ' {}\n", items.join(" "))
}
