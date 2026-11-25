---
title: 'Hidden sidebars stay idle'
type: 'refactor'
created: '2026-11-24'
status: 'done'
baseline_commit: '64a48fdc80574b2b58d085b36976c73d211bee1d'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-performance/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-performance/cost-today.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every sidebar, shown or not, runs `display` and a `list-panes` of the whole session every second and on every tmux notification; a tab switch or a Claude hook call wakes all 53 at once and the new tab's sidebar waits behind them (SPEC CAP-1..3, cost-today.md).

**Approach:** A sidebar tracks whether its tab is shown from its control connection's notifications. While hidden it sends tmux no commands and has no 1 s tick; when it becomes shown it draws at once. The shown sidebar asks `list-panes` only for tabs' own panes, so tmux reads `/proc` for those alone.

## Boundaries & Constraints

**Always:**
- Shown: its window is its session's current window and a non-control client is attached to the session (control clients, the sidebars, count in `session_attached`).
- Shown sidebars behave exactly as today: redraw on every notification and every second, menu files, clicks, drag, `width::keep`, right-click pause.
- Becoming shown (switch, attach, `switch-client`, a second terminal) draws the next frame without waiting for a tick; `width::keep` runs before it, so a width changed meanwhile is applied.
- A hidden sidebar's work per notification is at most one constant-size query, never a `list-panes` of the session; with no notifications, none at all.
- `list-panes -s -f '#{!:#{@sidebar}#{@reader}#{@reader_slot}}'` (same rows `model::tabs` keeps today).
- Files under 100 lines: `src/control.rs` (106) and `src/main.rs` (99) shrink or split by job, e.g. a `src/shown.rs`.

**Never:** Change what is drawn or when the shown sidebar draws; spawn processes; add tmux options or rename existing ones; rely on a hook to tell hidden sidebars they are shown if a notification already says so; test on the default socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Switch | A shown, B hidden; select B | B current at once; A goes quiet | N/A |
| Idle | 50 tabs, 60 s | only the shown sidebar sends commands | N/A |
| Hook burst | `set-buffer sidebar-refresh` 10/s | shown redraws; hidden send at most a tiny query each | N/A |
| Detach | last client detaches | shown sidebar goes quiet | N/A |
| Attach | client attaches to the session | current tab's sidebar draws at once | N/A |
| Two clients | sessions S1 and S2 each shown | both current tabs draw | N/A |
| Width changed while hidden | `@sidebar_width` set | shown again: resized before its frame | N/A |
| New tab | `new-window` (not selected) | its sidebar starts hidden, draws when shown | N/A |

</frozen-after-approval>

## Code Map

- `src/main.rs` -- loop: `display` head, `keep`, `list-panes`, groups file, draw, menu files, clicks, `t.wait(1 s)`.
- `src/control.rs:96-106` `read()` -- maps every `%` line to `Changed`; the notification text is needed now. 106 lines.
- `src/width.rs` `keep()` -- reads `FORMAT` from the head query; may resize the pane.
- `src/model.rs:30` -- skips `@sidebar`, `@reader`, `@reader_slot` rows; stays as a guard.
- `tabrail.tmux:41` -- `after-select-window` hook's `set-buffer`; `client-attached` runs `tabrail-tmux ensure`.
- `README.md` -- "Fast:" bullet.

## Tasks & Acceptance

**Execution:**
- [x] `src/control.rs` -- events carry the notification line; under 100 lines.
- [x] `src/shown.rs` (new, or a fitting name) -- shown state from the start query and notifications, with `#[cfg(test)]` cases for the matrix rows that are pure parsing.
- [x] `src/main.rs` -- skip the frame and the tick while hidden; draw at once on becoming shown; the `list-panes` filter.
- [x] `README.md` -- "Fast:" bullet: only the sidebar shown works; numbers from `tabrail-perf-test`.

**Acceptance Criteria:**
- Given `tabrail-perf-test 5 50` before and after, then the 50-tab switch median is at most 50 ms and at most 1.5x the 5-tab one, `switch@hooks` too, and idle CPU at 50 tabs at most 2x that at 5 (SPEC CAP-1..3).
- Given every `tabrail-*-test`, then each passes as before.

