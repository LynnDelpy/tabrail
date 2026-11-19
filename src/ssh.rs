//! The name of a tab while ssh runs in it, from the pane title alone. The remote shell's prompt title
//! (`lde@app01: ~/src/infra`) gives `lde@app01 infra`; vim's or nvim's own title (`nginx.conf
//! (/etc/nginx) - VIM`) gives `vim nginx.conf`; until the remote sends a title, the ssh command line
//! oh-my-zsh put there gives the host (`ssh -p 2222 lde@app01.example.ch`: `app01`); the local
//! shell's own title left over gives `ssh`; anything else the remote titles stays, without `VAR=value`
//! words in front and without Claude's spinner.
use crate::name::bare;

/// ssh's options that take a value, attached (`-p2222`) or as the next word.
const VALUED: &str = "BbcDEeFIiJLlmOoPpQRSWw";

pub fn label(title: &str, local_host: &str) -> String {
    // Claude's spinner (braille) or ✳ in front of its topic; any other first character stays ([No Name])
    let claude = |c: char| c.is_whitespace() || c == '✳' || ('\u{2800}'..='\u{28FF}').contains(&c);
    let t = title.trim_start_matches(claude).trim_end();
    let shell = prompt(t);
    if let Some((user, host, path)) = &shell {
        if host != local_host {
            return format!("{user}@{host} {}", folder(path)).trim_end().to_string();
        }
    }
    if let Some(name) = editor(t) {
        return name;
    }
    let line = bare(t);
    let mut words = line.split_whitespace();
    if words.next().is_some_and(|w| w.rsplit('/').next() == Some("ssh")) {
        return destination(words).map(host).filter(|h| !h.is_empty()).unwrap_or("ssh".into());
    }
    let local = shell.is_some() || line.is_empty() || line == local_host
        || line.starts_with(&format!("{local_host}.")) || line == "Claude Code";
    if local { "ssh".into() } else { line.to_string() }
}

/// A shell's prompt title `user@host:path` or `user@host: path`: user, short host, path.
fn prompt(t: &str) -> Option<(&str, String, &str)> {
    let (who, path) = t.split_once(':')?;
    let (user, at) = who.split_once('@')?;
    let whole = !user.is_empty() && !at.is_empty() && !who.contains(char::is_whitespace);
    whole.then(|| (user, host(at), path.trim()))
}

/// A path's last component; `~` for home, `/` for the root.
fn folder(path: &str) -> &str {
    let p = path.trim_end_matches('/');
    if p.is_empty() && path.starts_with('/') { "/" } else { p.rsplit('/').next().unwrap_or(p) }
}

/// vim's and nvim's title `FILE[ FLAG][ (DIR)][ (N of M)] - APP` (`a.conf + (/etc) (2 of 3) - VIM`,
/// `x.rs (~/p) - Nvim`, gvim's `- VIM1`) gives `vim a.conf`: the app in lower case without digits,
/// then the file's name without vim's modified/read-only flag word (`+`, `-`, `=`).
fn editor(t: &str) -> Option<String> {
    let (head, app) = t.rsplit_once(" - ")?;
    let app = app.trim_end_matches(|c: char| c.is_ascii_digit()).to_lowercase();
    if !matches!(app.as_str(), "vim" | "gvim" | "nvim") {
        return None;
    }
    let file = head.split_once(" (").map_or(head, |(file, _)| file);
    let file = match file.rsplit_once(' ') {
        Some((name, flag)) if flag.chars().all(|c| "+-=".contains(c)) => name,
        _ => file,
    };
    Some(format!("{app} {}", file.rsplit('/').next().unwrap_or(file)))
}

/// The first of ssh's arguments that is neither an option nor an option's value.
fn destination<'a>(mut args: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    while let Some(arg) = args.next() {
        let Some(flags) = arg.strip_prefix('-').filter(|f| !f.is_empty()) else {
            return Some(arg);
        };
        // flags bundle (-tt, -vp 2222): the first one that takes a value takes the rest of the word,
        // or the next word when it ends the word
        if flags.find(|c| VALUED.contains(c)).is_some_and(|i| i + 1 == flags.len()) {
            args.next();
        }
    }
    None
}

/// The host of `ssh://lde@app01.example.ch:2222`: no user, scheme, port or domain (`app01`); an IP
/// address stays whole.
fn host(dest: &str) -> String {
    let d = dest.strip_prefix("ssh://").unwrap_or(dest);
    let d = d.rsplit_once('@').map_or(d, |(_, h)| h);
    let d = match d.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(v6), // [::1]:2222
        None if d.matches(':').count() > 1 => d,         // ::1
        None => d.split(':').next().unwrap_or(d),
    };
    let ip = d.contains(':') || d.chars().all(|c| c.is_ascii_digit() || c == '.');
    if ip { d.to_string() } else { d.split('.').next().unwrap_or(d).to_string() }
}
