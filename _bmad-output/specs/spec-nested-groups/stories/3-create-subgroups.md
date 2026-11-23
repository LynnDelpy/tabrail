---
title: 'Create subgroups'
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
- [ ] `src/control.rs`, `src/input.rs` -- `Click.col` (1-based) -- the `+` is a column
- [ ] `src/draw.rs` -- dim `+` in the last column of every non-Default header, the header text cut to `width - 2` -- the click target
- [ ] `src/drag.rs`, `src/main.rs` -- press and release on the `+` column of one header: open the name field for a subgroup of that group; else as before -- CAP-4 by click
- [ ] `src/menu.rs` -- `'New subgroup…' s` after New group… in the group menu, not for Default -- CAP-4 by menu
- [ ] `tabrail-name` -- kind `sub` (title `New subgroup of GROUP`) -- the name field
- [ ] `tabrail-groups` -- `new-group NAME TARGET [PARENT]`: with PARENT, the line goes after PARENT's subtree at its depth + 1; a missing or Default PARENT adds nothing -- one auto-pick
- [ ] `tabrail-tree-test` -- the matrix rows through SGR clicks on the `-L` outer server -- regression
- [ ] `README.md` -- `+` and New subgroup… in the groups and mouse text -- behaviour reference

**Acceptance Criteria:**
- Given two sessions on the test server, when a subgroup is created from session A's sidebar, then session B's sidebars show it on their next redraw, without a restart.

## Design Notes

A popup needs a real client. Commands the sidebar sends over its control connection run as the control client, so `display-popup` there has nowhere to open. Either look up the attached non-control client showing this window (`list-clients -f '#{!:#{client_control_mode}}'`, matching `#{window_id}`), or let `tabrail.tmux`'s `MouseUp1Pane` binding handle a release on the sidebar's last column itself, where `#{client_name}` is the user's client. Pick one, and keep drags reaching the sidebar.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-name tabrail-groups && wc -l src/*.rs tabrail-*` -- expected: syntax ok, each under 100
- `./tabrail-tree-test` -- expected: every line `ok`
