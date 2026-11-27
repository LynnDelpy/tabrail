---
title: 'Groups without their icon'
type: 'feature'
created: '2026-11-26'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_revision: '84b7f146bd7e1c3d05dbb33ba26be382ca9e0982'
followup_review_recommended: false
deferred:
  - summary: >-
      Move to group and Move into group… lose their indent cue from row 10: tabrail-pick -n numbers rows with %d., one column wider, the same as one level of indent.
    evidence: |-
      Pre-existing: with the icon after the indent the shift was the same. %2d. or 2 columns per level fixes it.
    location: >-
      tabrail-pick
    severity: low
  - summary: >-
      Move to group lists Default twice when the groups file has a hand-written Default line.
    evidence: |-
      Pre-existing: tabrail-groups tree prints every line with 3 or more columns and tabrail-move appends Default after it; tabrail-nest filters Default, tabrail-move does not.
    location: >-
      tabrail-move:22
    severity: low
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
- [x] `src/head.rs` -- header `{indent}{▾|▸} {name}`, doc without the icon -- CAP-1
- [x] `src/head/tests.rs` -- expected rows and spans without the icon: open, folded with count, long name cut 2 later, widths 11 and 6, subgroup -- CAP-1
- [x] `src/drag/tests.rs` -- time columns 11–15; no new lines -- clicks unchanged
- [x] `tabrail-colour`, `tabrail-move` -- swatch and list without the icon, list indent kept -- CAP-2
- [x] `tabrail-tree-test`, `tabrail-timer-test`, `tabrail-timer-click-test`, `tabrail-colour-test` -- rows, columns, swatch and list text without the icon; group rows found by arrow and name -- CAP-1, CAP-2
- [x] `README.md`, `groups.conf.example` -- example rows, the icon column not shown -- behaviour reference

**Acceptance Criteria:**
- Given groups Website (`◆`) and Infra (`✚`) on a throwaway server, when a sidebar is captured, then `▾ Website`, `▾ Infra` and `▾ Default` start in column 1 and no `◆`, `✚` or `•` shows.
- Given egov with St.Gallen nested under it, when a sidebar is captured, then St.Gallen's header reads ` ▾ St.Gallen` with no icon.
- Given Website's timer running, when its time is pressed and released, then it stops and the fold state is unchanged; a press and release on Infra's name folds Infra.
- Given a groups file written before this change, when tabrail reads it and New group… adds a line, then every group shows and the new line still has its icon column.

## Implementation Notes

