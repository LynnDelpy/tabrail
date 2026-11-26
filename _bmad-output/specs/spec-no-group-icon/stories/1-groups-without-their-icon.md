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

**Problem:** Every group row shows the groups.conf icon (`✚`, `■`, `★`, `◆` …, Default's `•`) between the fold arrow and the name, and the Colour… and Move to group pickers show it too. Lynn wants it gone (2026-11-26).

**Approach:** head.rs builds the header as `{indent}{▾|▸} {name}`; the swatch in tabrail-colour and the line in tabrail-move drop the icon. The time, count, `▷` and their click spans come from the head's length, so they move left by 2 by themselves. groups.conf keeps the column; tests and docs follow.

## Boundaries & Constraints

**Always:** Arrow in column depth + 1, name from depth + 3, open or folded, Default too. Tint, bold, the time's colour, the `▷`, the folded count and the cut order (count first, then name) as today. A press and release on the row folds; on the time or `▷` it toggles the timer at their new columns. Move to group keeps one column of indent per level. groups.conf read as today (icon in column 3); New group… still writes an unused icon. Tab rows unchanged. Files under 100 lines (`src/drag/tests.rs` is at 100: change lines, add none).

**Never:** Removing the fold arrow; changes to `❯` or tab rows; dropping the icon column or rewriting anyone's groups file; a replacement mark; testing on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open, no timer | Website `◆`, width 26 | `▾ Website` + 15 spaces + `▷ `; `▷` span (25, 25) | N/A |
| Folded, no timer | Infra `✚`, 3 tabs, width 26 | `▸ Infra  3 tabs` + 9 spaces + `▷ ` | N/A |
| Running | Website `▶ 12m`, width 26 | `▾ Website ▶ 12m`, time span (11, 15), open or folded | N/A |
| Long name | `Abcdefghijklmnopqrstuvwxyz0123` folded, `▶ 2h0m`, width 26 | `▸ Abcdefghijklmnopq ▶ 2h0m`, span (21, 26) | N/A |
| Subgroup | Archiv at depth 2, folded, `▶ 5s` | `  ▸ Archiv ▶ 5s`, time span (12, 15) | N/A |
| Default | tabs in no group | `▾ Default`, no `•` | N/A |
| Colour… | Website's menu, Colour… | swatches ` Website ` on each tint, `· current` mark as today | N/A |
| Move to group | egov, St.Gallen under it, a tab in egov | `egov  1 tab · current`, ` St.Gallen  1 tab`, `Default  …`; no icons; search on names as today | N/A |
| Old file | groups.conf lines with icons, flat or indented | read as today, icons ignored | N/A |

</frozen-after-approval>

## Code Map

Lines as on main `93b89e4` (nested-groups stories 1-2 merged).

- `src/head.rs:1,18,26` -- module doc "fold arrow, icon and name"; `row()` destructures `Group { name, tint, icon, depth }`; line 26 `format!("{}{} {icon} {name}", indent, arrow)` is the change. Dropping `icon` from the pattern (`..`) spares every caller.
- `src/groups.rs:12,29,32` -- `Group.icon`, read from column 3; Default's `"•"`. The field may stay as data; no format change.
- `src/head/tests.rs:26-97` -- expected rows start `▾ ◆ `/`▸ ◆ `; spans (`:64-66`), narrow cases (`:41,75,76`: widths 11 and 6) and the subgroup test (`:95-97`, time 14-17 → 12-15) need recomputing, the name now gets 2 more characters.
- `src/drag/tests.rs:15,39-60` -- time columns 13–17 and click pairs `(13, 13)…(17, 13)`, `14` become 11–15; `(25, 25)` for the `▷` stays.
- `src/drag.rs`, `src/draw.rs`, `src/row.rs` -- no change: `Row::Group` carries head.rs's span.
- `tabrail-colour:2,32,40` -- header comment, `read -r tint icon`, swatch `' %s %s '` with `"$icon" "$group"`: swatch ` $group `.
- `tabrail-move:20-28` -- `tabrail-groups tree` gives depth, name, icon; Default `'0\tDefault\t•'`; line `'%s\t%*s%s %s'`: keep the `%*s` indent, drop the icon. tabrail-pick searches the last shown field, unchanged.
- `tabrail-groups:4,11,75-81` -- `tree` may keep printing the icon; New group… keeps picking an unused one. No change needed.
- Tests: `tabrail-tree-test` (rows `▾ ★ egov`, ` ▾ ✚ St.Gallen`, `▾ • Default`, Move to group `1. ★ egov  1 tab · current`…), `tabrail-timer-test` (`"◆ $W"`, `"■ Q'a"`, `"▾ ◆ $W ▶ "`, `"▸ ▲ Ops ▶ "`, `"▾ ● Abcdefghijklmno ▶ 2h0m"` gains 2 chars), `tabrail-timer-click-test` (`c == 13` → 11; `clk 5` still lands on the name), `tabrail-colour-test` (`"▾ ◆ Website"`, `"• Default"`, swatch `"│ [ ❯]  ◆ Website "`). Find group rows by arrow and name (`[▾▸] NAME`) so tab rows do not match. `tabrail-folder-test` only writes icons into files: unchanged.
- `README.md:7-11,17,19,195` -- example group rows (`▾ Website` + 15 spaces + `▷`, annotations at column 36), "a faint tint and an icon", the swatch "with the group's icon and name"; the Files row for `tabrail-groups` says `tree` gives the icon "for the pickers".
- `groups.conf.example:7` -- column list: icon kept, not shown.
- spec-nested-groups story 4 (ready-for-dev) -- "name starts at column `depth + 5`": depth + 3 once this is merged.

## Tasks & Acceptance

**Execution:**
- [ ] `src/head.rs` -- header `{indent}{▾|▸} {name}`, doc without the icon -- CAP-1
- [ ] `src/head/tests.rs` -- expected rows and spans without the icon: open, folded with count, long name cut 2 later, widths 11 and 6, subgroup -- CAP-1
- [ ] `src/drag/tests.rs` -- time columns 11–15; no new lines -- clicks unchanged
- [ ] `tabrail-colour`, `tabrail-move` -- swatch and list without the icon, list indent kept -- CAP-2
- [ ] `tabrail-tree-test`, `tabrail-timer-test`, `tabrail-timer-click-test`, `tabrail-colour-test` -- rows, columns, swatch and list text without the icon; group rows found by arrow and name -- CAP-1, CAP-2
- [ ] `README.md`, `groups.conf.example` -- example rows, the icon column not shown -- behaviour reference

**Acceptance Criteria:**
- Given groups Website (`◆`) and Infra (`✚`) on a throwaway server, when a sidebar is captured, then `▾ Website`, `▾ Infra` and `▾ Default` start in column 1 and no `◆`, `✚` or `•` shows.
- Given egov with St.Gallen nested under it, when a sidebar is captured, then St.Gallen's header reads ` ▾ St.Gallen` with no icon.
- Given Website's timer running, when its time is pressed and released, then it stops and the fold state is unchanged; a press and release on Infra's name folds Infra.
- Given a groups file written before this change, when tabrail reads it and New group… adds a line, then every group shows and the new line still has its icon column.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `./tabrail-tree-test && ./tabrail-timer-click-test && ./tabrail-timer-test && ./tabrail-colour-test` -- expected: every check ok, on their own tmux -L servers
