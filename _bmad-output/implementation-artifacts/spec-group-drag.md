---
title: 'Drag groups to reorder them'
type: 'feature'
created: '2026-11-19'
status: 'done'
route: 'dispatch'
baseline_commit: '520e4c3a8bf230b58926b9f6a0b51aa6ec84a476'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-group-drag/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Group order is the groups file's line order, changed only by editing the file, while tabs already move with the mouse. A press on a group row folds at once, so a group row cannot start a drag.

**Approach:** Story 1 (CAP-2): a group row folds on release over the same row, like a tab click switches on release; released elsewhere it never folds. Story 2 (CAP-1): released over another group's row or one of its tabs, the pressed group's line moves in the groups file (above that group's line moving up, below it moving down) and every sidebar redraws from the file.

## Boundaries & Constraints

**Always:** The order lives only in the groups file (`@tabrail_groups`, else `groups.conf`); it is also the new-tab folder match order (`tabrail-tmux group`), no second order. A reorder moves exactly one line; comments, blank lines and every other line keep their bytes and relative order. The write goes to a temp file next to the target and is renamed over it only if the result is a permutation of the old lines; a symlinked groups file stays a symlink (write through to its target). Default is not in the file: it cannot be dragged and stays last. Tab press, release, drop targets and right-click menus behave as before. README mouse and groups text updated. Files under 100 lines.

**Never:** Drag preview or drop marker; keyboard or menu reordering; merging or nesting groups; touching the default tmux socket or the user's groups file (tests use a copy named by `@tabrail_groups` on `tmux -L`).

## I/O & Edge-Case Matrix

| Scenario | Shown A, B, C (file order); press → release | Result |
|---|---|---|
| CAP-2 fold | press and release on B's row | B folds (again: unfolds); file unchanged |
| CAP-2 no fold | press B's row, release on A's row or a blank row | folds unchanged |
| Up | C's row → A's row | file: C, A, B; every sidebar shows C, A, B |
| Down | A's row → C's row | file: B, C, A |
| Onto a tab | C's row → a tab of A | as Up |
| Own tab | B's row → a tab of B | nothing |
| Hidden group | file A, X (no tabs), C; C → A | file C, A, X |
| Default dragged | Default's row → A's row | nothing |
| Onto Default | A's row → Default's row or tab (user decision) | A's line after the last group line: shows right above Default |
| Write fails | groups file's folder not writable | file unchanged, no temp left |
| Tab drags | any tab press/release | as before |

</frozen-after-approval>

## Code Map

- `src/drag.rs:14-39` -- `left` folds on the press (`:29-36`). Hold the press as `(row, draw::Row)` for tabs and groups (explicit match, no Clone derive); fold on same-row release with the release-time `folded`. ONTO = the release row's group (`Row::Group`, or `Tab.group` of a `Row::Tab`); blank row, same group or SRC Default: nothing. Names through `menu::clean`. `drop_on` unchanged.
- `src/main.rs:47,86` -- `drag` comment; pass `&helper` (tabrail-act, `:28`) to `drag::left`. Doc `:2-3` "drag a tab" → "drag a tab or group". File is at 99 lines: edit in place, add none.
- `tabrail-act` -- new `move-group SRC ONTO`; reuse `CONF`, `refresh`; usage at `:2-8`, `:72-73`. Group line = not `^#`, `NF >= 3`, first match by `$1` (as `model::groups`). Leave `rename_group`'s write alone.
- `tabrail-tmux:19-29` -- read-only: `group` walks file lines in order, so the match priority follows with no change.
- `README.md:17,30,74` -- groups bullet, mouse bullet, Files row for `tabrail-act`. `groups.conf.example:1-5` -- one comment sentence: dragging a group's row reorders these lines.
- `tabrail.tmux:10-14` -- comment "the sidebar drags tabs with it": tabs and groups. Bindings unchanged.

## Tasks & Acceptance

**Execution:**
- [x] `src/drag.rs` -- story 1: fold on same-row release; story 2: `run-shell -b '{helper} move-group SRC ONTO'`; module doc.
- [x] `src/main.rs` -- pass `&helper`, comments.
- [x] `tabrail-act` -- `move_group`: two-pass awk over `readlink -f` of the file; skip SRC's line, print it before ONTO's moving up, after it moving down (ONTO `Default`: after the last group line); fail if SRC or ONTO is missing or the same line. Write `"$file.new.$$"`; `mv` and `refresh` only if awk succeeded and `sort` of old and new match, else `rm -f`. Exit 0 either way (run-shell -b).
- [x] `README.md`, `groups.conf.example`, `tabrail.tmux` -- text as in the Code Map.
- [x] Scratch `rustc --test` crate (not committed) -- `#[path]` the src modules; tabs from `model::tabs` FIELDS lines; one assert per matrix row through `drag::left`.
- [x] Scratch bash test (not committed) -- `TMUX=<test socket>,0,0 ./tabrail-act move-group` on a copy with comments, a blank line, a hidden group and a symlink; `diff` against expected files; a read-only folder leaves the file and no temp.

**Acceptance Criteria:**
- Given a throwaway server (`tmux -L`, two sessions, sidebars running, `@tabrail_groups` → scratch copy) with groups A, B, C holding tabs, when an SGR drag goes from C's row to A's row, then `capture-pane` of a sidebar in each session shows C, A, B, the copy lists C, A, B with comments unchanged, and A onto C then gives B, C, A.
- Given that server, when `tabrail-tmux restart` runs and then the server is killed and started again, then the sidebars still show the reordered groups.
- Given two groups whose folders overlap (`~/x` and `~/x/y`), when their order is swapped by a drag, then `tabrail-tmux group ~/x/y` prints the one now first.
- Given that server, when a group row gets a press and release on the same row, then it folds; an SGR press on one group row released on another leaves `@sidebar_fold` unchanged.
- Given `cargo build --release --offline`, then no warnings, and every touched file stays under 100 lines.

## Implementation Notes

- AC1 "A onto C then gives B, C, A" holds from A, B, C (matrix row Down). From C, A, B a drag of A onto C moves up (A, C, B), so the live run restored A, B, C by dragging C onto B first (byte-identical to the original file), then A onto C gave B, C, A.
- `move_group` checks validity only in awk's END: an invalid run's partial output is discarded by the `rm -f`, which kept tabrail-act under 100 lines.
- Review fixes: `drag.rs` sends nothing unless both names pass `menu::clean` unchanged and hold no backtick, and passes them double-quoted; awk compares names as strings (`1` no longer matches `01`); `chmod --reference` and `mv` sit in the if-condition, so the mode is kept and any failure reaches `rm -f`.

## Spec Change Log

## Review Triage Log

| # | Source | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|---|
| R1 | verification-gap | fold-on-release and `reorder` in drag.rs pinned only by the uncommitted scratch crate | low | defer | Pre-verified: fold-on-press, no Default guard, swapped names all build; `cargo test` runs 0 tests. Same owner call as DW-3. |
| R2 | verification-gap | `move_group` pinned only by the uncommitted scratch script | low | defer (with R1) | Pre-verified: swapped `<`/`>`, no `readlink -f`, no Default rule all pass unnoticed. Same root cause as R1. |
| R3 | verification-gap (other) | `clean()` changes a name, so it no longer matches the file: `A'x` sends `Ax` (no-op), `C#` sends `C` (moves group C) | low | patch | `menu::clean` drops `' " { } ; \\ $ #`; awk matches `$1` raw. `new_group` only replaces spaces, so such names are reachable. Fix: no command unless both names survive `clean` unchanged. |
| R4 | verification-gap (other) | `rename_group` turns a symlinked groups file into a plain file | low | defer | `tabrail-act:45-46` `mv "$CONF.new" "$CONF"`; predates this story, Code Map leaves it alone. |
| R5 | blind | group names reach `sh` unquoted: `R&D` backgrounds `move-group R`, runs `D Infra`; a backtick executes | medium | patch | Reproduced `sh -c 'echo move-group R&D Infra'` → `move-group R`, `D: command not found`; tabrail-name/new-group allow `&`. Fix: double-quote both names, refuse a backtick. The menu.rs `new`/`fold` half predates the story: defer. |
| R6 | blind | `C#` cleaned to `C` moves the wrong group | low | patch (R3) | Same root cause as R3. |
| R7 | blind | a 0400 groups file is replaced and comes back 0644 | low | patch (mode) / reject (read-only) | `mv` needs only folder write. Mode: `chmod --reference` before `mv`, a direct correction. Refusing a read-only file is a new guard for a file users do not chmod; `sed -i` replaces those too. |
| R8 | blind | every failure is silent | low | reject | A message needs a branch telling failure from the legit no-op (already last, missing name); failures need an unwritable folder. |
| R9 | blind | a comment above a group stays behind when it moves | low | reject | Frozen Always: comments keep their bytes and relative order. |
| R10 | blind | the two groups-file writers disagree on symlinks, no DW entry | low | defer (R4) | Same root cause as R4. |
| R11 | blind | no tests committed | low | defer (R1) | Same claim as R1/R2. |
| R12 | blind | a kill between awk and `mv` leaves the temp file | low | reject | Millisecond window during a server exit; the fix is a trap. |
| R13 | blind | a file without a final newline gains one | low | reject | Reproduced; editors write final newlines and the fix tracks the last line's terminator in awk. |
| R14 | blind | `fold()` writes raw names into `set -g @sidebar_fold '…'`, a `'` breaks it | low | defer | Moved verbatim from the old press branch; predates the story. |
| R15 | edge-case | shell metacharacters in names split the run-shell command | medium | patch (R5) | Same root cause as R5. |
| R16 | edge-case | `clean()` changes a name to another group's or none | low | patch (R3) | Same root cause as R3. |
| R17 | edge-case | awk `==` compares `1` and `01` as numbers | low | patch | Reproduced: `-v o=1` matches line `01`. Fix: compare as strings (`$1 == o ""`). |
| R18 | edge-case | mode, owner and hard links lost on replace | low | patch (R7, mode) | Mode as R7; owner is the user's own, hard links on a dotfile are not met in use. |
| R19 | edge-case | a second move-group or new-group between read and `mv` is lost | low | reject | Two writes within milliseconds; the fix is a lock in two functions. |
| R20 | edge-case | NBSP or form feed separators: Rust splits, awk does not | low | reject | Not met in a hand-written config; the fix changes split rules on both sides. |
| R21 | edge-case | `'` or `\|` in a name breaks `@sidebar_fold` | low | defer (R14) | Same root cause as R14. |
| R22 | edge-case | `mv` failing after the check leaves the temp file | low | patch | Spec matrix: a failed write leaves no temp. Fix: `chmod` and `mv` inside the condition, `rm -f` on any failure. |
| R23 | edge-case | AC1 "A onto C then gives B, C, A" holds only from A, B, C | low | reject | Code follows the matrix's Down row; the fix edits this spec. |

## Design Notes

tabrail-act decides direction from file line numbers, which match screen order (draw.rs shows groups in file order, skipping empty ones). All groups-file writes stay in tabrail-act, which already owns rename and new group. The permutation check guards against a truncated write; the `$$` temp keeps two quick drags apart.

## Verification

**Commands:**
- `cargo build --release --offline`; the scratch Rust and bash tests; `tmux -L outer capture-pane -p` -- expected: no warnings, all pass, the ACs.

**Manual checks (if no CLI):**
- Kill both test servers, their sidebars and their menu files in `$XDG_RUNTIME_DIR/tabrail`; never touch the default socket.
