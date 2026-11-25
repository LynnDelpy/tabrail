//! shown.rs: the I/O matrix rows that are parsing alone.
use super::Shown;

fn tab(window: &str, clients: &[&str]) -> Shown {
    let mut s = Shown::new("$1", window);
    s.clients(&clients.iter().map(|c| c.to_string()).collect::<Vec<_>>());
    s
}

#[test]
fn switch_hooks_new_tab_two_sessions() {
    // shown, to be shown, a new tab not selected; none of these lines asks tmux
    let (mut a, mut b, mut c) = (tab("@4", &["@4"]), tab("@7", &["@4"]), tab("@9", &["@4"]));
    assert!(a.is() && !b.is() && !c.is());
    for line in ["%session-window-changed $1 @7", "%paste-buffer-changed x", "%session-window-changed $2 @4"] {
        assert!(!a.note(line) && !b.note(line) && !c.note(line));
    }
    assert!(!a.is() && b.is() && !c.is());
}

#[test]
fn detach_and_attach() {
    let mut a = tab("@4", &["@4"]);
    assert!(a.note("%client-detached /dev/pts/3"));
    a.clients(&[]);
    a.note("%session-window-changed $1 @4"); // current, but nobody sees it
    assert!(!a.is() && a.note("%client-session-changed /dev/pts/3 $1 main"));
    a.clients(&["@4".into(), "@4".into()]); // two terminals
    assert!(a.is());
    assert!(!a.note("%client-session-changed client-4242 $1 main") && !a.note("%client-detached client-4242"));
}

#[test]
fn hidden_wakes_for_its_own_layout_only() {
    let mut a = tab("@4", &["@7"]);
    assert!(!a.note("%layout-change @7 b25d,80x24,0,0,3 b25d,80x24,0,0,3 *") && !a.layout);
    assert!(!a.note("%layout-change @4 b25d,80x24,0,0,1 b25d,80x24,0,0,1 *") && a.layout && !a.is());
}
