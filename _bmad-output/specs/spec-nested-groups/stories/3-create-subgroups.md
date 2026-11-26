---
title: 'Create subgroups'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'd0d798e987af127f53a7c699c67e47260607f530'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-nested-groups/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The only way to make a subgroup is to edit the groups file by hand.

**Approach:** A group's menu gets New subgroup…, and every header but Default's gets a dim `+` at its right edge whose left click does the same. Both open the name field. Enter adds the name's line right after the parent's subtree, one level deeper (2 spaces per level), with an unused tint and icon as New group… picks, and opens a new tab in it.

## Boundaries & Constraints

**Always:** Esc, or an empty or blank name: groups file identical, no tab. A name that exists anywhere adds no line and opens the new tab in that group. Names: one word (spaces become `-`), never Default. Exactly one line added, written safely (symlink, mode, rename into place). The `+` acts only when press and release are both on its column of the same header. A click elsewhere on the header still folds, and a press on `+` released on another row is a group drag. A long header name is cut so the `+` stays.

**Never:** A `+` or New subgroup… on Default. New subgroup… in a tab's or the empty-space menu. Changing New group… (top-level).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Menu | right-click egov, New subgroup…, `Bern`, Enter | `  Bern` line (unused tint, icon) after egov's subtree; a new tab in Bern, shown under egov | N/A |
| `+` | left click on the last column of egov's header | same name field, same result | N/A |
| Deeper | New subgroup… on St.Gallen (depth 1) | line indented 4 spaces after St.Gallen's subtree | N/A |
| Cancel | Esc | `cmp` identical, no new tab | N/A |
| Exists | `Köniz`, already a group | no line; new tab in Köniz | N/A |
| Default | right-click Default's header | no New subgroup…; no `+` drawn | N/A |
| Header click | press and release on egov's name | folds as today | N/A |

</frozen-after-approval>

## Code Map

- `src/menu.rs` `Some(Row::Group(name))` branch -- items `new_tab`, `new_group`, Rename group (not Default), Fold; `namer` closure builds the `tabrail-name` call; `clean()` for names in tmux strings.
- `tabrail-name` -- kinds `tab`, `group`, `new`; `--ask KIND TARGET NAME` keeps the typed name out of command strings. A subgroup needs the parent and the sidebar's pane (`pID`): separate arguments.
- `tabrail-groups` `new-group NAME TARGET` (moved in story 1) -- auto-pick of tint and icon, `p*` target opens a tab via `tabrail-act new`. Extend it with an optional PARENT rather than a second auto-pick.
- `src/input.rs:19` -- parses `n[1]` (column) but drops it; `src/control.rs:11` `Click { right, release, row }`.
- `src/drag.rs:30` -- `Some((from, Row::Group(src))) if from == click.row => fold`: the `+` check goes here; needs the sidebar width.
- `src/draw.rs` header line (story 1) -- the `+` goes in the last column, dim (`fg("dim")`).
- `tabrail.tmux:13-14` -- `MouseUp1Pane` / `MouseDragEnd1Pane` pass releases to the sidebar.
- If `spec-group-colour` has landed, the auto-pick reads its `palette` file instead of `TINTS`.
- If `spec-group-timer` has landed, `Click.col` exists and the header carries a time after the name (a click on it toggles the timer): keep the `+` at the right edge, cut the name first, never the time.

## Tasks & Acceptance

