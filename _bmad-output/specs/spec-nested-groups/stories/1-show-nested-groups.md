---
title: 'Show nested groups'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'a3ee8b6a2d880efb48a0e8e487a86ae1de6a6c8f'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-nested-groups/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Group lines are read flat, so related projects (egov and its customers) cannot be shown as one tree.

**Approach:** A group's parent is the nearest group line above it with less leading whitespace. Every sidebar draws the tree: headers and tab rows one column further right per level, a parent's own tabs above its subgroups, a folded parent hiding its whole subtree. The bash readers and writers keep indentation.

## Boundaries & Constraints

**Always:** A flat groups file draws exactly as before. A group shows only if it or a descendant holds a tab. A folded header counts every tab in its subtree; subgroups keep their own entry in `@sidebar_fold`. One blank line after each top-level tree (Default included), none inside one. Leading spaces and tabs count one each; an indented line with no shallower group line above is top-level. Rename changes only the name: leading whitespace and column spacing stay. Moved writers keep their safe write (symlink, mode, rename into place). Files under 100 lines, standard library only.

**Never:** Change folder matching (story 2), add menu items or the `+` (stories 3, 5), change move-group (story 4), nest Default.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Flat | today's `groups.conf.example` | `capture-pane` identical to a build from main | N/A |
| Nested | `egov`, `  St.Gallen`, `    Archiv`, each with a tab | headers start at columns 1, 2, 3; each tab row shifted by its group's depth; one blank line after Archiv's tab | N/A |
| Parent without tabs | egov none, St.Gallen one | egov's header, then St.Gallen's | N/A |
| Empty leaf | Archiv no tabs | no Archiv header | N/A |
| Fold parent | egov folded | `▸ ★ egov  3 tabs`, then the blank line | N/A |
| Unfold | St.Gallen folded, then egov folded and unfolded | St.Gallen still `▸` | N/A |
| Odd indent | first group line indented | top-level | N/A |
| Rename | rename `  St.Gallen` to `SG` | `diff` shows only the name changed, indent kept | N/A |

</frozen-after-approval>

## Code Map

- `src/model.rs:50` `groups()` -- returns `(name, tint, icon)`; model.rs is at 97 lines, so it moves out.
- `src/draw.rs:63` `screen()` -- one header plus tabs per group, a blank line after each; at 96 lines. The tab-row block (lines 76-90) moves out. `rows` must stay in step with `lines`: clicks map rows.
- `src/main.rs:73` -- calls `model::groups`; passes `&helper` (tabrail-act) to `drag::left`, which uses it only for move-group.
- `tabrail-act` -- 99 lines. `rename_group` awk assumes the line starts with the name. `new_group`, `move_group`, `TINTS`, `ICONS` are groups-file writers.
- `tabrail-name:21-24` -- calls `tabrail-act rename-group` and `new-group`.
- `tabrail-move:23` `groups()` -- `awk 'NF >= 3'` flat list for the picker.
- `README.md` groups bullet and Files table; `groups.conf.example` header comment.

## Tasks & Acceptance

