---
title: 'Reader bar across the bottom'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_revision: 'b026ce50d78ffe00b27b053761195b8e3a43e7b1'
route: 'dispatch'
review_loop_iteration: 0
followup_review_recommended: false
deferred:
  - summary: >-
      Layout commands other than even-horizontal and main-vertical leave the sidebar off the left
      column; width.rs restores only a sidebar that is still the left column.
    evidence: |-
      Blind Hunter and Edge Case Hunter on throwaway tmux 3.8 servers: after even-vertical,
      main-horizontal, main-horizontal-mirrored and tiled the sidebar is a full-width top row
      (flags 01101 / 01001), after main-vertical-mirrored it sits at the right (00101); width.rs
      keep acts only on 01100, so @sidebar_relayout stays 1. Pre-existing for every layout but
      tiled; in tiled, place() taking the reader cell out now leaves the sidebar alone in its row
      (before: a half-height top-left cell, also out of its column).
    location: >-
      src/width.rs keep; tabrail-reader-size place()
    severity: medium
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-reader-bar/stack.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The reader pane sits 26 columns wide under the sidebar, too narrow to read an answer in its small state (SPEC Why, CAP-1, CAP-2).

**Approach:** `tabrail-reader` places the pane across the full width of the tab's bottom (`split-window -fv`, `join-pane -fv`) instead of under the sidebar; small is 4 rows, big stays 2/3 of the window height; a bar that tmux moved out of place (layout command, a sidebar added beside it) is put back across the bottom when it is fitted.

## Boundaries & Constraints

**Always:**
- Heights only: no pane's width changes through move, grow or shrink; `@sidebar_width` is never written.
- `follow`, `fit`, `away` keep their names, arguments and callers (`tabrail.tmux` hooks, `tabrail-tmux`); the lock, the 10 s restart guard, zoom handling (`rezoom`, no resize while zoomed) stay.
- `fit WINDOW`: if the bar is in WINDOW but `pane_width < window_width` or not `pane_at_bottom`, `join-pane -fv -d -l <its height> -s <bar> -t <sidebar>` first, then size it as today.
- `after-select-layout[50]` keeps `set -w @sidebar_relayout 1` and also runs `tabrail-reader fit '#{window_id}'` when the window holds the reader (same `if -F` guard as `window-pane-changed`).
- Option names `@reader`, `@reader_big`, `@tabrail_reader`, `@tabrail_reader_started` unchanged.

