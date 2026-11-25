//! Group timers, a stopwatch per group name that tabrail-timer starts, stops, resets and renames:
//! $XDG_STATE_HOME/tabrail/timers, else ~/.local/state/tabrail/timers, one line per timed group,
//! NAME ACCUMULATED SINCE in seconds (SINCE the epoch it was started, 0 while stopped). Read on every
//! redraw; wall-clock, so no sidebar counts. A missing file or a line that does not parse: no timer.
use crate::model;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

/// Group name -> (seconds accumulated, start epoch or 0 while stopped).
pub struct Timers(HashMap<String, (i64, i64)>);

impl Timers {
    pub fn read() -> Timers {
        let home = || PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".local/state");
        let dir = env::var_os("XDG_STATE_HOME").filter(|d| !d.is_empty()).map(PathBuf::from).unwrap_or_else(home);
        Timers::parse(&String::from_utf8_lossy(&fs::read(dir.join("tabrail/timers")).unwrap_or_default()))
    }

    /// As tabrail-timer reads it: three fields, both numbers digits only; the first line of a name.
    fn parse(text: &str) -> Timers {
        let num = |s: &str| if s.len() < 16 && s.bytes().all(|b| b.is_ascii_digit()) { s.parse().ok() } else { None };
        let mut timers = HashMap::new();
        for line in text.lines() {
            if let [name, acc, since] = line.split_whitespace().collect::<Vec<_>>()[..] {
                if let (Some(acc), Some(since)) = (num(acc), num(since)) {
                    timers.entry(name.to_string()).or_insert((acc, since));
                }
            }
        }
        Timers(timers)
    }

    /// None: the group has no timer; else whether it runs.
    pub fn running(&self, name: &str) -> Option<bool> {
        self.0.get(name).map(|&(_, since)| since > 0)
    }

    /// `▶ 12m` and busy while it runs, `⏸ 12m` and dim stopped; None without a timer.
    pub fn label(&self, name: &str, now: i64) -> Option<(String, &'static str)> {
        let &(acc, since) = self.0.get(name)?;
        Some(if since > 0 {
            (format!("▶ {}", model::ago(acc + (now - since).max(0))), "busy")
        } else {
            (format!("⏸ {}", model::ago(acc)), "dim")
        })
    }
}

#[cfg(test)]
mod tests;
