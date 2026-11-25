---
title: 'Group timer from the menu'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: '567fcea9ce0057f509665cec7c9f0792b5c31c5f'
route: 'dispatch'
review_loop_iteration: 1
context:
  - '{project-root}/_bmad-output/specs/spec-group-timer/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Nothing in the sidebar measures time the user starts and stops by hand, so the time spent on one project (a group) is unknown.

**Approach:** A stopwatch per group name, kept in a state file as accumulated seconds plus a start epoch. `tabrail-timer` changes it; every sidebar reads it each redraw and shows `▶ 12m` / `⏸ 12m` right after the group's name. The group menu gets Start/Stop timer and Reset timer; Rename group carries the timer along. Decision (2026-11-25, review pass 1): a group with a timer keeps its header row while it has no tabs, so its time and menu stay in reach; a group without a timer and without tabs stays hidden as now.

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
| Last tab closed | Website running, its last tab closed | header `▾ ◆ Website ▶ …` stays with no tab rows, still counting; its menu's Stop timer stops it | N/A |
| No tabs, no time | Infra without tabs and without a timer | no Infra header, as before | N/A |

</frozen-after-approval>

## Code Map

- Pass 1 is on local branch `lde/group-timer-pass1` (one WIP commit on the baseline; also /tmp/claude-1002/-home-lde-Dev-tabrail-build/6e4f37d2-55cf-4e7e-9543-3ccaee7e2d6b/scratchpad/group-timer-story1-pass1.patch): `tabrail-timer`, `src/timer.rs` + `src/timer/tests.rs`, draw/menu/main/model/tabrail-act/README changes and `tabrail-timer-test` (18 checks, all passed). Start from it: `git diff HEAD lde/group-timer-pass1 | git apply`, then the Tasks below. It reviewed clean apart from what they list.
- `src/draw.rs` (pass 1: 100 lines) -- `screen()`: `if members.is_empty() { continue }` at the loop top hides empty groups; the pass-1 header block (`time`, `count`, `head`, the `lines.push`) moves out. `fg`, `rgb`, `fit` are private; `pub(crate)` them for the new file.
- `src/drag.rs:69` -- a tab dropped on a group row without tabs only sets `@group`: already works for an empty header. `src/menu.rs` `Row::Group` branch needs no change.
- `tabrail-act` `rename_group` (pass 1: 100 lines) -- the `tabrail-timer rename` call sits before the groups.conf awk/mv.
- `src/timer.rs` `Timers::read` -- `fs::read_to_string`.
- `tabrail-test-lib` -- throwaway servers `-L tabrail-$L`, `inner`, `side`, `check`; the outer server runs `-f /dev/null`, so its first window is `o:0`.

## Tasks & Acceptance

