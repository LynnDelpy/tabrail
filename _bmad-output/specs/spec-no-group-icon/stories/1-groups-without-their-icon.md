---
title: 'Groups without their icon'
type: 'feature'
created: '2026-11-26'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-no-group-icon/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every group row shows the groups.conf icon (`✚`, `■`, `★`, `◆` …, Default's `•`) between the fold arrow and the name, and the two pickers show it too. Lynn wants it gone (2026-11-26).

**Approach:** head.rs builds the header as `{▾|▸} {name}`; the swatch in tabrail-colour and the line in tabrail-move drop the icon. The time, count, `▷` and their click spans come from the head's length, so they move left by 2 by themselves. groups.conf keeps the column; tests and docs follow.

## Boundaries & Constraints

**Always:** Arrow in column 1, name from column 3, open or folded, Default too. Tint, bold, the time's colour, the `▷`, the folded count and the cut order (count first, then name) as today. A press and release on the row folds; on the time or `▷` it toggles the timer at their new columns. groups.conf read as today (4 columns, icon in column 3); New group… still writes an unused icon. Tab rows unchanged. Files under 100 lines (`src/drag/tests.rs` is at 100: change lines, add none).

**Never:** Removing the fold arrow; changes to `❯` or tab rows; dropping the icon column or rewriting anyone's groups file; a replacement mark; testing on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open, no timer | Website `◆`, width 26 | `▾ Website` + 15 spaces + `▷ `; `▷` span (25, 25) | N/A |
| Folded, no timer | Infra `✚`, 3 tabs, width 26 | `▸ Infra  3 tabs` + 9 spaces + `▷ ` | N/A |
| Running | Website `▶ 12m`, width 26 | `▾ Website ▶ 12m`, time span (11, 15), open or folded | N/A |
| Long name | `Abcdefghijklmnopqrstuvwxyz0123` folded, `▶ 2h0m`, width 26 | `▸ Abcdefghijklmnopq ▶ 2h0m`, span (21, 26) | N/A |
| Default | tabs in no group | `▾ Default`, no `•` | N/A |
| Colour… | Website's menu, Colour… | swatches ` Website ` on each tint, `· current` mark as today | N/A |
| Move to group | a tab in Website, prefix M | `Website  1 tab · current`, `Infra  …`, `Default  …`, no icons; search on names as today | N/A |
| Old file | groups.conf lines with 4 columns, icons included | read as today, icons ignored | N/A |

</frozen-after-approval>

## Code Map

- `src/head.rs:1,16,24` -- module doc "fold arrow, icon and name"; `row()` takes `(name, tint, icon)`; line 24 `format!("{} {icon} {name}", …)` is the change. Keeping the tuple and ignoring the icon (`_icon`) spares every caller and test helper.
- `src/model.rs:49,58` -- `groups()` reads the icon; Default's `"•"`. May stay as data, or Default's becomes empty; no format change.
- `src/head/tests.rs:25-89` -- expected rows start `▾ ◆ `/`▸ ◆ `; spans (`:63-65`) and narrow cases (`:40,74,75`: widths 11 and 6) need recomputing, the name now gets 2 more characters.
- `src/drag/tests.rs:39-60` -- time columns 13–17 and click pairs `(13, 13)…(17, 13)`, `14` become 11–15; `(25, 25)` for the `▷` stays.
- `src/drag.rs`, `src/draw.rs` -- no change: `Row::Group` carries head.rs's span.
- `tabrail-colour:2,32,40` -- header comment, `read -r tint icon`, swatch `' %s %s '` with `$icon "$group"`: swatch ` $group `.
- `tabrail-move:22-29` -- awk prints `$1 "\t" $3`, Default `'Default\t•'`, line `"$icon" "$name"`: name only. tabrail-pick searches the last shown field, unchanged.
- `tabrail-act:7,14,82-83` -- New group… keeps picking an unused icon; no change.
- `tabrail-timer-test`, `tabrail-timer-click-test`, `tabrail-colour-test` -- greps `"◆ $W"`, `"▲ Infra"`, `"■ Q'a"`, `"• Default"`, `"▾ ◆ Website"`, swatch `"│ [ ❯]  ◆ Website "`, expected rows `"▾ ◆ $W ▶ "`, `"▸ ▲ Ops ▶ "`, `"▾ ● Abcdefghijklmno ▶ 2h0m"` (gains 2 chars); click test's `c == 13` becomes 11 (`clk 5` still lands on the name). Find group rows by `^[▾▸] NAME` so tab rows do not match.
- `README.md:7-11,17,19` -- example group rows (`▾ Website` + 15 spaces + `▷`, annotations at column 36), "a faint tint and an icon", the swatch "with the group's icon and name".
- `groups.conf.example:6` -- column list: icon kept, not shown.
- spec-nested-groups stories 1 and 4 -- draw `{indent}{▾|▸} {icon} {name}`. If story 1 is merged before this one, drop the icon from its indented headers too (name at depth + 3).

## Tasks & Acceptance

**Execution:**
- [ ] `src/head.rs` -- header `{▾|▸} {name}`, doc without the icon -- CAP-1
- [ ] `src/head/tests.rs` -- expected rows and spans without the icon: open, folded with count, long name cut 2 later, widths 11 and 6 -- CAP-1
- [ ] `src/drag/tests.rs` -- time columns 11–15; no new lines -- clicks unchanged
- [ ] `tabrail-colour`, `tabrail-move` -- swatch and list without the icon -- CAP-2
- [ ] `tabrail-timer-test`, `tabrail-timer-click-test`, `tabrail-colour-test` -- group rows found by `^[▾▸] NAME`, rows and columns without the icon, swatch and Move to group text -- CAP-1, CAP-2
- [ ] `README.md`, `groups.conf.example` -- example rows, the icon column not shown -- behaviour reference

**Acceptance Criteria:**
- Given groups Website (`◆`) and Infra (`✚`) on a throwaway server, when a sidebar is captured, then `▾ Website`, `▾ Infra` and `▾ Default` start in column 1 and no `◆`, `✚` or `•` shows.
- Given Website's timer running, when its time is pressed and released, then it stops and the fold state is unchanged; a press and release on Infra's name folds Infra.
- Given a groups file written before this change, when tabrail reads it and New group… adds a line, then every group shows and the new line still has 4 columns.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `./tabrail-timer-click-test && ./tabrail-timer-test && ./tabrail-colour-test` -- expected: every check ok, on their own tmux -L servers
