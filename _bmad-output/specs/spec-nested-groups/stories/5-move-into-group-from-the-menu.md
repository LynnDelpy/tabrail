---
title: 'Move into group from the menu'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'a11d87d8b6647eb6b9bb573bf47eb55226dcc3ac'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-nested-groups/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A group cannot be taken out of its parent with the mouse at all, and nesting needs an aimed drag.

**Approach:** A group's menu gets Move into group…, a picker popup like Move to group…. It lists Top level first, then every group except this one and its subtree, in tree order, indented, with the current parent marked `· current`. A pick runs story 4's nest, and Top level moves the tree to the end of the top level, above Default.

## Boundaries & Constraints

**Always:** Esc changes nothing. Picking the current parent makes the group its last subgroup; Top level for a top-level group moves it to the end of the top level. The groups file is written by `tabrail-groups nest`, nothing else. The popup is no taller than the client; fzf scrolls the rest.

**Never:** The item on Default. Offering Default, the group itself or its descendants.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nest | Köniz → St.Gallen | Köniz's tree is St.Gallen's last subgroup | N/A |
| Out | St.Gallen → Top level | St.Gallen's tree at depth 0 after the last top-level tree | N/A |
| List | menu on egov | Top level, 4tw, …; not egov, St.Gallen, Archiv, Default | N/A |
| Current | menu on St.Gallen | egov's row `· current` | N/A |
| Cancel | Esc | `cmp` identical | N/A |
| Gone | group renamed while the picker is open | file identical | nest finds no line, exit 0 |

</frozen-after-approval>

## Code Map

- `tabrail-move` -- the model: `WINDOW CLIENT` builds a list in a mktemp file and opens `display-popup -c CLIENT -B -E -w 60 -h ROWS+4`; `--pick` runs `tabrail-pick` and acts. Its indented tree list (story 1) can be reused.
- `tabrail-pick` -- prints the value column; an empty value reads as Esc, so Top level needs a value no group name can be (e.g. `/`).
- `src/menu.rs` group branch and `Place` -- add the new script's path; `src/main.rs:29-31` builds the paths.
- `tabrail-groups nest SRC PARENT` (story 4, awk in `tabrail-groups-move`) -- PARENT empty means top level. As built, nest refuses an empty PARENT (`!b`) and move-group Default refuses a nested SRC (`par[a]`); the awk's top-level path (`p = 0; at = last; d = 0`) is the one to open for an empty PARENT. `src/menu.rs` and `src/main.rs` are at 99 lines.
- `README.md` mouse bullet's menu list and Files table.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-nest` -- new, `GROUP CLIENT` opens the picker, `--pick GROUP LIST` runs `tabrail-groups nest` -- one script per job, like tabrail-move
- [x] `src/menu.rs`, `src/main.rs` -- `'Move into group…' i` after Rename group, not for Default, name through `clean()` -- the entry point
- [x] `tabrail-tree-test` -- the matrix rows (drive the popup with `send-keys` on the outer server) -- regression
- [x] `README.md` -- Move into group… in the menu list, `tabrail-nest` in Files -- behaviour reference

