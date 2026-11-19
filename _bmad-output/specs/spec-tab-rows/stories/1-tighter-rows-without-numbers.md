---
title: 'Tighter rows without numbers'
type: 'feature'
created: '2026-11-19'
status: 'done'
baseline_revision: '7b7f16b1eee56b354e768bb5906c41827ca8d26c'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/specs/spec-tab-rows/row-layout.md'
  - '{project-root}/AGENTS.md'
warnings: []
deferred:
  - summary: >-
      Row width is measured in chars, so a double-width character (emoji or CJK in a tab name, group
      icon or group name) makes a row wider than the pane.
    evidence: |-
      draw.rs fit() and the tab row's room/rest math use chars().count(); a 2-column character
      overflows the 26-column pane and wraps into the next row, and diff-only redraw can leave it
      corrupted. Predates this story.
    location: >-
      src/draw.rs:27-30, src/draw.rs:76-78
    severity: low
  - summary: >-
      The README example is simplified against real output: no leading blank line, no blank line
      after each group, no folded-group row, clock indented 12 instead of centred in 26 columns.
    evidence: |-
      draw.rs:56 and :83 push blank lines and {clock:^width$} centres the clock at columns 5-21;
      the README block shows neither. The simplification predates this story, which only moved the
      columns and kept the clock line as the spec asked.
    location: >-
      README.md:6-14
    severity: low
  - summary: >-
      No automated test pins the sidebar layout; only the throwaway-tmux capture-pane harness checks
      it, and it does not run on later builds.
    evidence: |-
      The repo has no #[test] or tests/ dir; cargo test runs 0 tests. Restoring the index in
      draw.rs:77 still builds and passes. A std-only src/draw_test.rs (via #[cfg(test)] #[path])
      calling draw::screen and stripping ANSI would pin it; it would be the repo's first test, a
      convention call for the owner.
    location: >-
      src/draw.rs:54-89
    severity: low
  - summary: >-
      When content overflows the pane, rows is not truncated with lines, so the clock line maps to a
      hidden tab or group row for clicks and right-click menus.
    evidence: |-
      draw.rs:86-87 resizes lines to height-1 and pushes the clock but leaves rows as built, so
      rows.get(height) (main.rs:61, :75) returns the first hidden row's entry. A rows.truncate(height)
      would fix it. Predates this story; the test pane (30 rows, 11 content lines) never overflows.
    location: >-
      src/draw.rs:86-87
    severity: low
---

<intent-contract>

## Intent

**Problem:** Sidebar rows spend columns on the window number and on indentation (group rows start in column 2, tab names in column 6), leaving fewer of the 26 columns for names the user actually reads.

**Approach:** Drop the window index from tab rows and shift both row kinds left to the columns in `row-layout.md` (SPEC CAP-3, CAP-4): group arrow in column 1, tab `❯` in column 2, tab name from column 4. Update the README example to match.

## Boundaries & Constraints

**Always:** Row columns exactly as the `row-layout.md` table. The window index stays the ordering key (`model::tabs` BTreeMap, `drag::drop_on` before/after). `Screen.rows` stays index-aligned with `Screen.lines` (one leading `Row::Nothing` for the 1-based mouse row, one per line after). Unchanged: full-width group tint, the blank line after each group, status right after the name with a long name cut to make room, the clock on the last line, colours, width.

**Never:** Touch `input.rs`, `drag.rs`, `menu.rs`, `model.rs` or `tabrail.tmux` (clicks, drags and menus key on the row, never the column). Change tab names (story 2), tmux window numbering, or prefix+digit bindings. Test against the user's default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open group | group `Website`, icon `◆`, unfolded | `▾ ◆ Website` from column 1, tinted to full width | No error expected |
| Folded group | `Notes` folded, 2 tabs | `▸ ● Notes  2 tabs` from column 1 | No error expected |
| Current tab | window 1 current, name `Fix login redirect`, status `⚙ 2m` | ` ❯ Fix login redirect ⚙ 2m`; no `1` | No error expected |
| Other tab, no status | window 4, name `infra` | `   infra`, no index | No error expected |
| Long name with status | name longer than 26 - 3 - status - 1 | name cut so ` ⏱ 4m` still fits in 26 columns | Cut, never wrap |
| Index ≥ 10 | window 12 | no `12` on the row | No error expected |

</intent-contract>

## Code Map

- `src/draw.rs:64` -- group head `format!(" {} {icon} {name}", …)`: the leading space is the column-2 start; drop it.
- `src/draw.rs:77` -- tab label `format!("  {mark} {} {}", tab.index, tab.name)`: becomes `" {mark} {name}"` (space, mark, space, name). `room`/`rest`/`gap` math (lines 75-80) works on char counts of `label` and needs no change.
- `src/draw.rs:56-57,70,81,84` -- `lines` starts with one blank line, `rows` with two `Row::Nothing` (1-based click row). Every `lines.push` has a matching `rows.push`; keep that pairing.
- `src/input.rs:15-18` -- read-only evidence: SGR parse keeps `n[2]` (row) only; the column `n[1]` is ignored.
- `src/main.rs:61,75` -- read-only: menus `rows.get(y + 1)` and clicks `rows.get(click.row)` index by row.
- `tabrail.tmux:17` -- read-only: right-click menu file chosen by `#{mouse_y}`.
- `src/drag.rs:46`, `src/model.rs:32-42` -- read-only: `Tab.index` still used for ordering; field stays.
- `README.md:6-13` -- example block: rows in the old columns with numbers; comments aligned at column 36.

## Tasks & Acceptance

**Execution:**
- `src/draw.rs` -- drop the leading space from the group head; build the tab label as space, mark, space, name (no index) -- CAP-3, CAP-4 columns.
- `README.md` -- rewrite the example block's group and tab rows to the new columns without numbers; keep names, statuses, comments (same comment column) and the clock line -- README is the behaviour reference.
- Throwaway tmux server (scratchpad conf, not committed) -- check every I/O matrix row and the ACs via `capture-pane` -- surface-level verification; the repo has no unit-test harness.

**Acceptance Criteria:**
- Given a throwaway tmux server running tabrail with groups Website (2 tabs), Infra (2 tabs), Notes (2 tabs, folded), when the outer `capture-pane -p` reads the sidebar, then its rows match the after-layout of `row-layout.md` column for column (names aside) and no row contains a window index.
- Given that server, when a non-current tab row is left-clicked (press and release on the same row), then tmux switches to that tab.
- Given that server, when a tab row is dragged onto another group's tab row, then the tab moves into that group next to the target and tab order is otherwise unchanged.
- Given that server, when a group row is left-clicked, then the group folds or unfolds; when a tab row is right-clicked, then that tab's menu opens.

## Spec Change Log

## Review Triage Log

### 2026-11-19 — Review pass
- verdicts: 20 findings — high 0, medium 0, low 15, false 5, maybe-false 0
- findings:
  - `low` `reject` Intent says old tab names started in column 6; they started in 7 — true, but the fix edits this spec's intent text.
  - `low` `reject` Design Notes say the label shrinks by 4 characters; it is 2 + index digits — true, conclusion unaffected; fix edits this spec.
  - `low` `reject` Verification gives no recipe for the test server (conf, groups, fold, wait for ensure) — fix edits this spec; the scratchpad harness (setup.sh) sources the worktree's tabrail.tmux and does all of it.
  - `false` `reject` AC fixture misses window 12, the long-name cut and a digit-safe index check — check.py creates window 12, the cut row (`   TERM=xterm-256colo ⚙ 4m`, 26 columns) and asserts each tab row's name starts at column 4; all pass.
  - `false` `reject` Drag AC vague; group-row drop, first group row, group right-click, blank/clock-line clicks untested — check2.py drove a drop on the Website group row (joins at its end), column-1 fold/unfold of the first group, Infra group menu, a blank-line click (no-op): all pass.
  - `low` `defer` README example differs from real output (no blank lines, no folded row, clock at column 13) — pre-existing simplification; deferred with the README entry.
  - `low` `defer` Width measured in chars, so double-width characters overflow the row — pre-existing (fit, room/rest); deferred with the wide-character entry.
  - `low` `reject` Hiding the numbers loses the prefix+digit mapping and README does not say so — out of scope: CAP-3 removes the numbers and SPEC lists the digit bindings as a non-goal; README never documented them.
  - `low` `reject` Frontmatter context lacks SPEC.md and README.md — fix edits this spec.
  - `low` `reject` Cleanup filed under manual checks; test servers leave menu files in $XDG_RUNTIME_DIR/tabrail — spec part edits this spec; this run's dead-server menu files (pids 710963-719908) were removed by hand.
  - `low` `reject` Empty tab name (idle shell at /, or renamed to "") draws a blank-looking tab row — rare, and the fix adds a branch; the row still clicks to its tab.
  - `low` `defer` Double-width characters in tab names overflow the pane — same root cause as the wide-character entry.
  - `low` `defer` Double-width group icon or name overflows the tinted head — same root cause as the wide-character entry.
  - `low` `reject` Tabs with identical names are no longer told apart by the index — CAP-3 removes the index by intent; row position and ❯ still separate them and clicks map rows.
  - `low` `defer` No automated test pins the layout; cargo test runs 0 tests — the repo has no test harness (pre-existing); adding its first test module is the owner's convention call; deferred.
  - `false` `reject` Clicks never sent at a moved column — check2.py clicked columns 1 and 24, both map to the row; input.rs:15-18 keeps only the row.
  - `low` `defer` On overflow, rows is not truncated with lines, so the clock line maps to a hidden row — pre-existing at draw.rs:86-87; deferred.
  - `false` `reject` Drag coverage partial (group-row drop, release on empty row) — check2.py group-row drop and blank-line click pass; drag.rs and the rows list are unchanged.
  - `false` `reject` Success signal mixes in story 2's names — the AC says "names aside"; names are story 2.
  - `low` `defer` README example still omits blank lines and the folded group — same root cause as the README entry.

## Design Notes

No unit tests: the repo has none and `draw.rs` is near the 100-line limit; the capture-pane check reads the real surface instead. Status placement needs no change because `room` already subtracts status and gap from the full width and the label shrinks by 4 characters.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds with no warnings.
- `tmux -L outer capture-pane -p` against `tmux -L outer new -d "TMUX= tmux -L test attach"` -- expected: sidebar rows as in `row-layout.md` after-example (group arrow column 1, `❯` column 2, names column 4), no window indexes.

**Manual checks (if no CLI):**
- Kill the test servers (`tmux -L test kill-server`, `tmux -L outer kill-server`) afterwards; never touch the default socket.

## Auto Run Result

Status: done

**Change:** tab rows drop the window number and both row kinds move left to the `row-layout.md` columns: group arrow in column 1, tab `❯` in column 2, name from column 4 (CAP-3, CAP-4). Click, drag and menu mapping is untouched: they key on the row, and `Screen.rows` still pairs with `Screen.lines`.

**Files:**
- `src/draw.rs` -- group head without its leading space; tab label ` {mark} {name}`, no index.
- `README.md` -- example rows in the new columns, no numbers; comments, names and clock line kept.

**Review:** 20 findings (high 0, medium 0, low 15, false 5). Patches 0. Deferred 4 entries (6 findings): double-width characters overflow rows; README example simplified; no automated layout test; rows not truncated on overflow (clock line maps to a hidden row). Rejected: 5 false (coverage claims disproved by the harness runs), 5 whose fix edits this spec, empty-name blank row (rare, adds a branch), identical names without the index (CAP-3 by intent), undocumented digit mapping (non-goal).

**Follow-up review:** not recommended; patched 0 high, 0 medium, 0 low.

**Verification:**
- `cargo build --release --offline` after `touch src/main.rs`: compiles, no warnings.
- Throwaway servers `s1t` inside `s1o` (scratchpad `setup.sh`, sources the worktree's `tabrail.tmux`): `check.py` 20/20 pass (layout rows 1-11 exact, 26-column cut, name at column 4, clock, click to windows 2 and 12, group fold/unfold, tab right-click menu, tab-onto-tab drag); `check2.py` 7/7 pass (clicks at columns 1 and 24, first group fold at column 1, blank-line no-op, group menu, drop on group row). Servers killed, sockets and this run's menu files removed.

**Residual risks:** the layout has no regression test in the repo (deferred). Story 2 changes names only, so the success signal's `lde@app01 infra` row is still open.