**Execution:**
- [x] `src/head.rs` -- new: the group header row out of `draw::screen` (arrow, icon, name cut and `trim_end()`ed, ` ` + label in its colour, then the folded count only when it fits whole, else none, padded to the width; tint and bold to the row's end) -- keeps draw.rs under 100, one place for story 2's time columns
- [x] `src/draw.rs` -- skip a group only when it has no tabs and no timer (`timers.running(name).is_none()`); an empty timed group gets its header and no count -- decision
- [x] `tabrail-act` -- `rename_group`: call `tabrail-timer rename` right after the groups.conf `mv` succeeds, not before -- no frame of OLD without its time; no line added
- [x] `tabrail-timer` -- `rename` refuses an empty NEW (usage, exit 2) -- `rename A ""` deleted A's timer
- [x] `src/timer.rs` -- read with `String::from_utf8_lossy(&fs::read(..))` -- a non-UTF-8 byte dropped every timer
- [x] `tabrail-timer-test` -- sidebar side (throwaway servers): add Default's menu and Start, a folded timed group (`▸ ▲ Ops ▶ Ns  1 tab`; at width 26 the long group folded shows no count), a dirty group name (`Q'a`) without timer items, `capture-pane -e` (busy `249;226;175` right before `▶`, no `0m`/`22m`/`49m` between name and time), Stop's value at least the seconds run and resume adding to it, and the last tab closed (header stays, counting; Stop from its menu) -- matrix and review gaps
- [x] `tabrail-timer-state-test` -- new, script only (temp `XDG_STATE_HOME`, `TMUX` on a socket that does not exist): toggle twice, start twice, reset of a running one then start from 0, rename (timed, untimed, onto timed, empty NEW refused), no state file (exit 0, nothing made), usage exit 2, garbage dropped on write, 40 racing start/stop on one group and 40 starts on distinct groups (40 lines kept) -- moved out to keep both under 100
- [x] `README.md` -- groups bullet: a timed group keeps its header without tabs; Files table: `src/head.rs`, `tabrail-timer-state-test`

**Acceptance Criteria:**
- Given a running timer in one session, when another session's sidebar shows that group, then it shows the same time.
- Given the right-click menu of a group with no time, then it has Start timer and no Reset timer; while running, Stop timer and Reset timer.
- Given two sidebars click Start timer and Stop timer at the same moment, then the state file holds one well-formed line for the group.
- Given 40 starts on 40 different groups at once, then the state file holds 40 lines.
- Given a group whose timer runs, when its last tab closes, then its header stays with the time counting and its menu still stops it.

## Spec Change Log

- Pass 1 review, intent_gap B4: a running timer of a group whose last tab closed counted on unseen. Human chose: keep a timed group's header without tabs (frozen Approach + two matrix rows). Known-bad state avoided: a timer counting with no header and no menu. KEEP: everything on `lde/group-timer-pass1` (the script's format, flock + rename, the menu items and their quoting, the clean() guard, main.rs's one-line helper paths, the README text), plus the pass-1 patch findings now in Tasks.

## Review Triage Log

Pass 1 (blind B, edge-case E, verification-gap V):

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| B4 | A running timer of a group whose last tab closes counts on unseen (draw skips a group without tabs, so no header and no menu to stop it) | medium | `draw.rs:63` `members.is_empty() => continue`; timer keeps running; editing a time is a non-goal, so the overcount can only be reset away. Spec covers removed groups, not tabless ones; auto-stop vs. drawing the header vs. accepting it are all readable | intent_gap |
| B5, E15 | Narrow folded header cuts the count (`  3 ta`) instead of dropping it | low | `draw.rs` `fit(&count, room)` cuts per char; frozen constraint says "the count is dropped first" | patch |
| B1, E8 | rename_group renames the timer before groups.conf; its refresh draws one frame of OLD without a time; a failed conf write orphans it | low | `tabrail-act:46` runs before the awk/mv at :47-48 | patch (call after the mv) |
| B6b, E2 | Name cut ending at `▾ ◆ ` leaves two spaces before the time | low | `.take()` without `trim_end()`, tab rows trim | patch |
| B7, E9 | `tabrail-timer rename A ""` deletes A's timer | low | `$3` not checked non-empty; awk sets `names[i] = ""` | patch |
| E3 | A non-UTF-8 byte in the state file makes `read_to_string` fail: every timer vanishes | low | `timer.rs` read; awk keeps such a line, so it persists; only by hand | patch (`from_utf8_lossy`) |
| B8 | No test for `toggle`, Reset of a running timer then Start from 0 (CAP-3), no state file, usage, rename from an untimed name | low | `tabrail-timer-test` has none of these | patch (tests) |
| V1 | Racing-writes check passes without flock | low | gap layer: 40 starts on distinct groups lose lines without the lock, test only races one group | patch (tests) |
| V2 | Stop's added seconds never checked as numbers | medium | gap layer: `acc[g] =` instead of `+=` passes every check | patch (tests) |
| V3, B8 | Folded header with a time (order, count dropped first) untested | low | gap layer: no test folds a group | patch (tests) |
| V4, B8 | Default's timer items untested | low | gap layer: only Website/Infra right-clicked | patch (tests) |
| V5 | clean() guard on timer items untested | low | gap layer: no dirty group name in any test | patch (tests) |
| V6 | Tint, bold and time colour untested | low | gap layer: `capture-pane -p` strips escapes | patch (tests) |
| V7 | Fallback state path computed twice (Rust and bash), never cross-checked | low | gap layer, filed as defer | defer |
| B10 | AGENTS.md does not tell test servers to set `XDG_STATE_HOME` | medium | a `tmux -L test` sidebar's menu writes the user's real timers file | defer (agent-context file) |
| B3, E7 | A timed group renamed to a name `clean()` changes keeps a timer with no Stop item | low | needs `'"{};\$#` in a group name; fix adds a guard; spec's Never rule chose no items for such names | reject |
| E6 | Rename group of a dirty name moves the cleaned name's timer | low | needs a group `a#b` and a timed `ab`; pre-existing cleaned-name rename bug; fix adds a guard | reject |
| B6a, E1 | Sidebar narrower than the time label: row overflows | low | width below ~8 columns; tab rows overflow the same way; fix adds a cut the spec forbids | reject |
| E4 | `\r` or Unicode whitespace in a hand-edited line: Rust parses, awk drops | low | writer never produces it; fix adds a guard | reject |
| E5 | Relative or changed `XDG_STATE_HOME`, unset HOME: reader and writer differ | low | unlikely setup; fix adds guards | reject |
| E12 | Symlinked state file replaced by a regular file | low | state dir is not config; fix adds readlink | reject |
| B12, E13 | No trap for the temp file; flock without timeout | low | window between mktemp and mv is milliseconds; awk cannot hang; fix adds guards | reject |
| B9, E14 | `num()` reads only `Ns`: a check past 59 s errors | low | the checks run 3-15 s of fixed sleeps after start; fix adds parsing | reject |
| B2 | Rename onto a timed name: overwrites if OLD timed, else NEW's stale time shows | false | both are specified: matrix "old's timer replaces it", SPEC assumption "shows again if a group of that name comes back" | reject |
| E10 | A removed group's timer shows again when the name returns | false | SPEC assumption states exactly this | reject |
| B11 | draw.rs and tabrail-act at 100 lines leave no room for story 2 | false | story Verification: "at or under 100"; story 2 plans its own room | reject |
| E11 | `EPOCHSECONDS` needs bash 5 | false | `tabrail-reader` already relies on it; the repo requires bash 5 | reject |

Pass 2 (blind B2, edge-case E2, verification-gap V2):

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| B2-1 | State file present but unreadable: `cat` fails silently, the next write keeps only the changed group | medium | `cat "$FILE" 2>/dev/null \| awk` without pipefail; reproduced with a mode-000 file (e.g. root-owned after a sudo run) | patch |
| B2-2, E2-1, E2-6 | `rename` compares number-like names numerically (`07` vs `7`, `1e2` vs `100`) | low | awk strnums from `$1`/`ENVIRON`; reproduced by two reviewers; `tabrail-act move_group` already forces `x ""` | patch |
| B2-5 | README silent on an orphaned timer (group removed from groups.conf) and on `tabrail-timer reset` by hand | low | behaviour from SPEC assumption, README is the behaviour reference | patch |
| B2-6 | `tabrail-test-lib` does not set `XDG_STATE_HOME`: every other test's sidebars read the real timers file, a group right-click there would write it | low | only `tabrail-timer-test` exports it | patch (lib) |
| B2-8 | `head/tests.rs` comment arithmetic reads wrong | low | 26 - 13 - 7 = 6, comment says 26 - 13 | patch |
| B2-9, V2-1 | Stop with a start time in the future: the writer's clamp is untested | low | gap layer: guard removed, state test still 12/12, Stop then writes `A -510 0` | patch (test) |
| V2-2 | Dim colour of a stopped time never checked | low | gap layer: `fg("busy")` for all passes 11/11 | patch (test) |
| V2-3 | Bold never checked | low | gap layer: `\x1b[1m` removed passes 11/11 | patch (test) |
| V2-4 | rename-group with an unwritable groups.conf: timer must stay under OLD, untested | low | gap layer, filed as defer | defer |
| B2-7 | Deferred items never recorded; AGENTS.md lacks the state file in Conventions | low | pass 1 loopback made them moot; recorded this pass (V7, B10 + Conventions, V2-4) | defer |
| B2-3 | Reset timer has no confirm-before | low | by design: CAP-3 is one menu item; a misclick needs right-click then the wrong item; fix adds a prompt | reject |
| B2-10 | tabrail-timer's own refresh mid-rename draws one half-renamed frame | low | one frame on a rare action; fix adds a flag to the script | reject |
| B2-4 | A stopped timer keeps an empty header in every session until Reset | false | the frozen decision: "a group with a timer keeps its header row while it has no tabs"; CAP-1 "every sidebar of every session"; README says it | reject |
| E2-2 | Rename to a name clean() changes: timer without Stop item | low | carried: B3/E7, code unchanged | reject |
| E2-3, E2-7 | Width below the time label: row overflows | low | carried: B6a/E1, `head.rs` keeps the same rule | reject |
| E2-4 | CRLF or Unicode whitespace in a hand-edited line | low | carried: E4, writer never produces it | reject |
| E2-5 | Removed group's timer returns with the name | false | carried: E10, SPEC assumption | reject |

## Design Notes

State file, one line per timed group (names are one word, as in groups.conf):

```
Website 5400 0
Infra 120 1795001234
```

Header at width 26: `▾ ◆ Website ▶ 12m` (running, `f9e2af`), `▸ ◆ Infra ⏸ 2m  3 tabs` (stopped, `6c7086`). The time's colour must not drop the row's tint: set only the foreground, keep the `48;2` background and bold to the row's end.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, no new warnings, all tests pass
- `bash -n tabrail-timer tabrail-act tabrail-timer-test tabrail-timer-state-test && wc -l tabrail-timer tabrail-act tabrail-timer*-test src/*.rs` -- expected: syntax ok; new and touched files at or under 100
- `./tabrail-timer-state-test` -- expected: every line `ok`, exit 0; no server touched
- `./tabrail-timer-test` -- expected: every line `ok`, exit 0 (throwaway servers, temp `XDG_STATE_HOME`; `~/.local/state/tabrail` not created by it)