**Never:** Change `src/` logic (only `src/width.rs`'s doc comment); change `tabrail-tmux` behaviour; touch the reader command or `~/Dev/vmConfig`; test on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Start | tab 1 shown, no reader | bar 160x4 at `pane_left` 0, bottom row | 10 s guard as today |
| Tab switch | bar in tab 1 → tab 3 | tab 3 bottom, full width, same pid; tab 1 panes back to full height | N/A |
| Grow / shrink | click into bar / own pane | 160x30 / 160x4; every pane's width unchanged | zoomed: untouched |
| Layout command | prefix Space in a tab with the bar | bar back across the bottom at its size; sidebar back to shared width (width.rs) | N/A |
| Sidebar re-added | `ensure` splits `-hbf` beside the bar | next `follow`/`fit` re-joins it full width | N/A |
| Resize | terminal 160x46 → 160x56 | bar keeps 4 rows (or 2/3 if big) | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-reader:22-41` -- `move()` joins under `$4` (sidebar) with `-v`; `start()` splits `-v -t sidebar`. Switch both to `-fv`; keep `-t` on the sidebar pane (any pane of the window works with `-f`). Header comment says "under the sidebar".
- `tabrail-reader-size` -- `SMALL=5` → 4 (no TUI mode line under the input any more); `fit()` gains the full-width re-join; `big`/`small` unchanged.
- `tabrail.tmux:18,37-58` -- `after-select-layout[50]`; reader hooks' comment block.
- `src/width.rs:1-17` -- doc comments: "The reader pane shares the column" → it sits across the bottom; flags "01100" (sidebar not at bottom) still describe it. No code change.
- `tabrail-reader-test` -- geometry checks assume 26x5 under the sidebar and "own panes untouched" while big; `sizes`/`mine` helpers reusable.
- `README.md` -- "Reader pane" bullet (placement, "5 rows at the sidebar's width", "only in height: the sidebar and the tab's own panes stay") and the Files row for `tabrail-reader`.
- `tabrail-tmux` -- read only: `ensure`, `reap`, `close` call `reader follow`/`away`.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-reader` -- `-fv` in `move` and `start`; comments -- CAP-1.
- [x] `tabrail-reader-size` -- SMALL 4; re-join in `fit`; comment -- CAP-2 and displaced bars.
- [x] `tabrail.tmux` -- layout hook runs `fit`; comments -- layout commands rearrange the bar.
- [x] `src/width.rs` -- doc comment only.
- [x] `tabrail-reader-test` -- expectations per the I/O matrix: 160x4 at `0,42` on a 160x46 server; big 160x30; widths of sidebar and own panes unchanged while big and after; prefix Space check; keep every other check; stay under 100 lines (move helpers if needed).
- [x] `README.md` -- reader-pane bullet and Files row.

**Acceptance Criteria:**
- Given the reader in tab 1, when tab 3 is selected, then `pane_width == window_width`, `pane_left == 0`, `pane_at_bottom == 1` in tab 3 with the same pid.
- Given a grown bar, when the tab's own pane is clicked, then the bar is 4 rows and `#{pane_width}` of every pane in the tab equals what it was before the bar grew.
- Given `tabrail-reader-test`, then every line is `ok`.

## Implementation Notes

- Baseline was SMALL=6 (c076ee8 merged after this story was written), not the 5 the Code Map names; it goes to 4 as the intent says.
- `-l <its height>` in `fit`'s re-join is read as the height the bar should have (SMALL, or 2/3 with `@reader_big`, via the new `tall()` that `big()` uses too): its height after a layout command (up to 46 rows) would squash the tab's panes between the join and the resize.
- `place()` does nothing when the window has no sidebar (an empty `-t` would target the current pane, maybe in another window); `tabrail-tmux ensure` adds one and its `follow`/`fit` re-joins the bar.
- Tests beyond the listed ones: big after a resize (37 rows at 56), a sidebar killed and re-added through `ensure`, fit in a zoomed tab, a second prefix Space (even-vertical), the restarted reader's geometry.

## Spec Change Log

## Review Triage Log

### 2026-11-23 — Review pass
- verdicts: 24 findings — high 0, medium 6, low 15, false 3, maybe-false 0
- findings:
  - `[medium]` `[reject]` (intent-alignment) SMALL=4 with today's Claude TUI as the reader leaves no answer line in the small bar; the test's `sleep` reader never shows content — real until story 4 runs tabrail-chat, but the intent sets small to 4 rows and keeping 6 edits this build's spec; recorded as a residual risk.
  - `[low]` `[patch]` (intent-alignment) only one layout tested, only while small — grouped with the verification-gap `pane_at_bottom` finding; a second prefix Space check added. Own panes' widths after a repair are not a move/grow/shrink, so outside the heights-only rule.
  - `[low]` `[reject]` (intent-alignment) zoom check calls `fit` directly and asserts only the zoom flag — the guard returns before `place` and any resize, and the unzoom is the harm it prevents; removing the guard fails the check (mutation run).
  - `[false]` `[reject]` (intent-alignment) width invariant checked on tab 3 only — the tab-switch, grow, shrink and move-out checks cover every path the intent names (tab 1 back to `26x46 133x46`, tab 3 back to its sizes).
  - `[low]` `[reject]` (intent-alignment) switch-client check asserts session and pid, not geometry — it runs the same `move()` whose `-fv` the tab-switch check asserts.
  - `[low]` `[reject]` (intent-alignment) swap-pane / rotate-window have no trigger — the old code never repaired them either; the next pane change or resize now does; a `window-layout-changed` hook guarded against fit's own resizes is more than a direct fix for a rare case.
  - `[false]` `[reject]` (intent-alignment) `place()`'s no-sidebar return is not in the intent — no bad outcome: without it `-t ""` targets the current pane; the Sidebar re-added path repairs the bar once `ensure` adds the sidebar.
  - `[medium]` `[reject]` (blind-hunter) SMALL=4 breaks today's TUI reader — same as the first row: the fix edits this build's spec.
  - `[medium]` `[defer]` (blind-hunter) layout commands other than even-horizontal and main-vertical leave the sidebar out of its column — pre-existing (width.rs acts only on 01100); deferred in frontmatter.
  - `[low]` `[reject]` (blind-hunter) prefix `}` and a bare `next-layout` never trigger a fit — same as the swap-pane row.
  - `[low]` `[patch]` (blind-hunter) `place()` comment says "at the height it had"; joins at the height `@reader_big` implies — comment now says "SMALL rows high or 2/3 if big". Sizing twice is not a defect: joining at the final height avoids an intermediate layout and fit's resize is then a no-op.
  - `[low]` `[patch]` (blind-hunter) the `source` comment in tabrail-reader names only big, small, fit — a comment line above it now lists `big, small, fit, place, tall` and that they use `reader` and `sidebar`.
  - `[low]` `[reject]` (blind-hunter) prefix Space while big (the `tall` branch) untested — fit's `case` sets the final height right after `place`, so the end state does not depend on that branch.
  - `[false]` `[reject]` (blind-hunter) the brace-block layout hook is untested without the bar or with `@tabrail_reader` unset — tabrail-reader exits when `@tabrail_reader` is empty and `fit` returns when the reader is in another window; `@sidebar_relayout` is asserted through the prefix Space check's sidebar width 26 (without it width.rs would adopt 53).
  - `[low]` `[reject]` (blind-hunter) `move`/`start` into a zoomed tab untested — `rezoom` is unchanged and was untested before; the implementer checked it by hand on `-L tabrail-zt`; the file is at 99 lines.
  - `[low]` `[reject]` (blind-hunter) zoom check asserts only the flag — same as the intent-alignment zoom row.
  - `[low]` `[patch]` (blind-hunter) switch-client, prefix & and reap checks assert no geometry — grouped with the verification-gap restart finding: the restart check now asserts `160 0 1`; switch-client and & run the covered `move()`.
  - `[low]` `[reject]` (blind-hunter) the test hardcodes 160x4 / 0,42 — the pre-change test did the same with 26x6; deriving them is an uninvited refactor.
  - `[low]` `[reject]` (blind-hunter) story file out of step (tasks unticked, Code Map SMALL=5) — the fix edits this build's spec; tasks are ticked at finalize and the SMALL baseline is noted in Implementation Notes.
  - `[medium]` `[defer]` (edge-case-hunter) even-vertical, main-horizontal, tiled move the sidebar off the left column and width.rs never restores it — same root cause as the blind-hunter sidebar row; deferred.
  - `[low]` `[reject]` (edge-case-hunter) with `pane-scrollbars on` the bar is window_width - 1 wide and `place` re-joins it on every fit — the option is off by default and not set in ~/.tmux.conf (vmConfig/tmux.conf); the fix needs a different geometry test.
  - `[low]` `[patch]` (edge-case-hunter) only the first prefix Space is checked — grouped with the verification-gap `pane_at_bottom` finding; second press added.
  - `[medium]` `[patch]` (verification-gap) `start()`'s `-fv` never checked: later fits repair the first start before the only check — "reader gone: started again" now also asserts `#{pane_width} #{pane_left} #{pane_at_bottom}` == `160 0 1`; fails with `-f` dropped (mutation run on the final tree).
  - `[medium]` `[patch]` (verification-gap) `place()`'s `pane_at_bottom` branch never decides — a second prefix Space (even-vertical) and "prefix Space again: the bar back down at the bottom"; fails with the branch removed (mutation run on the final tree).

## Design Notes

Verified on tmux 3.8 (`-L spec-rb`): with sidebar `%1` and bar `%2`, `select-layout even-horizontal` makes `%2` a 52x46 column; `join-pane -fv -d -l 4 -s %2 -t %1` puts it back at `0,42 160x4`. Detect "out of place" with one `display -p -t <bar> '#{pane_width} #{window_width} #{pane_at_bottom}'`.

## Verification

**Commands:**
- `cargo build --release --offline && bash -n tabrail-reader tabrail-reader-size tabrail-reader-test && wc -l tabrail-reader* ` -- expected: builds, parses, each under 100 lines.
- `./tabrail-reader-test` -- expected: all `ok`, exit 0; it kills its own `-L tabrail-rt`/`-L tabrail-ro` servers.

**Manual checks (if no CLI):**
- `tmux -L tabrail-ro capture-pane -p` during the test: the bar's rows span the full width under the sidebar's clock line.

## Auto Run Result

Status: done

**Summary:** the reader pane is a bar across the full width of the tab's bottom (`split-window -fv`, `join-pane -fv`), 4 rows small, 2/3 of the window high while it has the keyboard. `fit` puts a bar that tmux moved (layout command, a sidebar split in beside it) back across the bottom; the `after-select-layout` hook runs `fit`. Built on `lde/reader-bar` from `origin/main` b026ce5 (the worktree was on the merged `lde/reader-tall`).

**Files changed:**
- `tabrail-reader` -- `-fv` in `move` and `start`; header and function comments describe the bar.
- `tabrail-reader-size` -- `SMALL=4`; `tall()`; `place()` re-joins a displaced bar, called by `fit` after the zoom return.
- `tabrail.tmux` -- `after-select-layout[50]` also runs `tabrail-reader fit` when the window holds the reader; comments.
- `src/width.rs` -- doc comments only.
- `tabrail-reader-test` -- geometry per the I/O matrix; new checks for big after a resize, zoom, prefix Space twice, a re-added sidebar, the restarted reader's place. 24 checks, 99 lines.
- `README.md` -- reader-pane bullet and Files row.

**Review findings:** 24 (medium 6, low 15, false 3). Patched 4 entries: restart geometry check (medium), second prefix Space check for the `pane_at_bottom` branch (medium), `place()` comment (low), `source` comment (low). Deferred 1: layout commands other than even-horizontal and main-vertical leave the sidebar out of its column (pre-existing, width.rs). Rejected: SMALL=4 with today's TUI (intent sets 4; fixing edits the spec), swap-pane/next-layout without a trigger (rare, pre-existing, needs a new guarded hook), `pane-scrollbars on` (not set, off by default), the `tall` branch, `move` into a zoomed tab, zoom check scope, switch-client geometry, hardcoded geometry in the test, story file bookkeeping; false: width invariant only on tab 3, the no-sidebar return, the hook untested without the bar. Reasons per row in the Review Triage Log.

**Follow-up review recommendation:** false. Patched at entry verdict: medium 2, low 2. Both medium patches only add checks; each was shown to fail on the final tree with its bug put back (`-f` dropped from `start`, the `pane_at_bottom` test removed from `place`), so no unverified risk remains.

**Verification:** `cargo build --release --offline` builds; `bash -n` parses all three scripts; `wc -l`: 75, 48, 99. `./tabrail-reader-test`: 24 `ok`, exit 0, `-L tabrail-rt`/`-L tabrail-ro` gone afterwards. Mutation runs on scratch copies: zoom guard removed fails the zoom check; `-f` dropped from `start` and the bottom test removed fail the restart and second prefix Space checks. Implementer's screen check: `capture-pane` on `-L tabrail-ro` shows the bar's border across all 160 columns under the sidebar's clock line.

**Residual risks:**
- Until story 4 runs tabrail-chat, a 4-row bar with Claude Code's TUI (prompt box plus mode line) shows no answer line; merge this branch to main only with story 4.
- After the second and later prefix Space presses the sidebar stays a full-width row or at the right (deferred); the bar itself is back at the bottom.
- swap-pane, rotate-window or a bare `next-layout` leave the bar out of place until the next pane change or resize.
- Live tmux runs ~/Dev/tabrail (main): nothing changes until merge, rebuild there, re-source tmux.conf (new layout hook) and `tabrail-tmux restart`.
