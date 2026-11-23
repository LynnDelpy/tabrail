---
title: 'Click the time to stop or start'
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

**Problem:** After story 1 a group's timer stops and starts only from its right-click menu, two clicks away, while the time sits right there on the header row.

**Approach:** A left press and release on the time itself runs `tabrail-timer toggle GROUP`. The sidebar learns the click's column, and draw reports which columns of each group row the time covers.

## Boundaries & Constraints

**Always:** Toggle only when press and release both land on the time's columns of the same group row. Any other press and release on that row folds as now; a press on the time released on another row is a group drag as now. Columns are counted as draw counts them (one per char, SGR columns 1-based). Act on the release, as drag.rs does. Files under 100 lines.

**Never:** Switch tabs or fold on the press. A clickable target where no time is shown (a reset or never-started group): starting stays in the menu. Adding lines to `src/control.rs` (106 already). New dependencies.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Stop | `▶ 12m` running, press+release on it | `⏸ 12m`, frozen, in every sidebar | N/A |
| Resume | `⏸ 12m`, press+release on it | `▶ 12m`, counting from there | N/A |
| Fold | press+release on the group's name | folds, timer unchanged | N/A |
| Off the time | press on the time, release on the name, same row | folds, timer unchanged | N/A |
| Drag | press on the time, release on another group's row | group line moves (move-group), timer unchanged | N/A |
| Space after | press+release right of the time on a running row | folds | N/A |
| No time | group never timed, click anywhere on its row | folds | N/A |
| Tab row | press+release on a tab's status time | goes to the tab as now | N/A |

</frozen-after-approval>

## Code Map

- `src/input.rs:18` -- builds `Click` from `n[0..3]`; `n[1]` is the column, dropped today.
- `src/control.rs:11-15` -- `struct Click { right, release, row }`; 106 lines, put `col` on an existing line.
- `src/draw.rs:33-36` -- `Screen { lines, rows }`; needs the time's column span per group row, set where story 1 places the label.
- `src/drag.rs:16-31` -- `left()`: `drag` holds `(row, Row)` from the press; `from == click.row` on a group folds. Needs the press column too. 79 lines.
- `src/main.rs:86` -- calls `drag::left` with `&helper`; the timer script's path (story 1's `Place.timer`) has to reach it.
- `README.md:33` -- Mouse bullet: click a group's time to stop or start its timer.

## Tasks & Acceptance

**Execution:**
- [ ] `src/control.rs`, `src/input.rs` -- `Click.col` from `n[1]` -- where the click landed in the row
- [ ] `src/draw.rs` -- `Screen` carries, per group row with a time, its first and last column -- draw is the only place that knows after cutting the name
- [ ] `src/drag.rs` -- keep the press column; on release on the same row with both columns inside the span, `run-shell -b '<tabrail-timer> toggle <name>'` plus nothing else; otherwise today's fold, reorder and drop -- CAP-2
- [ ] `src/main.rs` -- pass the span and the timer path into `drag::left` without growing past 100 lines
- [ ] `README.md` -- Mouse bullet: clicking a group's time stops or resumes it -- behaviour reference
- [ ] `src/drag.rs` or `src/draw.rs` unit tests -- span hit and miss at the time's first and last column and one beside each, folded and unfolded rows

**Acceptance Criteria:**
- Given a running group timer shown in two sessions, when the time is clicked in one, then both show it stopped on their next redraw.
- Given a click on the time, then the group's fold state (`@sidebar_fold`) is unchanged.

## Design Notes

Header `▾ ◆ Website ▶ 12m` at width 26: the time is ` ▶ 12m` without its leading space, columns 13-17 (`▾`=1, ` `=2, `◆`=3, ` `=4, `Website`=5-11, ` `=12). Include the marker in the target so the click area is a few columns wide.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `wc -l src/*.rs` -- expected: touched files at or under 100, `control.rs` not longer
- On a `tmux -L test` server shown in `tmux -L outer` with `XDG_STATE_HOME` on a temp dir: start a timer from the menu, then `send-keys -l $'\e[<0;C;RM'` and `$'\e[<0;C;Rm'` with C on the time and R its row -- expected: `capture-pane -p` shows `⏸` and the same value 3 s later; repeat -- expected: `▶`; repeat with C on the name -- expected: group folds, timer unchanged
