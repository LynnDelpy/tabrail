---
title: 'Hidden sidebars stay idle'
type: 'refactor'
created: '2026-11-24'
status: 'ready-for-dev'
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
- [ ] `src/control.rs` -- events carry the notification line; under 100 lines.
- [ ] `src/shown.rs` (new, or a fitting name) -- shown state from the start query and notifications, with `#[cfg(test)]` cases for the matrix rows that are pure parsing.
- [ ] `src/main.rs` -- skip the frame and the tick while hidden; draw at once on becoming shown; the `list-panes` filter.
- [ ] `README.md` -- "Fast:" bullet: only the sidebar shown works; numbers from `tabrail-perf-test`.

**Acceptance Criteria:**
- Given `tabrail-perf-test 5 50` before and after, then the 50-tab switch median is at most 50 ms and at most 1.5x the 5-tab one, `switch@hooks` too, and idle CPU at 50 tabs at most 2x that at 5 (SPEC CAP-1..3).
- Given every `tabrail-*-test`, then each passes as before.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Design Notes

`%session-window-changed $S @W` names the session's new current window, so a sidebar knows from the line alone whether it was shown or hidden by it. Attach and detach of other clients: check on tmux 3.8 what a control client receives (`%client-session-changed`, `%client-detached`, `%sessions-changed`); answer those with one query such as `list-clients -t $S -F '#{client_control_mode}'`. Hidden: `rx.recv()` without timeout.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --offline` -- expected: builds, tests pass.
- `for t in ./tabrail-*-test; do $t || echo "FAIL $t"; done` -- expected: no FAIL.
- `./tabrail-perf-test 5 50` -- expected: the AC numbers; before and after in Implementation Notes.
