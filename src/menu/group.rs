//! menu.rs: a group's row. After its new tab and new group: new subgroup, rename, move into group and
//! colour (not Default's), fold, start or stop and reset its timer.
use super::{clean, Place};
use crate::timer::Timers;
use std::collections::HashSet;

/// Group `raw`'s menu: `items` (its new tab and new group), then its own; `namer` opens tabrail-name.
pub fn items(raw: &str, mut items: Vec<String>, namer: &dyn Fn(&str, &str, &str) -> String, folded: &HashSet<String>,
             timers: &Timers, p: &Place) -> Vec<String> {
    let (name, h) = (clean(raw), p.helper);
    if name != "Default" { // New subgroup…, Move into group… and Colour… as the timer below: a cleaned name is another group's line
        items.extend((name == *raw).then(|| format!("'New subgroup…' s {}", namer("sub", &name, ""))));
        items.push(format!("'Rename group' r {}", namer("group", &name, &name)));
        items.extend((name == *raw).then(|| format!("'Move into group…' i {{ run-shell -b \"{} '{name}' '#{{client_name}}'\" }}", p.nester)));
        items.extend((name == *raw).then(|| format!("'Colour…' c {{ run-shell -b \"{} '{name}' '#{{client_name}}'\" }}", p.colourer)));
    }
    let fold = if folded.contains(&name) { "Unfold" } else { "Fold" };
    items.push(format!("{fold} f {{ run-shell -b '{h} fold {name}' }}"));
    if name == *raw { // a cleaned name would key another group's timer
        let timer = |verb: &str| format!("{{ run-shell -b \"{} {verb} '{name}'\" }}", p.timer);
        let (label, verb) = if timers.running(&name) == Some(true) { ("Stop", "stop") } else { ("Start", "start") };
        items.extend(["''".into(), format!("'{label} timer' t {}", timer(verb))]);
        items.extend(timers.running(&name).map(|_| format!("'Reset timer' 0 {}", timer("reset"))));
    }
    items
}