**Execution:**
- [x] `src/groups.rs` -- new: `Group { name, tint, icon, depth }` from the parent rule, in file order, Default last at depth 0; subtree helper (index range of a group's descendants) for draw now and drag later -- one reader of the tree
- [x] `src/model.rs`, `src/main.rs` -- drop `groups()`, `mod groups`, call it -- room in model.rs
- [x] `src/draw.rs`, new `src/row.rs` -- tab-row formatting into `row.rs` with an indent argument; `screen()` walks the tree: hidden below a folded ancestor, skipped without subtree tabs, indented header, subtree count when folded, blank line after top-level trees -- the tree on screen
- [x] `tabrail-groups` -- new, the groups-file writers moved from tabrail-act (`rename-group`, `new-group`, `move-group`, TINTS, ICONS); rename keeps leading whitespace -- tabrail-act has no room left, one job per script
- [x] `tabrail-act`, `tabrail-name`, `src/main.rs`, `src/drag.rs` -- callers use tabrail-groups; tabrail-act keeps `new`, `dup`, `fold` -- no forwarding stubs
- [x] `tabrail-move` -- the list in tree order, each label indented one space per level -- the picker shows the tree
- [x] `tabrail-tree-test` -- new, modelled on `tabrail-reader-test` (own `-L` servers, temp groups copy via `@tabrail_groups`): the I/O matrix rows -- later stories add their checks
- [x] `README.md`, `groups.conf.example` -- indentation nests groups, a nested example, tabrail-groups and tabrail-tree-test in Files -- README is the behaviour reference

**Acceptance Criteria:**
- Given a nested groups file, when Move to group… opens on a tab, then St.Gallen is listed under egov, indented.
- Given a tab moved into Archiv, when every sidebar redraws, then each shows it under Archiv's header at its depth.

## Review Triage Log

| # | Layer | Location | Claim | Verdict | Route | Evidence |
|---|-------|----------|-------|---------|-------|----------|
| 1 | blind, edge-case, verification-gap | `src/groups.rs:19`, `tabrail-groups:22` | An indented comment (`  # customers of egov`) becomes a group `#` that adopts the deeper lines below it | medium | patch | Only column-0 `#` is skipped; replayed with the `tree` awk (`1 # of`, St.Gallen at depth 2). tabrail-tmux's group() already skips it after `read`, so readers disagree; indentation now invites indented comments |
| 2 | blind, edge-case | `src/groups.rs:28`, `tabrail-groups:32` | A name on two lines (or a `Default` line) draws its tabs under both headers; rename rewrites both | low | reject | Hand-written duplicates behaved so in a flat file before this change; writers refuse Default and new-group refuses an existing name. Fix is a dedup guard |
| 3 | blind, edge-case | `tabrail-groups:31-41` | A failed rename write still sets the tabs' `@group` to NEW, which has no line, so they vanish | medium | defer | The `@group` and fold loops ran after a failed write before this change too (old `> $CONF.new && mv`); model.rs keeps an unknown `@group` and draw.rs draws only known groups |
| 4 | blind | `tabrail-groups:32` | Rename has no `NF >= 3` check and does not refuse a NEW that exists | low | reject | Same as the old awk; a non-group line starting with a group's name is unlikely. Fix adds guards |
| 5 | blind, edge-case | `src/drag.rs:53-67` | Dragging a group in a nested file moves one line and re-parents its subgroups | low | reject | Frozen Never: move-group unchanged until story 4 (same branch, merged only with all stories) |
| 6 | blind | `src/groups.rs`, `tabrail-groups tree` | The parent rule lives in Rust and awk with no cross-check | low | reject | Both implement the same stack rule; no divergence shown. Fix is more tests |
| 7 | blind, edge-case | `src/row.rs:13-15`, `src/head.rs:26` | Depth near the width cuts the label to spaces, so rows lose their name | low | reject | Needs ~20 levels at width 26; fix is a cap |
| 8 | blind | `src/draw.rs:68` | A folded parent hides a running subgroup timer | false | reject | CAP-2: folding a parent hides its whole tree |
| 9 | blind | `tabrail-tree-test:31` | Flat check compares text, not `capture-pane -e` against a build of main | low | reject | depth 0 runs the same code paths (empty indent); implementer compared `-ep` byte for byte once; fix is a build-of-main harness |
| 10 | blind | `tabrail-colour` | No test that Colour… keeps an indented line's whitespace | low | reject | Its `match($0, /^[ \t]*[^ \t]+[ \t]+/)` keeps the indent; not a writer this story changes |
| 11 | blind | `src/main.rs`, `src/menu.rs` | Files at 99/98 lines after squashing `use` lines leave no room for stories 3-5 | low | reject | No defect now; the next stories split by job as AGENTS.md says |
| 12 | blind | `README.md` groups bullet | Says a group shows while it or a subgroup holds a tab, leaving out the timer case | low | patch | draw.rs keeps a subtree with a timer; direct text correction |
| 13 | blind | `tabrail-act:2`, README Files | "what the menus do with tabs" but `fold` is a group action | low | patch | Direct text correction |
| 14 | blind | `README.md` "none above" | Should say "no shallower group line above" | false | reject | "none" refers to the group line with less whitespace named just before |
| 15 | blind | `src/head.rs` | The tint covers the indent, so depth shows only as a shifted arrow | false | reject | Design Notes: the tint fills the row as today |
| 16 | verification-gap | `src/draw.rs:68` | No test: a tab-less subgroup kept by its timer under a tab-less parent | medium | patch | Pre-verified: narrowing the rule to the group's own timer passes every test |
| 17 | verification-gap | `src/row.rs:13` | Indent in the program-drop and cut math untested with a program and status | low | patch | Pre-verified: dropping `indent +` passes every test |
| 18 | verification-gap | `tabrail-name:22` | Rename group via the menu is driven by no test | medium | patch | Pre-verified: a stale `tabrail-act rename-group` call passes every test |
| 19 | verification-gap | story Verification | `cargo test`, timer and colour tests not in Verification | low | reject | Fix edits this build's spec; all three were run and pass (26, 22, 22 ok) |
| 20 | verification-gap | `deferred-work.md` DW-16 | Ledger still lists the symlink rename bug as open | low | reject | Ledger is append-only here; the sweep resolves it |

## Design Notes

Header: `"{indent}{▾|▸} {icon} {name}"` with `indent = " ".repeat(depth)`; the tint fills the row as today. Tab row: the same indent before `" {❯| } {name}"`; the cut logic counts the indent as part of the label. At 26 columns:

```
▾ ★ egov
 ❯ gever-core      ✳ ◇
 ▾ ✚ St.Gallen
    stgallen·main  ⏱ 4m
  ▾ ◆ Archiv
     archiv·dev
 ▸ ✚ Köniz  3 tabs

▾ ■ 4tw
```

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-groups tabrail-act tabrail-name tabrail-move tabrail-tree-test && wc -l src/*.rs tabrail-*` -- expected: syntax ok, every file under 100
- `./tabrail-tree-test` -- expected: every line `ok`, exit 0
- `./tabrail-reader-test` -- expected: still all `ok`
