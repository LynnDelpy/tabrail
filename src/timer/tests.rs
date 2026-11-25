//! timer.rs: the state file's lines and the label.
use super::Timers;

#[test]
fn running_stopped_and_none() {
    let t = Timers::parse("Website 5400 0\nInfra 120 1000\n");
    assert_eq!(t.running("Website"), Some(false));
    assert_eq!(t.running("Infra"), Some(true));
    assert_eq!(t.running("Notes"), None);
    assert_eq!(t.label("Website", 2000), Some(("⏸ 1h30m".into(), "dim")));
    assert_eq!(t.label("Infra", 1045), Some(("▶ 2m".into(), "busy"))); // 120 + 45 s
    assert_eq!(t.label("Infra", 1000), Some(("▶ 2m".into(), "busy")));
    assert_eq!(t.label("Notes", 2000), None);
}

#[test]
fn a_start_in_the_future_adds_nothing() {
    assert_eq!(Timers::parse("A 90 5000\n").label("A", 4000), Some(("▶ 1m".into(), "busy")));
}

#[test]
fn garbage_is_skipped() {
    let t = Timers::parse("garbage here\n\u{0}\u{fffd} junk\nA 1\nB -1 0\nC 1 2 3\nE +5 0\nF 1234567890123456 0\nD 10 0\nD 20 0\n");
    assert_eq!(t.0.len(), 1);
    assert_eq!(t.label("D", 0), Some(("⏸ 10s".into(), "dim"))); // the first line of a name
    assert_eq!(Timers::parse("").0.len(), 0);
}