## Implementation Notes

Before and after, back to back, f4l002, 4 cores, tmux 3.8, 2026-11-25: `tabrail-perf-test 5 50` on main
64a48fd (a release build of `git archive` in a scratch dir, 684 s), then on this branch (486 s);
exit 0, nothing on stderr either time:

```
before tabs=5 switch=15ms max=26ms switch@hooks=12ms hooks=9.8/s reader=112ms idle=0.50% hook=14.8ms
before tabs=50 switch=399ms max=493ms switch@hooks=653ms hooks=5.2/s reader=1249ms idle=37.68% hook=348.1ms
after  tabs=5 switch=9ms max=39ms switch@hooks=10ms hooks=9.8/s reader=105ms idle=0.08% hook=15.3ms
after  tabs=50 switch=23ms max=64ms switch@hooks=17ms hooks=9.8/s reader=151ms idle=0.43% hook=16.5ms
```

- AC 1, met: 50-tab switch 23 ms and switch@hooks 17 ms, both at most 50 ms (from 399 and 653 ms);
  idle 37.68% to 0.43%; the hook load now reaches 9.8/s at 50 tabs.
- AC 1, not met: the ratios. Switch 23 ms is 2.6x the 5-tab 9 ms (limit 1.5x), switch@hooks 17 ms is
  1.7x of 10 ms, idle 0.43% is 5.4x of 0.08% (limit 2x). The 5-tab side shrank too (15 to 9 ms,
  0.50 to 0.08%), as its 4 hidden sidebars went quiet. What is left grows with tabs inside tmux:
  - Idle: with the shown sidebar killed, or no terminal attached, the 50-tab server used 0 ticks in
    20 s (8 ticks attached). All of it is the shown sidebar's 1 s `display` + `list-panes`: tmux
    builds a format tree for each of the session's 150 panes to apply the filter, and expands 23
    fields for 50. The 1 s list is what catches changes tmux sends no notification for (current
    command and path, OSC 133 timers), so it stays (Never: change when the shown sidebar draws).
  - Switch, timed per phase in an instrumented scratch copy (5 / 50 tabs): first notification after
    `select-window` ~7 / ~10 ms (each notification goes to every control client, 50 processes
    wake), the new sidebar wakes +0.1 / +1.5-3 ms, `display` 0.5 / ~2 ms, `list-panes` ~0.5 /
    ~3 ms, its draw 0.4 / 5.2 KB (at 50 tabs each switch lands on a sidebar that never drew since
    setup), then ~3 / ~4.5 ms until the outer screen shows it. With the reader off the 50-tab
    switch was still ~20 ms, so story 3 (reader follow) is not the main lever. Pipelining
    `display` with `list-panes` and writing menu files after the draw would save ~2-3 ms, not the
    ~10 ms the 1.5x needs; not done.
