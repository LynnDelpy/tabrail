---
title: "Start button at the row's right edge"
type: 'feature'
created: '2026-11-25'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-group-timer/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A group that was never timed, or was reset, shows no time, so starting its timer takes the right-click menu. The user wants to start it directly (2026-11-25).

**Approach:** The header row of a group without a time ends in a dim `▷` in its last column. head.rs reports that column as the row's click span, so story 2's press-and-release toggle runs `tabrail-timer toggle`, which starts a group with no time.

## Boundaries & Constraints

**Always:** The `▷` only on group rows with no timer, in the row's last column (where tab rows put their status), dim (`fg("dim")`) on the group's tint. Never cut: a folded group's count goes first (whole or not at all), then the name is cut, with at least one space before the `▷`. Press and release both on it start the timer; a press on it released elsewhere on the same row folds; a drag from it moves the group, as from the time. After a start the row shows the running time after the name and no `▷`; Reset timer brings the `▷` back. Act on the release. Files under 100 lines.

**Never:** A `▷` on a running (`▶`) or stopped (`⏸`) row: there the time is the button. A `▷` for a name `clean()` changes: drag.rs folds there and the menu has no timer items for it. A new `tabrail-timer` op, a change to the menu's Start timer, hover effects. Lines added to `src/control.rs` (106) or `src/draw.rs` (96).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Start | A never timed, press+release on `▷` | `▶ 0s` after A's name in every sidebar, `▷` gone, `@sidebar_fold` unchanged | N/A |
| Reset | A `⏸ 12m`, menu Reset timer | time gone, `▷` back in the last column | N/A |
| Folded | A folded, 3 tabs, no timer, width 26 | `▸ ◆ Infra  3 tabs        ▷`; a click on `▷` starts, row stays folded | N/A |
| Narrow | name and count do not fit beside `▷` | count dropped first, then the name cut; `▷` in the last column | N/A |
| Beside | press+release one column left of `▷` | folds, no timer | N/A |
| Off | press on `▷`, release on the name, same row | folds, no timer | N/A |
| Drag | press on `▷`, release on another group's row | group line moves (move-group), no timer | N/A |
| Running | A `▶ 12m` | no `▷`; the time toggles as in story 2 | N/A |
| Unclean | a group name `clean()` changes, no timer | no `▷`; a click folds | N/A |

</frozen-after-approval>

## Code Map

Lines as on `origin/lde/group-timer` (PR #35); the same once it is merged.

- `src/head.rs:9-25` -- `row()`: with `time` None, pad the row so `▷` is its last column and return `Some((width, width))` as the span; today the span is None without a time. 28 lines.
- `src/head/tests.rs` -- `text()` strips escapes; `time_after_the_name_then_the_count` expects `"▸ ◆ Infra  3 tabs         "` for no time, which now ends in `▷`. Span tests exist for the time (`span` helper).
- `src/menu.rs:49` -- `clean()`: head.rs uses it to leave the `▷` out (or draw.rs passes a flag without new lines).
- `src/draw.rs:13,70-72` -- `Row::Group(name, span)`: the comment says "its time's first and last column"; make it "the time's or the ▷'s". No new lines.
- `src/drag.rs:1-3,35` -- the toggle arm already compares press and release against the press-time span and checks `clean(&src) == src`; only the module doc should name the `▷`.
- `tabrail-timer:6` -- `toggle` on a group with no time starts it from 0 (no change).
- `tabrail-timer-click-test:50-52` -- "No time" clicks Infra's row at column 5 and expects a fold; keep it, and add Start on `▷`, Beside, Off and a folded row (61 lines; split by job if past 100).
- `README.md:20-21,45` -- groups bullet and Mouse bullet: the `▷` at a timeless group's right edge starts its timer.

## Tasks & Acceptance

**Execution:**
- [ ] `src/head.rs` -- `▷` in the last column of a row without a time, never cut, its column as the span; none for a name `clean()` changes -- CAP-7
- [ ] `src/head/tests.rs` -- `▷` placement unfolded, folded with count, narrow (count dropped, name cut), no `▷` with a time or an unclean name; span `(width, width)` -- CAP-7
- [ ] `src/draw.rs`, `src/drag.rs` -- comments only, unless the toggle arm turns out to need the `▷` -- click constraint
- [ ] `tabrail-timer-click-test` -- Start from `▷` in both sessions, Beside, Off, Drag from `▷`, folded row, Reset brings `▷` back
- [ ] `README.md` -- groups and Mouse bullets -- behaviour reference

**Acceptance Criteria:**
- Given group Infra without a timer shown in two sessions, when its `▷` is pressed and released in one, then both show a running time after Infra's name and no `▷` on their next redraw, and `@sidebar_fold` is unchanged.
- Given Infra's timer reset from the menu, then its row ends in `▷` again.
- Given a running or stopped timer, then its row shows no `▷`.

## Spec Change Log

## Design Notes

At width 26, `▾ ◆ Infra` with no time: `▾ ◆ Infra` + 16 spaces + `▷` (column 26); the span is `(26, 26)`. The click target is one column wide, like a tab row's status edge; the time stays the wider target once running.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `./tabrail-timer-click-test` -- expected: every check ok, on its own tmux -L servers with XDG_STATE_HOME on a temp dir