**Execution:**
- [x] `src/control.rs`, `src/input.rs` -- `Click.col` (1-based) -- the `+` is a column (already there since spec-group-timer: check, do not redo)
- [x] `src/head.rs` (and `head/tests.rs`, the tests pinning the ▷'s column) -- dim `+` at column `width - 1` of every non-Default header, the pad space after it kept; the ▷ moves to `width - 3`, a space between, in every header (Default's too, its `+` column blank); the name cut first, never the time, the ▷ or the `+`; the `+` column returned for drag.rs -- the click target
- [x] `src/drag.rs`, `src/main.rs` -- press and release on the `+` column of one header: open the name field for a subgroup of that group; else as before -- CAP-4 by click
- [x] `src/menu.rs` -- `'New subgroup…' s` after New group… in the group menu, not for Default -- CAP-4 by menu
- [x] `tabrail-name` -- kind `sub` (title `New subgroup of GROUP`) -- the name field
- [x] `tabrail-groups` -- `new-group NAME TARGET [PARENT]`: with PARENT, the line goes after PARENT's subtree at its depth + 1; a missing or Default PARENT adds nothing -- one auto-pick
- [x] `tabrail-tree-test` -- the matrix rows through SGR clicks on the `-L` outer server -- regression
- [x] `README.md` -- `+` and New subgroup… in the groups and mouse text -- behaviour reference

Implementation notes: `Click.col` was already there (no change). The popup's client: tabrail-name, given no CLIENT, looks up the non-control client showing PANE's window (the last active if several); bindings unchanged. `new_group` moved into a sourced `tabrail-groups-new` (tabrail-groups was at 98 lines); a subgroup's indent is 2 per level, clamped into (parent's indent, last child's indent] so a hand-indented file cannot re-parent it. The menus pass the sidebar's pane to tabrail-name for every kind (5th argument). The matrix rows went into a new `tabrail-subgroup-test`, as `tabrail-tree-test` (86 lines) cannot take them under 100 (story 2 did the same with `tabrail-folder-test`); `tabrail-tree-test`'s `bar()` drops the `+` with the `▷`. The + drag tests are in `src/drag/tests/plus.rs` (drag/tests.rs was at 100).

**Acceptance Criteria:**
- Given two sessions on the test server, when a subgroup is created from session A's sidebar, then session B's sidebars show it on their next redraw, without a restart.

## Review Triage Log

