//! The name a tab gets in the sidebar. A name given by hand (rename) wins; while ssh runs, what ssh.rs
//! makes of the title; a Claude tab shows Claude's topic (its terminal title without the spinner or ✳);
//! while a command runs, its command line (the title oh-my-zsh sets) without `VAR=value` words in front;
//! an idle shell shows its folder and git branch (main and master left out).
use crate::ssh;
use std::fs;
use std::path::Path;

pub struct Facts<'a> {
    pub window: &'a str,  // window_name
    pub renamed: bool,    // automatic-rename off: named by hand
    pub claude: bool,     // the claude process runs (pane_current_command)
    pub title: &'a str,   // pane_title
    pub command: &'a str, // pane_current_command
    pub running: bool,    // OSC 133 says a command runs (tmux.zsh)
    pub path: &'a str,
    pub host: &'a str,    // host_short: tmux's title when nobody set one
}

pub fn label(f: &Facts) -> String {
    if f.renamed {
        return f.window.to_string();
    }
    if f.command == "ssh" {
        return ssh::label(f.title, f.host);
    }
    // no title worth showing: none, tmux's default (the host), the shell's idle one ("lde@f4l002:
    // ~/Dev") left over, or Claude's own name
    let title = f.title.trim_start_matches(|c: char| !c.is_alphanumeric()).trim();
    let told = !(title.is_empty() || title == f.host || title.starts_with(&format!("{}.", f.host))
        || (title.contains('@') && title.contains(':')) || title == "Claude Code");
    if f.claude {
        if told {
            return title.to_string();
        }
    } else if f.running || !shell(f.command) {
        return if told { bare(title).to_string() } else { f.command.to_string() };
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let folder = if f.path == home { "~".to_string() } else {
        Path::new(f.path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
    };
    match branch(Path::new(f.path)) {
        Some(b) if b != "main" && b != "master" => format!("{folder}·{b}"),
        _ => folder,
    }
}

/// A shell: in the foreground of a tab it runs no program.
pub fn shell(command: &str) -> bool {
    matches!(command, "zsh" | "bash" | "sh" | "fish")
}

/// `line` without the `NAME=value` words in front: `TERM=xterm-256color htop` gives `htop` (an alias
/// like ssh='TERM=xterm-256color ssh' reaches the title expanded); `line` itself if nothing else is left.
pub fn bare(line: &str) -> &str {
    let mut rest = line.trim_start();
    while assignment(rest) {
        let mut quote = None; // the value ends at the first whitespace outside '…' or "…"
        let end = rest.char_indices().find(|&(_, c)| {
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '\'' || c == '"' => quote = Some(c),
                None => return c.is_whitespace(),
            }
            false
        });
        rest = end.map_or("", |(i, _)| rest[i..].trim_start());
    }
    if rest.is_empty() { line } else { rest }
}

/// `word` starts with `NAME=`, NAME a shell variable's name.
fn assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// The checked-out branch (its last part: lde/add-person gives add-person) of the repository `dir`
/// is in, read from .git/HEAD; a worktree's .git file points to its own HEAD.
fn branch(dir: &Path) -> Option<String> {
    let git = dir.ancestors().map(|d| d.join(".git")).find(|g| g.exists())?;
    let head = if git.is_dir() {
        fs::read_to_string(git.join("HEAD")).ok()?
    } else {
        let gitdir = fs::read_to_string(&git).ok()?;
        fs::read_to_string(Path::new(gitdir.trim().strip_prefix("gitdir: ")?).join("HEAD")).ok()?
    };
    let head = head.trim();
    match head.strip_prefix("ref: refs/heads/") {
        Some(name) => name.rsplit('/').next().map(str::to_string),
        None => head.get(..7).map(str::to_string), // detached: short hash
    }
}
