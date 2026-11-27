---
title: 'Drag groups with their tree'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: '04ef83b22f91dbeecca661b207b066513c94c9ed'
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
- [x] `tabrail-groups` -- `move-group SRC ONTO` moves SRC's subtree above ONTO or below ONTO's subtree, refusing non-siblings; `nest SRC PARENT` moves SRC's subtree after PARENT's subtree at PARENT's depth + 1, refusing Default and SRC's own subtree -- whole trees, checked on the bash side too
- [x] `src/drag.rs`, `src/main.rs` -- pick nest or reorder from row and column; resolve the sibling through `groups.rs` -- CAP-5, CAP-7
- [x] `tabrail-tree-test` -- the matrix rows through SGR drags (press `M`, motion `32`, release `m`) -- regression
- [x] `README.md` -- drag text: own level, header right of the name nests -- behaviour reference

Implementation notes: `move-group` and `nest` live in a new sourced `tabrail-groups-move` (one awk for both; `tabrail-groups` would pass 100 lines), the function still `move_group` (tabrail-colour's comment names it). A tree is SRC's line through the last group line of its subtree; comment and blank lines inside move verbatim, ones after its last group line stay. Up inserts right above ONTO's line, down and nest right after the last group line of ONTO's tree. The moved top line gets 2 spaces per level clamped, as `sub_line` does, into [indent of the next group line if a sibling, else parent's + 1; indent of the sibling above the insertion point], so the parent rule keeps the tree in hand-indented files (4 spaces, tabs, uneven siblings); its subgroups 2 more per level. `move-group X Default` when X's tree is already last: nothing. The drop logic moved from `drag.rs` (96 lines) to `src/drag/group.rs`; `groups.rs` gains `find`, `parent`, `sibling`, `nests`. "Köniz released on 4tw's rows" read as tab rows and 4tw's header left of its name: right of the name Köniz nests into 4tw (CAP-5). The matrix went into a new `tabrail-tree-drag-test`, as `tabrail-tree-test` (87 lines) cannot take it under 100 (stories 2 and 3 did the same); `tabrail-tree-test`'s flat drag and two `tabrail-timer-click-test` drags now release on a tab row or left of the name, as their old column now nests. Unit tests: `src/drag/tests/tree.rs`, `src/groups/tests.rs`; `drag/tests.rs` and `plus.rs` adjusted (release on a header's name nests). Test safety: helpers called with `TMUX="$(t display -p '#{socket_path}'),0,0"` reach the user's tmux (and its groups file) when the test server is down (`,0,0` with an empty path); `tabrail-test-lib` gains `sock` (the `-L` socket path, a fixed string: never the user's tmux), used by this test, tree-, subgroup-, folder-, bar- and timer-test; perf-test keeps `#{socket_path}` behind its `-n` guard. Writers through `tg` also check `t has-session`, as tabrail-groups falls back to `$D/groups.conf` without the server.

**Acceptance Criteria:**
- Given a nested groups file that is a symlink, when a nest drag lands, then the link is still a link and the target's mode is unchanged.
- Given any of the matrix's moves, when every sidebar redraws, then all sessions show the new tree.

## Review Triage Log

| # | Layer | Location | Claim | Verdict | Route | Evidence |
|---|-------|----------|-------|---------|-------|----------|
| 1 | blind, edge-case | `tabrail-test-lib:14`, `tg` in tree-drag/subgroup tests | With the test server gone, tabrail-groups falls back to `$D/groups.conf` | medium | patch | `tmux show` fails, CONF defaults to the checkout's file (none exists today); `t has-session` guard |
| 2 | verification-gap | `tabrail-groups-move:43-45` | Comment and blank lines inside a moved tree are never tested | medium | patch | Pre-verified: a mutant leaving them behind is byte-identical on every test file |
| 3 | blind | `tabrail-groups-move:37` | No test moves a group down between nested siblings | medium | patch | Only nested writer case is an up move; the user's main use is reordering customers inside egov |
| 4 | blind | `groups.conf.example` | Dangling possessive; "on that group's name" vs "at or right of" | low | patch | Direct text fix |
| 5 | blind | `src/drag/tests/tree.rs:71` | `// folded` claims a case drop_on never sees | low | patch | Direct deletion |
| 6 | blind | `tabrail-groups-move:37` | An up move lands below ONTO's comment; SRC's comment stays behind | low | defer | The single-line move-group inserted at the same place |
| 7 | blind | `tabrail-groups-move:45` | Comments inside a moved tree keep their old indent | low | reject | Comments are not read as groups at any indent; cosmetic |
| 8 | blind | `tabrail-groups-move:46` | A no-op drop rewrites the file, re-indenting hand-indented moved lines | low | reject | Frozen Always: moved lines are re-indented at 2 per level |
| 9 | blind | `tabrail-groups-move:46` | The safety check does not prove other groups keep their parents | low | reject | Implements the frozen check: same lines ignoring leading whitespace |
| 10 | blind, edge-case | `src/drag/group.rs:24` | `depth + 5` assumes a one-column icon | low | reject | Char counting predates this story (spec-group-timer-start row 2); the user's icons are one cell |
| 11 | blind | `README`, matrix Folded | Nesting into a folded group hides the dragged group | low | defer | Same cause as story 3's folded entry |
| 12 | blind | `tabrail-tree-drag-test` | AC "all sessions show the new tree" checked after 2 of 9 moves; `safe` not after every write | low | reject | One redraw path (re-read on `sidebar-refresh`) for every move |
| 13 | blind, verification-gap, edge-case | story Verification, notes | Verification omits tree-drag-test; note says bar/timer keep the old form | low | reject | Fix edits this build's spec; every suite ran and passes. The note was corrected as housekeeping |
| 14 | blind | `deferred-work.md` DW-15 | Ledger entry now covered | low | reject | Append-only ledger |
| 15 | blind | `tabrail-groups-move` | Story 5's "back to the top level" cannot reuse `nest`/`move-group Default` | low | reject | Story 5's work; noted for it |
| 16 | edge-case | `src/groups.rs:44-61` | Duplicate names or a nested `Default` line make find() pick the wrong group | low | reject | Hand-written duplicates; story 1 row 2 |
| 17 | edge-case | `tabrail-groups-move:26` | NBSP before `#`: Rust reads a comment, awk a group | low | reject | No realistic input |
| 18 | edge-case | `tabrail-groups-move:21-37` | No lock between overlapping writers | low | reject | spec-group-drag R19 |

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-groups && wc -l src/drag.rs tabrail-groups` -- expected: syntax ok, under 100
- `./tabrail-tree-test` -- expected: every line `ok`