| # | Layer | Location | Claim | Verdict | Route | Evidence |
|---|-------|----------|-------|---------|-------|----------|
| 1 | blind | `src/head.rs:26-29` | ` ▷ + ` costs 2 more columns, so a folded header's count drops out more often at width 26 | low | reject | Follows from the layout chosen 2026-11-26 (Design Notes); head/tests.rs pins the count/cut order. Raised to the user |
| 2 | blind | `tabrail-act new` | `+`/New subgroup… on a folded group opens a tab hidden under the fold | medium | defer | New tab on a folded group behaved so before; nothing touches `@sidebar_fold` |
| 3 | blind, edge-case | `tabrail-name:34-39` | No matching client: `display-popup -c ""` fails into the pane | low | patch | Tab switched before the job runs; one `|| exit 0` |
| 4 | blind, edge-case, verification-gap | `tabrail-name:37-38` | With two terminals on one window the popup's terminal is untested and ties by the second | low | defer | Pre-verified gap; one client per window in the test; budget of the test file |
| 5 | blind, edge-case | `tabrail-groups-new` header | "missing PARENT adds nothing at all", but an existing NAME still opens its tab | low | patch | Comment vs code; behaviour matches "a name that exists opens the tab in that group" |
| 6 | blind, edge-case | `tabrail-groups-new:17` | A parent gone while the field was open: Enter silently does nothing | low | reject | Rare; fix adds a message branch |
| 7 | blind, edge-case | `tabrail-groups-new:10-17` | A `#name` writes a comment line; `clean()` characters give a dead-end group | low | defer | New group… behaved so before; name validation in `new_group` |
| 8 | verification-gap, blind | `tabrail-groups-new` `sub_line` | The `x <= at` clamp is never reached (4-space files) | medium | patch | Pre-verified: deleting it makes the subgroup a sibling, every test passes |
| 9 | verification-gap, blind | `tabrail-groups-new:11-17` | Missing or Default PARENT untested | medium | patch | Pre-verified: dropping `|| return 0` opens a stray tab, every test passes |
| 10 | verification-gap, blind | `src/menu.rs:82` | No check that New subgroup… is left out for a name `clean()` changes | low | patch | Pre-verified: dropping the guard passes both `Q'a` menu checks |
| 11 | blind, edge-case | `src/drag.rs:37-38` | Toggle and fold arms ignore a release row that now holds another group | low | defer | Arms predate this story; only the `+` arm is new |
| 12 | blind, verification-gap | story Verification, Tasks | Verification lists only tree-test; tasks tick tree-test and main.rs | low | reject | Fix edits this build's spec; all seven suites and `cargo test` were run and pass |
| 13 | blind | README groups text | "after its last subgroup's line" wrong when that subgroup has children | low | patch | sub_line inserts after the subtree's last group line |
| 14 | blind | `src/head/tests.rs:13`, `src/head.rs:22` | Escape stripping compressed into a tuple; `clean` shadows the function | low | reject | Readability only; head/tests.rs is at 97 lines |
| 15 | blind | `tabrail-subgroup-test:25`, `tabrail-tree-test` | Raw U+EB2C in `nf=$'…'` | low | patch | Direct correction to `$'\ueb2c'` |
| 16 | blind, edge-case | groups-file writers | No lock: a write between read and `mv` is lost | low | reject | Same as spec-group-drag R19 |
| 17 | edge-case | `src/head.rs:29-44` | Width under 9 with a timer overflows the pane | low | reject | A sidebar that narrow shows nothing usable; spec-group-timer-start row 3 |
| 18 | edge-case | `tabrail-groups-new:12` | `$1 == g` compares `01` and `1` as numbers | low | patch | Direct correction to `g ""`, as sub_line |
| 19 | edge-case | `tabrail-groups-new:14-15` | With every icon or tint used, the last is reused | low | reject | new_group's auto-pick predates this story |
| 20 | edge-case | `tabrail-groups-new:39` | A parent name on two lines: the subgroup goes under the first | low | reject | Duplicate names; story 1 row 2 |
| 21 | edge-case | `tabrail-timer-click-test` | The folded count assertion was dropped | low | reject | Its presence now depends on the time's length; head/tests.rs pins it |
| 22 | edge-case | `src/head.rs:22-26`, `src/menu.rs:83` | No `+` or New subgroup… for a name `clean()` changes, though the Approach says every header but Default's | low | reject | Such names cannot travel through tmux command strings (no ▷, Colour… either); quoting them is more than a direct fix |
| 23 | edge-case | AC, `src/draw.rs:69-72` | Session B shows the subgroup only if B holds a tab in it | false | reject | Story 1: a group shows only with a tab of the sidebar's session; the AC checks the re-read across sessions |

## Design Notes

Decided 2026-11-26 (spec-group-timer-start landed after this story was written and put the ▷ one column in from the edge): the `+` takes that column, the ▷ moves two left. At width 26:

```
▾ ◆ Infra             ▷ + 
▸ ◆ Website  3 tabs   ▷ + 
▾ ◆ Ops ▶ 12m           + 
▾ • Default           ▷   
```

`+` at column 25, ▷ at 23, column 26 the pad. A name `clean()` changes gets no `+` (it gets no ▷ either: it cannot travel through the tmux command strings). Tests that pin the ▷ at `width - 1` (`head/tests.rs`, `drag/tests.rs`, `tabrail-timer-click-test`) move with it.

A popup needs a real client. Commands the sidebar sends over its control connection run as the control client, so `display-popup` there has nowhere to open. Either look up the attached non-control client showing this window (`list-clients -f '#{!:#{client_control_mode}}'`, matching `#{window_id}`), or let `tabrail.tmux`'s `MouseUp1Pane` binding handle a release on the sidebar's last column itself, where `#{client_name}` is the user's client. Pick one, and keep drags reaching the sidebar.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-name tabrail-groups && wc -l src/*.rs tabrail-*` -- expected: syntax ok, each under 100
- `./tabrail-tree-test` -- expected: every line `ok`
