---
title: 'Reader bar across the bottom'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
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
- [ ] `tabrail-reader` -- `-fv` in `move` and `start`; comments -- CAP-1.
- [ ] `tabrail-reader-size` -- SMALL 4; re-join in `fit`; comment -- CAP-2 and displaced bars.
- [ ] `tabrail.tmux` -- layout hook runs `fit`; comments -- layout commands rearrange the bar.
- [ ] `src/width.rs` -- doc comment only.
- [ ] `tabrail-reader-test` -- expectations per the I/O matrix: 160x4 at `0,42` on a 160x46 server; big 160x30; widths of sidebar and own panes unchanged while big and after; prefix Space check; keep every other check; stay under 100 lines (move helpers if needed).
- [ ] `README.md` -- reader-pane bullet and Files row.

**Acceptance Criteria:**
- Given the reader in tab 1, when tab 3 is selected, then `pane_width == window_width`, `pane_left == 0`, `pane_at_bottom == 1` in tab 3 with the same pid.
- Given a grown bar, when the tab's own pane is clicked, then the bar is 4 rows and `#{pane_width}` of every pane in the tab equals what it was before the bar grew.
- Given `tabrail-reader-test`, then every line is `ok`.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Design Notes

Verified on tmux 3.8 (`-L spec-rb`): with sidebar `%1` and bar `%2`, `select-layout even-horizontal` makes `%2` a 52x46 column; `join-pane -fv -d -l 4 -s %2 -t %1` puts it back at `0,42 160x4`. Detect "out of place" with one `display -p -t <bar> '#{pane_width} #{window_width} #{pane_at_bottom}'`.

## Verification

**Commands:**
- `cargo build --release --offline && bash -n tabrail-reader tabrail-reader-size tabrail-reader-test && wc -l tabrail-reader* ` -- expected: builds, parses, each under 100 lines.
- `./tabrail-reader-test` -- expected: all `ok`, exit 0; it kills its own `-L tabrail-rt`/`-L tabrail-ro` servers.

**Manual checks (if no CLI):**
- `tmux -L tabrail-ro capture-pane -p` during the test: the bar's rows span the full width under the sidebar's clock line.