Implementation notes: `tabrail-groups nest SRC ''` (awk in `tabrail-groups-move`) now takes the top-level path: SRC's tree at depth 0 after the last group line outside it (for a nested SRC whose tree holds the file's last group line, the last one before it); a top-level SRC whose tree holds the last group line exits 1, file untouched. The dispatcher refuses `nest` without its PARENT argument (usage, exit 2), so only an explicit `''` means Top level. `tabrail-nest` builds its rows from `tabrail-groups tree` in one awk (Top level value `/`, the group's tree skipped by depth, Default dropped by name, the parent from the last name seen one level up), opens `tabrail-pick -n` like tabrail-move, sized as tabrail-colour (rows + 4, width 60, both clamped to the client); a group without a line opens no popup. The item is guarded like New subgroup… and Colour… (`name == *raw`): a name `clean()` changes would key another group's line. The group row's items moved verbatim into `src/menu/group.rs` (split by job, as drag/group.rs), the new item beside Colour… in its style; menu.rs is at 90. `main.rs` borrows the paths with `each_ref().map(String::as_str)` so `Place` uses field shorthand and stays at 99. The matrix went into a new `tabrail-nest-test`, as `tabrail-tree-test` (87 lines) cannot take it under 100 (stories 2-4 did the same); it also checks the current mark at depth 2, a second session's sidebar, a top-level group already last, `nest` without PARENT, a last top-level tree of two lines, a nested last tree out, a hand-written Default line, a group without a line (under `timeout 5`) and a 50x10 client (the popup opened with `run-shell -b`: `tmux display-popup` from a shell waits for the popup to close; tabrail-pick's footer is cut at that width). `tabrail-colour-test` pins no Move into group… for a name `clean()` changes. `groups.conf.example` names Move into group… beside the drag text.

**Acceptance Criteria:**
- Given a right-click on Default's header, then its menu has no Move into group….
- Given St.Gallen moved to Top level, when the sidebars redraw, then St.Gallen's header starts at column 1 and egov's tree no longer holds it.

## Review Triage Log

| # | Layer | Location | Claim | Verdict | Route | Evidence |
|---|-------|----------|-------|---------|-------|----------|
| 1 | blind | `tabrail-nest` listing | `$2 == g` compares `1` and `01` as numbers | low | patch | Every other awk here uses `g ""`; direct fix |
| 2 | blind, edge-case | `tabrail-groups` nest | `nest SRC` with PARENT left out by hand moves the tree to the top level | low | patch | Unset reads as empty; before this story it refused. Arg-count check |
| 3 | blind | `src/menu.rs` | Unrelated lines squashed to stay under 100 | medium | patch | Against "match surrounding style, do not refactor uninvited"; split the group items into `src/menu/group.rs` |
| 4 | blind | `tabrail-nest-test` `tn Nope` | Would hang instead of failing if the no-line guard regressed | low | patch | display-popup from a shell waits; `timeout 5` |
| 5 | blind | `tabrail-nest-test` Gone | Leaves kz in a group with no line for later checks | low | patch | Order dependence; undo the rename |
| 6 | verification-gap, blind | `tabrail-nest:25` | `· current` only checked at depth 0/1 | medium | patch | Pre-verified: `up[0]` passes |
| 7 | verification-gap, blind | `tabrail-groups-move:34-35` | Top level tested only after a one-line last tree; nested SRC holding the last line untested | medium | patch | Pre-verified: a last-top-level-line mutant reparents Sub and passes |
| 8 | verification-gap | `tabrail-nest:40` | Width clamp never exercised | medium | patch | Pre-verified: `-w 60` on a 40-column client fails, no check notices |
| 9 | verification-gap | `tabrail-nest:32` | "not Default" filter never runs | low | patch | Pre-verified; one fixture variant |
| 10 | verification-gap, blind | `src/menu.rs:84` | Move into group… hidden for a name `clean()` changes, untested | medium | patch | Pre-verified: moving it out of the guard passes every check |
| 11 | blind | `groups.conf.example`, README | Docs overstate targets; first bullet names only drags; nest-test row overclaims | low | patch | Direct text fixes |
| 12 | blind, edge-case | `tabrail-groups-move:35` | Top level lands below a hand-written `Default` line | low | reject | Hand-written Default lines; story 1 row 2 |
| 13 | blind, edge-case | `tabrail-nest` | A group named `/` reads as Top level | low | reject | Code Map's sentinel; no realistic name |
| 14 | blind | `tabrail-nest` `-n` | No search: past 9 rows only arrows reach a group | low | reject | Approach: a picker like Move to group…, which uses `-n` |
| 15 | edge-case | `src/menu.rs:84` | One more item: a client just tall enough before gets no menu | low | reject | tmux drops a menu taller than the client; every item added does this |
| 16 | edge-case | `tabrail-nest:31-32` | Duplicate GROUP name or a Default line with a subtree offer bad targets | low | reject | Hand-written duplicates; story 1 row 2 |

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-nest && wc -l tabrail-nest src/menu.rs` -- expected: syntax ok, under 100
- `./tabrail-tree-test` -- expected: every line `ok`
- `./tabrail-nest-test` -- expected: every line `ok`