- After the review fixes (full repaint when shown again, sidebars' own client notes skipped), the
  branch alone, 09:51, load average 5.2 falling to 2.5, 482 s, exit 0, nothing on stderr or left:
  ```
  tabs=5 switch=8ms max=13ms switch@hooks=9ms hooks=9.8/s reader=99ms idle=0.08% hook=13.9ms
  tabs=50 switch=18ms max=35ms switch@hooks=16ms hooks=9.8/s reader=132ms idle=0.38% hook=15.5ms
  ```
  Ratios 2.25x, 1.8x, 4.75x: still above AC 1's, as accepted. README carries these numbers.
- AC 2: every `tabrail-*-test` passes (bar 24, chat-session 13, chat 25, reader 34, slot 21 ok).
  The slot test's "layout command in a tab with a slot … the sidebar 26" and "a tab with a slot
  resized … the sidebar 26" check a hidden tab's sidebar width, which today's 1 s tick fixed:
  a hidden sidebar now runs `width::keep` (its one `display`, and `resize-pane` if needed) on a
  `%layout-change` of its own window (`shown::Wake::Layout`), within the Boundaries' one
  constant-size query per notification. A switch thus costs the tab left one `display` (the reader
  swap changes its layout).
- Notifications on tmux 3.8 (control client of session `$0`, checked on a throwaway server): a
  terminal attaching or `switch-client` to any session gives `%client-session-changed NAME $S name`,
  a detach or a lost terminal `%client-detached NAME`; a sidebar attaching or leaving gives the same
  for its control client (named `client-<pid>`; terminals by tty); command clients (`tmux
  set-buffer`) give none. A terminal's two cost one `list-clients -t $S -f
  '#{!:#{client_control_mode}}' -F '#{window_id}'` (one line per terminal); a control client's
  (`client-`) cost nothing, so new tabs and `tabrail-tmux restart` do not make every sidebar ask;
  `%session-window-changed` and everything else costs nothing. The first wake asks once (`seen`
  None). A sidebar that was hidden repaints every line on its next frame: tmux may have cut lines
  of its pane on a shrink while no frame ran.
- `swap-window` that puts another window at the current index (`swap-window -s :2 -t :1` with :1
  current) changes the current window without `%session-window-changed` (and without the hook, so the
  reader bar stays behind too): that sidebar stays hidden until the next switch or a click on it.
  Noted in README; tabrail and vmConfig's tmux.conf do not use swap-window.
- A click that reaches a hidden sidebar (a release racing its own `select-window`) runs one frame, as
  today. A menu marked open is forgotten once hidden, so the next show draws at once.
- Splits to stay under 100 lines: `Click` moved to `input.rs`, the menu files to `menu::Files`,
  the tests to `src/shown/tests.rs` (`#[cfg(test)] mod tests;`).
- Matrix checked by `tabrail-shown-test` (`-L tabrail-hx`, 17 checks) by `/proc/<sidebar pid>/io` `wchar` per tab
  over 2-3 s: idle, switch, hook burst (set-buffer 10/s), detach (`detach-client -t` the terminal),
  attach (respawned outer) drawing at once, `@sidebar_width` set while hidden (still 26, then 30
  when shown), new tab `new-window -d`, a second session with its own terminal, `switch-client`
  away and back: all as the matrix says. It also renames the shown tab during the hook burst (on
  screen within 0.3 s) and clicks a sidebar left believing it is hidden by a `swap-window`, which
  fails when `shown.rs` drops its `t.clicks.is_empty()`.

## Spec Change Log

- 2026-11-25, AC 1: the ratio limits (1.5x switch and switch@hooks, 2x idle) are not met (2.6x, 1.7x,
  5.4x); the 50-tab absolute numbers are (23 ms, 17 ms, idle 0.43%). Lynn accepted the result as is.
  The frozen block stays unchanged.

## Review Triage Log

Loop 1 (blind-hunter BH, edge-case-hunter EC, verification-gap VG):

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| EC1+EC8 hidden pane shrunk and grown back with no frame between: rows tmux cut stay blank (`main.rs:56-66`) | medium | patch | `draw::changes` leaves the cursor on the last changed line (the clock, row 0); a shrink deletes rows below the cursor, a grow adds blank ones, `drawn` still equals the size, so only changed lines are drawn. A terminal resize does this to every hidden tab. Force a full repaint after a hidden wake. |
| BH1+EC3 sidebars starting or leaving make every sidebar run `list-clients` (`shown.rs:79`) | low | patch | Verified on tmux 3.8: a control client shows as `%client-session-changed client-<pid> …` / `%client-detached client-<pid>`, terminals by tty name. New tab or close: N queries; restart: ~N². Skip `client-` names. |
| BH6+EC5 `tabrail-shown-test` reads a dead or missing sidebar as quiet (`:7-10`) | medium | patch | Empty pid: `/proc//io` unreadable, both values empty, `"" > ""` false, prints `=`; every quiet check would pass. Print `?`. Header also says "terminal" where wchar counts all writes. |
| BH7 two-session and switch-client checks glob, hook burst does not check the shown redraw (`:20,37-42`) | low | patch | `*"@2+ @3= @4+ "*` passes with `@0+`; `@1+` in the burst comes from the 1 s tick alone. Exact strings; rename a tab during the burst and look for it on screen. |
| VG1+BH8(click) a click on a sidebar that believes it is hidden is never tested (`shown.rs:38`) | medium | patch | Pre-verified: no test clicks such a sidebar; dropping `&& t.clicks.is_empty()` passes every test. `swap-window` onto the current index, press a tab row, check the switch. |
| BH9+BH10+EC6 docs say hidden sidebars cost a `display` only (README Fast, `shown.rs:4-5`) | low | patch | They also ask `list-clients` on a terminal's attach, detach or switch, and `keep` may `resize-pane`. tmux version is stated (README:89): that part false. |
| VG-o2 `tabrail-tmux:18-19` says touching `sidebar-refresh` makes them all redraw | low | patch | Only the shown sidebar redraws now. `width.rs:4` "every other sidebar follows": still true (hidden ones on their layout change or when shown), false. |
| VG-o1 `tabrail-shown-test` untracked while `src/shown*` are intent-to-add | low | patch | Real for the index; the commit adds every file of the story. |
| BH10 (drag) hidden `keep` publishes a border-dragged width only when next woken, maybe hours later | false | reject | Before, every sidebar ran `keep` each second and published 1 s later; same outcome, later. A shared-width change meanwhile resets `seen`, `moved` and cancels it. |
| BH4a+EC2 a tab seen through a grouped or linked session never draws | low | reject | Outside the intent's "shown" (its session's current window). No session groups or `link-window` in tabrail or vmConfig; fix means a session-wide `list-clients` per switch note. |
| BH4b `choose-tree` previews (vmConfig `bind Tab`) show hidden sidebars' last frames | low | reject | Intent: hidden sidebars send tmux nothing, so they do not draw; the preview shows the panes, the sidebar column is cosmetic there. |
| BH4c window id read once; `join-pane`/`break-pane`/`move-window` to another session | low | reject | tabrail never moves a sidebar pane; `move-window` within the session keeps the id. Moving to another session already listed the wrong session before. |
| EC4 after `swap-window` a click on the stale frame is matched against reordered rows | low | reject | Neither tabrail nor vmConfig uses `swap-window`; README states the gap; fixing it needs a new hook and buffer name. |
| BH5 `Shown::wait` untested; a click on a hidden first pass returns `Shown`, keeping `menu_open` | low | reject | `menu_open` is set only by a right press on the shown sidebar and cleared on the next hidden wake; the click race leaves at most a 1 s pause. Refactor to a pure step is more than a correction. |
| BH8 (rest) close current tab, drag reorder, started while detached, `swap-window` pin untested | low | reject | VG checked on 3.8: `kill-window` and `move-window` send `%session-window-changed`, the tested path; started-detached is the same note path as detach/attach. |
| BH2 `pane-command-finished` uses `session_attached`, which counts control clients (`tabrail.tmux:44`) | medium | defer | Pre-existing: no ✓ for the current tab while detached, not caused by this story. |
| BH3 AGENTS.md pitfalls lack this story's tmux findings | low | defer | Fix edits an agent-context file. |
| VG2 the `list-panes` filter and AC 1 numbers have no automated check; perf test always exits 0 | medium | defer | Pre-verified; a timing threshold flakes on the shared VM; `model::tabs` keeps it perf-only. |
| VG3 forgetting the open-menu mark on hide is untested | low | defer | Pre-verified; no right-click harness in the tests. |
| VG4 notes kept during `ask` are untested | low | defer | Pre-verified; needs a fake `Tmux` (the `ChildStdin` field). |
| EC7 README marks AC 1 met | false | reject | README gives numbers only; Implementation Notes and the Spec Change Log state the ratios unmet and accepted. |

## Design Notes

`%session-window-changed $S @W` names the session's new current window, so a sidebar knows from the line alone whether it was shown or hidden by it. Attach and detach of other clients: check on tmux 3.8 what a control client receives (`%client-session-changed`, `%client-detached`, `%sessions-changed`); answer those with one query such as `list-clients -t $S -F '#{client_control_mode}'`. Hidden: `rx.recv()` without timeout.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --offline` -- expected: builds, tests pass.
- `for t in ./tabrail-*-test; do $t || echo "FAIL $t"; done` -- expected: no FAIL.
- `./tabrail-perf-test 5 50` -- expected: the AC numbers; before and after in Implementation Notes.
