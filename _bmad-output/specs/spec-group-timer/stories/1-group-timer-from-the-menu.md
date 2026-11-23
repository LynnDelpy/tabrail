---
title: 'Group timer from the menu'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-group-timer/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Nothing in the sidebar measures time the user starts and stops by hand, so the time spent on one project (a group) is unknown.

**Approach:** A stopwatch per group name, kept in a state file as accumulated seconds plus a start epoch. `tabrail-timer` changes it; every sidebar reads it each redraw and shows `▶ 12m` / `⏸ 12m` right after the group's name. The group menu gets Start/Stop timer and Reset timer; Rename group carries the timer along.

## Boundaries & Constraints

**Always:** State file `$XDG_STATE_HOME/tabrail/timers`, else `~/.local/state/tabrail/timers`, dir created on first write. Read-modify-write under `flock`, written to a temp file in the same dir and renamed into place; then `tmux set-buffer -b sidebar-refresh x`. Wall-clock only: shown = accumulated + (now - since) while running. Missing, empty or garbled file or line: that group has no timer, the sidebar draws as before. Time after the name, before a folded group's `N tabs`; the count is dropped first, then the name cut; the time is never cut. Format via `model::ago` (`45s`, `12m`, `1h5m`). Files under 100 lines.

**Never:** Timer state in the groups file or only in tmux options. Counting inside the sidebar process. Timer items for a group whose name `clean()` changes (it would key the wrong timer). Automatic start/stop, history, totals, status-line display. New dependencies.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Start | Website never timed, Start timer | `Website 0 NOW` line; header `▾ ◆ Website ▶ 0s` in every sidebar | N/A |
| Stop | running 90 s, Stop timer | `Website 90 0`; header `⏸ 1m`, frozen | N/A |
| Resume | stopped at 90, Start timer | `Website 90 NOW`; counts from 1m | N/A |
| Start twice | running, `tabrail-timer start Website` | unchanged | no-op |
| Reset | running or stopped, Reset timer | line gone; no time in header; menu has no Reset | N/A |
| Two groups | Website and Infra started | both count; Stop on one leaves the other | N/A |
| Rename | Website with a time renamed to Web | line now `Web …`, same values; header of Web shows it | N name already timed: old's timer replaces it |
| Restart | stopped at 90, `kill-server`, new server | `⏸ 1m` again | N/A |
| Down time | running, server down 10 s | shows old value + 10 s | N/A |
| Narrow | width 26, group name 30 chars, running 2h | name cut, ` ▶ 2h0m` whole at its end | N/A |
| Broken file | file holds garbage | no timers; sidebar draws | lines that do not parse are skipped; next write drops them |

</frozen-after-approval>

## Code Map

- `src/draw.rs:55-70` -- `screen()` builds `head` (fold arrow, icon, name, folded `N tabs`) and `fit()`s it; the time goes in here. 96 lines.
- `src/model.rs:62` -- `fn ago`, private; make it `pub` for the timer label.
- `src/menu.rs:50-58` -- the `Row::Group` branch; `Place` holds helper paths; `clean()`.
- `src/main.rs:28-30,42,66,69` -- helper paths, `Place`, `draw::screen` and `menu::for_row` calls. 99 lines.
- `tabrail-act:43-54` -- `rename_group`: the guard and the hyphenated `$new`; 99 lines.
- `tabrail-name` -- calls `tabrail-act rename-group`; unchanged.
- `README.md:17,33,94` -- groups bullet, Mouse bullet (menu list), Files table.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-timer` -- new: `start|stop|toggle|reset GROUP`, `rename OLD NEW`; one line per timer `NAME ACCUMULATED SINCE` (SINCE 0 = stopped); flock, temp + rename, refresh -- the only writer
- [ ] `src/timer.rs` -- new: read the state file into name -> (accumulated, since); `label(name, now) -> Option<(String, kind)>` giving `▶ …` busy / `⏸ …` dim -- reading and formatting out of main.rs and draw.rs
- [ ] `src/draw.rs` -- `screen()` takes the timers; header = arrow, icon, name cut to fit, ` ` + label in its colour, folded count if it fits -- CAP-1, never-cut constraint
- [ ] `src/menu.rs` -- `Place.timer`; group menu after a separator: `'Start timer' t` or `'Stop timer' t` by state, `'Reset timer' 0` only when timed; Default included; skipped when `clean(name) != name` -- CAP-1..3
- [ ] `src/main.rs` -- build the helper paths in one line to make room; `mod timer`; read timers once per loop, pass to `screen` and `for_row` -- stay at or under 100 lines
- [ ] `src/model.rs` -- `pub fn ago`
- [ ] `tabrail-act` -- `rename_group` runs `"$D/tabrail-timer" rename "$old" "$new"` after its guard -- CAP-6
- [ ] `README.md` -- timer in the groups bullet, Start/Stop/Reset timer in the Mouse bullet's menu list, `tabrail-timer` and `src/timer.rs` in Files -- behaviour reference

**Acceptance Criteria:**
- Given a running timer in one session, when another session's sidebar shows that group, then it shows the same time.
- Given the right-click menu of a group with no time, then it has Start timer and no Reset timer; while running, Stop timer and Reset timer.
- Given two sidebars click Start timer and Stop timer at the same moment, then the state file holds one well-formed line for the group.

## Design Notes

State file, one line per timed group (names are one word, as in groups.conf):

```
Website 5400 0
Infra 120 1795001234
```

Header at width 26: `▾ ◆ Website ▶ 12m` (running, `f9e2af`), `▸ ◆ Infra ⏸ 2m  3 tabs` (stopped, `6c7086`). The time's colour must not drop the row's tint: set only the foreground, keep the `48;2` background and bold to the row's end.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-timer tabrail-act && wc -l tabrail-timer tabrail-act src/*.rs` -- expected: syntax ok; new and touched files at or under 100
- `XDG_STATE_HOME=$(mktemp -d) tmux -L test -f <conf> …` with tabrail loaded, shown in `tmux -L outer` (AGENTS.md): SGR right-click a group row, Start timer -- expected: `capture-pane -p` of the sidebar shows `▶` and a growing time after the name; the temp dir's `tabrail/timers` holds the line
- Same server: Stop timer, wait 3 s, capture twice -- expected: identical `⏸` value; Reset timer -- expected: no time, line gone
- `tmux -L test kill-server`, start again with the same `XDG_STATE_HOME` -- expected: the stopped value is back
