---
title: 'Show nested groups'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
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
- [ ] `src/groups.rs` -- new: `Group { name, tint, icon, depth }` from the parent rule, in file order, Default last at depth 0; subtree helper (index range of a group's descendants) for draw now and drag later -- one reader of the tree
- [ ] `src/model.rs`, `src/main.rs` -- drop `groups()`, `mod groups`, call it -- room in model.rs
- [ ] `src/draw.rs`, new `src/row.rs` -- tab-row formatting into `row.rs` with an indent argument; `screen()` walks the tree: hidden below a folded ancestor, skipped without subtree tabs, indented header, subtree count when folded, blank line after top-level trees -- the tree on screen
- [ ] `tabrail-groups` -- new, the groups-file writers moved from tabrail-act (`rename-group`, `new-group`, `move-group`, TINTS, ICONS); rename keeps leading whitespace -- tabrail-act has no room left, one job per script
- [ ] `tabrail-act`, `tabrail-name`, `src/main.rs`, `src/drag.rs` -- callers use tabrail-groups; tabrail-act keeps `new`, `dup`, `fold` -- no forwarding stubs
- [ ] `tabrail-move` -- the list in tree order, each label indented one space per level -- the picker shows the tree
- [ ] `tabrail-tree-test` -- new, modelled on `tabrail-reader-test` (own `-L` servers, temp groups copy via `@tabrail_groups`): the I/O matrix rows -- later stories add their checks
- [ ] `README.md`, `groups.conf.example` -- indentation nests groups, a nested example, tabrail-groups and tabrail-tree-test in Files -- README is the behaviour reference

**Acceptance Criteria:**
- Given a nested groups file, when Move to group… opens on a tab, then St.Gallen is listed under egov, indented.
- Given a tab moved into Archiv, when every sidebar redraws, then each shows it under Archiv's header at its depth.

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
