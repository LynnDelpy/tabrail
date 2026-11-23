---
title: 'Drag groups with their tree'
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

**Problem:** move-group moves one line, so reordering a nested file tears subgroups away from their parent, and no drag nests a group.

**Approach:** A group moves with its whole subtree. Released on another group's header at or right of that header's name, it becomes that group's last subgroup (re-indented). Released left of a header's name or on any tab row, it reorders against the target's ancestor-or-self that is its sibling: above when dragged up, below that sibling's subtree when dragged down. With no sibling there, nothing happens.

## Boundaries & Constraints

**Always:** Siblings share a parent; top-level groups and Default are siblings. Moved lines are re-indented at 2 spaces per level, keeping everything after their leading whitespace. Every other line stays byte for byte. Written safely (symlink, mode, rename into place), only when the same lines, ignoring leading whitespace, are still there. A press and release on one header still folds (or hits story 3's `+`). Names go into tmux strings only if `clean()` leaves them unchanged, as today.

**Never:** Taking a group out of its parent by drag. Moving Default, nesting it, or nesting into it. Nesting into the group itself or one of its descendants. Changing tab drags.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Up | 4tw (below egov) released on an egov tab row | 4tw's tree above egov's line | N/A |
| Down | egov released on 4tw's header, left of its name | egov's tree below 4tw's tree | N/A |
| Via subgroup | 4tw released on St.Gallen's header left of its name | 4tw above egov | N/A |
| Nest | 4tw released on St.Gallen's header at or right of its name | `    4tw` after St.Gallen's subtree, 4tw's subgroups 2 levels deeper | N/A |
| No sibling | Köniz (in egov) released on 4tw's rows | file identical | helper not run |
| Descendant | egov released right on St.Gallen's header | file identical | N/A |
| Own parent | Köniz released right on egov's header | Köniz's tree becomes egov's last subgroup | N/A |
| Default | top-level 4tw on a Default tab row / right on Default's header | below the last top-level tree / identical | N/A |
| Folded | 4tw released right on folded egov's header | nested into egov | N/A |

</frozen-after-approval>

## Code Map

- `src/drag.rs:44` `reorder()` -- today `onto` is the row's group, then `run-shell -b '{helper} move-group "src" "onto"'`. Needs the group tree (`src/groups.rs`, story 1) and `click.col` (story 3); `drag::left` gains both.
- Header layout (story 1): name starts at column `depth + 5` (1-based) after `{indent}▾ {icon} `, with icons one column wide.
- `tabrail-groups` `move_group` (moved from tabrail-act in story 1) -- two-pass awk, `cmp -s <(sort) <(sort)` safety check; must move line ranges (a subtree) instead of one line, and its check must ignore leading whitespace once lines are re-indented.
- `src/main.rs` -- passes the groups list to `drag::left`.
- `README.md` mouse bullet: "drag it onto another group's row or one of its tabs to move its line…".

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-groups` -- `move-group SRC ONTO` moves SRC's subtree above ONTO or below ONTO's subtree, refusing non-siblings; `nest SRC PARENT` moves SRC's subtree after PARENT's subtree at PARENT's depth + 1, refusing Default and SRC's own subtree -- whole trees, checked on the bash side too
- [ ] `src/drag.rs`, `src/main.rs` -- pick nest or reorder from row and column; resolve the sibling through `groups.rs` -- CAP-5, CAP-7
- [ ] `tabrail-tree-test` -- the matrix rows through SGR drags (press `M`, motion `32`, release `m`) -- regression
- [ ] `README.md` -- drag text: own level, header right of the name nests -- behaviour reference

**Acceptance Criteria:**
- Given a nested groups file that is a symlink, when a nest drag lands, then the link is still a link and the target's mode is unchanged.
- Given any of the matrix's moves, when every sidebar redraws, then all sessions show the new tree.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-groups && wc -l src/drag.rs tabrail-groups` -- expected: syntax ok, under 100
- `./tabrail-tree-test` -- expected: every line `ok`