- Built on `lde/no-group-icon`, branched from `lde/nested-groups` (PR #45: stories 3-5, not on main yet) with origin/main merged in, so the PR stacks on #45.
- The matrix and Code Map predate story 3's `+` (rows end in ` ▷ + `, `▷` at width - 3 = 23, `+` at 25). Only the name side moves as specified: name +2 characters, time 2 columns left (Website 11-15, Archiv 12-15); the right side stays as it is on this branch. Long name: `▸ Abcdefghijklmn ▶ 2h0m + `, span (18, 23) (was `▸ ◆ Abcdefghijkl`).
- Story 4 was already built with depth + 5: `src/drag/group.rs` now nests from depth + 3; test release points "left of the name" moved to column 2.
- `tabrail-nest` (Move into group…, story 5) drops the icon too. `Group.icon` removed (unread field warning); groups.conf still needs 3 columns.
- `tabrail-tree-test` no longer resizes to 28: the folded count fits at 26 now.

## Spec Change Log

## Review Triage Log

### 2026-11-26 — Review pass
- verdicts: 23 findings — high 0, medium 1, low 14, false 8, maybe-false 0
- findings:
  - `[low]` `[reject]` (intent) Matrix rows 1, 2, 4 give main's right edge (`▷ ` at 25, span (21, 26)); the code and tests give this branch's `▷ + ` (▷ 23, span (18, 23)) — fix is editing this build's frozen matrix; the deviation is recorded under Implementation Notes, the name side matches literally.
  - `[low]` `[reject]` (intent) A header's "move among siblings" drop zone shrinks to depth + 2 columns — SPEC.md sets story 4's nest rule to depth + 3; changing it contradicts the intent.
  - `[false]` `[reject]` (intent) Integration tests do not check the right edge — tabrail-timer-click-test checks the ▷ at width - 3 and tabrail-subgroup-test the + at width - 1 and ▷ at width - 3, on real tmux.
  - `[false]` `[reject]` (intent) `Group.icon` no longer carried in the model — nothing read it; the file format is unchanged and lines under 3 columns are still skipped (Notes in src/groups/tests.rs).
  - `[false]` `[reject]` (intent) tabrail-colour-test does not re-check each swatch's tint — the diff only drops the icon argument; the RGB arguments are untouched and the picked tint is checked on every sidebar row.
  - `[false]` `[reject]` (intent) "Search on names as today" has no surface in Move to group — tabrail-move uses `tabrail-pick -n`, unchanged; no defect.
  - `[false]` `[reject]` (intent) tabrail-nest changed beyond the named surfaces — SPEC.md: "The icon goes from everywhere it is shown"; covered by tabrail-nest-test.
  - `[false]` `[reject]` (intent) Story frontmatter and notes in the diff — workflow metadata, not behaviour.
  - `[low]` `[reject]` (blind) Contract still describes the old layout, Spec Change Log empty — same root cause as the first row; fix edits this build's spec.
  - `[low]` `[reject]` (blind) Story incomplete for in-review (checkboxes, Verification list) — fix edits this build's spec; boxes ticked at finalize, runs of all 7 changed scripts recorded in Auto Run Result.
  - `[medium]` `[patch]` (blind) Docs call the icon "not shown" but not required; a hand edit dropping it loses the group or reads its first folder as the icon (`c.len() < 3`, `read -r name tint icon dirs`) — groups.conf.example and README now say "required … not shown".
  - `[low]` `[reject]` (blind) The "left of the name" drop zone shrank — same as the second row: SPEC.md's depth + 3.
  - `[low]` `[reject]` (blind) Name column hard-coded in src/drag/group.rs and src/drag/tests/tree.rs — bites only when head's format changes again; the fix is a new shared fn, more than a direct correction.
  - `[low]` `[patch]` (blind) Nest boundary untested at depth 2 — added `drop("4tw", header("Archiv"), 5)` nests in src/drag/tests/tree.rs.
  - `[low]` `[patch]` (blind) src/groups/tests.rs comment "the icon column skipped" does not match the tint-only assertion — comment now "tints from column 2, Default none".
  - `[low]` `[reject]` (blind) `tabrail-groups tree` still prints an icon nobody reads — harmless output pinned by tabrail-tree-test; dropping it changes two readers and tests for no visible gain.
  - `[low]` `[patch]` (blind) README reflow split "a picker of the 25 named colours", which spec-more-colours story 1 looks for — rewrapped, the phrase is on one line (README.md:20).
  - `[low]` `[patch]` (blind) README:42 "an icon no group uses yet" without "not shown"; line 17 ambiguous — "(not shown)" added; line 17 now names the groups file's icon column.
  - `[low]` `[defer]` (blind) Picker indent misaligns from row 10 (`tabrail-pick -n` prints `%d.`) — pre-existing: with the icon after the indent the shift was the same.
  - `[false]` `[reject]` (blind) tabrail-colour-test finds Website by `▾ Website` only — Website is never folded in that script, so the lookup always matches.
  - `[low]` `[defer]` (edge) Move to group lists Default twice with a hand-written Default line — pre-existing: `tree` prints any 3-column line and tabrail-move appended Default before this diff.
  - `[false]` `[reject]` (edge) The ▷ does not move left by 2 as the Approach says — the ▷ stays at width - 3, matching the matrix's "▷ as today"; click tests pass there.
  - `[low]` `[reject]` (edge) No width-6 head test — the cut to the arrow alone runs through the same trim line, covered by the width-12 timed case `▾ ▶ 1h0m  + `.

## Verification

**Commands:**
- `cargo build --release --offline && cargo test --release --offline` -- expected: builds, tests pass
- `./tabrail-tree-test && ./tabrail-timer-click-test && ./tabrail-timer-test && ./tabrail-colour-test` -- expected: every check ok, on their own tmux -L servers

## Auto Run Result

Status: done

**Summary:** Group headers read `{indent}{▾|▸} {name}`: no groups.conf icon, Default's `•` gone, the name gains 2 columns, the time and its click span move 2 left. Colour… swatches, Move to group and Move into group… show names without icons. groups.conf keeps its icon column (required, not shown); New group… still writes one. Drag-to-nest (story 4) now starts at the name, depth + 3.

**Files changed:**
- `src/head.rs` -- header without the icon.
- `src/groups.rs` -- `Group.icon` removed; 3 columns still required.
- `src/drag/group.rs` -- nest from depth + 3.
- `src/head/tests.rs`, `src/drag/tests.rs`, `src/drag/tests/plus.rs`, `src/drag/tests/tree.rs`, `src/groups/tests.rs` -- rows, spans and drop columns recomputed; depth-2 nest boundary added.
- `tabrail-colour`, `tabrail-move`, `tabrail-nest` -- swatch and picker lines without the icon, indent kept.
- `tabrail-tree-test`, `tabrail-timer-test`, `tabrail-timer-click-test`, `tabrail-colour-test`, `tabrail-nest-test`, `tabrail-subgroup-test`, `tabrail-tree-drag-test` -- group rows found by `[▾▸] NAME`, expected text without icons.
- `README.md`, `groups.conf.example` -- example rows without icons; the icon column required but not shown.

**Review:** 23 findings. Patched 5 (medium 1, low 4): docs say the icon column is required; README phrase kept on one line and "(not shown)" added; depth-2 nest boundary test; groups test comment. Deferred 2 (low, pre-existing): picker indent from row 10, Default listed twice with a hand-written Default line. Rejected 16: 8 false (see triage log for each refutation), 8 low (spec-frozen matrix and depth + 3 rule, spec edits, a shared-fn refactor, `tree`'s unread icon, the width-6 case covered by the same trim line).

**Follow-up review recommended:** false (patched: high 0, medium 1, low 4).

**Verification:**
- `cargo build --release --offline`: no warnings; `cargo test --release --offline`: 39 passed (after patches too).
- `./tabrail-tree-test` 18 ok, `./tabrail-timer-click-test` 19 ok, `./tabrail-timer-test` 22 ok, `./tabrail-colour-test` 22 ok, run one after another after the patches; before them also `tabrail-nest-test`, `tabrail-subgroup-test`, `tabrail-tree-drag-test` all ok.
- `~/Dev/vmConfig/tabrail-groups.conf` byte-identical before and after every run.

**Residual risks:**
- Stacks on PR #45 (lde/nested-groups); merge #45 first.
- The frozen matrix's right-edge numbers (▷ at 25, long-name span (21, 26)) describe main without the `+`; the code follows this branch's layout.
- The header's "move among siblings" drop zone is now only the indent, arrow and space (depth + 2 columns).
