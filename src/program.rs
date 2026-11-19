//! What runs in a tab, shown dimmed after its name: `deploy · vim`, `✳` where claude runs. Nothing for
//! an idle shell, or when the name's first word already is the program (`htop`, `make test`,
//! `/usr/bin/htop`, `sudo vim x` while sudo runs), Claude included (`claude --continue`).
use crate::name::shell;

/// The mark for a tab named `name` whose foreground program is `command` (pane_current_command).
pub fn mark(name: &str, command: &str) -> String {
    // tmux takes the basename of an absolute argv0 only: `./target/release/x` stays whole
    let command = command.rsplit('/').next().unwrap_or(command);
    let first = name.split_whitespace().next().unwrap_or("");
    if command.is_empty() || shell(command) || first.rsplit('/').next() == Some(command) {
        String::new()
    } else if command == "claude" {
        "✳".into()
    } else {
        format!("· {command}")
    }
}
