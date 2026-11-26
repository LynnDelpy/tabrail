---
title: 'Group rows without the fold arrow'
type: 'feature'
created: '2026-11-26'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-no-fold-arrow/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every group row starts with a fold arrow (`▾` open, `▸` folded) before its icon. Lynn wants it gone (2026-11-26): the row already shows a fold through `N tabs` and the missing tab rows, and the arrow takes 2 columns from the name.

**Approach:** head.rs builds the header as `{icon} {name}` instead of `{▾|▸} {icon} {name}`. Everything right of the name (time, count, `▷`) and the click spans come from the head's length, so they move left by 2 by themselves. Tests and README follow.

## Boundaries & Constraints

**Always:** Icon in column 1, name from column 3, open or folded. Tint, bold, the time's colour, the `▷`, the folded count and the cut order (count first, then name) as today. A press and release anywhere on the row folds or unfolds it, and the time and `▷` toggle the timer at their new columns. Tab rows unchanged (`❯` column 2, names column 4). Act on the release. Files under 100 lines (`src/drag/tests.rs` is at 100: change lines, add none).

**Never:** A replacement fold mark (glyph, colour, count on open groups). Changes to `❯`, group icons, the menu's Fold/Unfold, `@sidebar_fold` or drag.rs's logic. Testing on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open, no timer | Website, width 26 | `◆ Website` + 15 spaces + `▷ `; `▷` span (25, 25) | N/A |
| Folded, no timer | Infra, 3 tabs, width 26 | `◆ Infra  3 tabs` + 9 spaces + `▷ ` | N/A |
| Running | Website `▶ 12m`, width 26 | `◆ Website ▶ 12m`, time span (11, 15), open or folded | N/A |
| Long name | `Abcdefghijklmnopqrstuvwxyz0123` folded, `▶ 2h0m`, width 26 | `◆ Abcdefghijklmnopq ▶ 2h0m`, span (21, 26) | N/A |
| Fold click | press and release on Infra's name (column 3) | folds; Infra in `@sidebar_fold`; row reads `▲ Infra  1 tab` | N/A |
| Empty folded | a folded group with no tabs and a timer | `◆ Ops ▶ 5s`, same as open | N/A |

</frozen-after-approval>

## Code Map

- `src/head.rs:1,24` -- module doc says "fold arrow, icon and name"; line 24 `format!("{} {icon} {name}", if fold { "▸" } else { "▾" })` is the change. `fold` still drives the count.
- `src/head/tests.rs:25-89` -- expected strings start with `▾ ◆ `/`▸ ◆ `; spans (`:63-65`) and the narrow cases (`:40,74,75`: widths 11 and 6) need recomputing, since the name now gets 2 more characters.
- `src/drag/tests.rs:39-60` -- `the_time_is_columns_13_to_17` and the click pairs `(13, 13)…(17, 13)`, `14`: the time is now columns 11–15. `(25, 25)` for the `▷` stays.
- `src/drag.rs`, `src/draw.rs` -- no change: the fold is any press and release on the row, and `Row::Group` carries head.rs's span.
- `tabrail-timer-click-test:29-80` -- `c == 13` becomes 11; `clk 5` (on the name) becomes 3; fold state checks `== "▸ "*`, `== "▾ "*`, `^▾ ▲ Infra` read the arrow: use the `fold` helper (`@sidebar_fold`) and the row without its arrow.
- `tabrail-timer-test:40,68,83,92` -- expected rows `▾ ◆ $W ▶ `, `▸ ▲ Ops ▶ `, `▾ ● Abcdefghijklmno ▶ 2h0m` (the cut name gains 2 chars).
- `tabrail-colour-test:34,41,47,70,73,77` -- `menu "▾ ◆ Website"` / `"▾ ▲ Infra"` grep the row: drop the `▾ `. The right-click at column 5 still lands on the row.
- `README.md:7,11` -- example group rows: `◆ Website` + 15 spaces + `▷`, `▲ Infra` + 17 spaces + `▷`, annotations stay at column 36.
- spec-nested-groups stories 1 and 4 (`_bmad-output/specs/spec-nested-groups/stories/`) -- ready-for-dev, draw `{indent}{▾|▸} {icon} {name}`. If story 1 is merged before this one, drop the arrow from its indented headers too (name at depth + 3).

## Tasks & Acceptance

**Execution:**
- [ ] `src/head.rs` -- header `{icon} {name}`, module doc without the fold arrow -- CAP-1
- [ ] `src/head/tests.rs` -- expected rows and spans without the arrow: open, folded with count, long name cut 2 later, widths 11 and 6 -- CAP-1
- [ ] `src/drag/tests.rs` -- time columns 11–15 in the column test and the click pairs; no new lines -- fold and time clicks unchanged
- [ ] `tabrail-timer-click-test`, `tabrail-timer-test`, `tabrail-colour-test` -- rows without the arrow, fold state from `@sidebar_fold`, name click at column 3, time from column 11 -- CAP-1
- [ ] `README.md` -- example group rows -- behaviour reference

**Acceptance Criteria:**
- Given a sidebar with groups Website and Infra on a throwaway server, when it is captured, then no row contains `▾` or `▸` and both group rows start with their icon in column 1.
- Given Infra open, when its row is pressed and released, then Infra is in `@sidebar_fold`, its tab rows are gone and its row reads `▲ Infra  1 tab`; a second press and release brings them back.
- Given Website's timer running, when its time is pressed and released, then the timer stops and the fold state is unchanged.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `./tabrail-timer-click-test && ./tabrail-timer-test && ./tabrail-colour-test` -- expected: every check ok, on their own tmux -L servers
- `grep -rn '[▾▸]' src README.md tabrail-*` -- expected: no hits
