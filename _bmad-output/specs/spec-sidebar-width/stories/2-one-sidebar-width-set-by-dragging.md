---
title: 'One sidebar width, set by dragging'
type: 'feature'
created: '2026-11-19'
status: 'done'
route: 'oneshot'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-sidebar-width/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-sidebar-width/brownfield.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** tmux spreads every terminal resize over all panes and resizes each tab on its own, so the
sidebar's width differs per tab and drifts; dragging its border fixes one tab only (SPEC.md CAP-1, CAP-2).

**Approach:** One width per tmux server in the global option `@sidebar_width` (default 26). Each sidebar
keeps its own pane at that width over its control connection whenever its window's width or the shared
width changes; when its width changes in place (window unchanged), the user dragged it, and once it
holds still that width becomes the shared one. `tabrail-tmux ensure` opens new sidebars at it.

</frozen-after-approval>

## Implementation Notes

- No mouse binding: `~/Dev/vmConfig/tmux-mouse.conf` binds `MouseDragEnd1Border` after tabrail.tmux,
  and a release off the border fires `MouseDragEnd1Pane` (tested on tmux 3.8). The sidebar detects the
  drag itself, which needs no binding and survives either release.
- Consequence: a keyboard `resize-pane` of a sidebar is adopted like a drag (SPEC assumption said
  undone). Layout commands are not: tabrail.tmux's `after-select-layout` hook marks the tab
  (`@sidebar_relayout`) and the sidebar goes back to the shared width; prefix Space is rebound to
  `select-layout -n`, as `next-layout` has no hook. Not touched while zoomed (`resize-pane` would
  unzoom), when a layout moved it off the left column, or when it is alone in its tab.
- Adoption waits until the width holds over two frames, so a drag usually sets the option once; a
  quiet frame mid-drag adopts the width so far (each set makes every sidebar resize and redraw).
- Review: the frame that resizes skips drawing, and any size change repaints the whole sidebar
  (`drawn` in main.rs): this is story 1's CAP-3 fix, pulled in because story 2 resizes far more often.
  `ensure` falls back to 26 on a non-numeric option and never marks the tab's own pane as a sidebar.
- After resizing itself the sidebar re-baselines on the next frame, so a width tmux clamped (narrow
  terminal, story 3) is never adopted.
- Files: `src/width.rs` (new), `src/main.rs` (query + call), `tabrail-tmux` (ensure reads the option),
  README (behaviour, setup, files), AGENTS.md (option list).
- Verified on `tmux -L rsz` + `-L outer -f /dev/null`, 3 tabs: 120 → 73 → 200 and every tab visited:
  all 26; tab 0 dragged to 34: all 34 within 2.5 s, new tab 34, 150 and 90 columns keep 34; zoomed tab
  resized to 130 stays zoomed, 34 on unzoom; drag paused 2 s mid-way is not fought (38, then 40).
- Re-verified after review: `main-vertical` and `even-horizontal` restore 34 and keep the shared width;
  prefix Space to a top sidebar leaves it alone; 26 → 40 → 26 leaves no stray rows; `abc` gives a new
  tab a 26-wide sidebar; a sidebar left alone is reaped with the shared width unchanged.

## Review Triage Log

- medium, patched: layout commands were adopted as the shared width (`main-vertical` → 80 everywhere).
- medium, patched: a non-numeric `@sidebar_width` failed the split and `set -p -t ""` marked the
  content pane a sidebar, so `reap` could close the tab; `20%` differed between shell and Rust.
- medium, patched: a forced resize drew at the old width and left reflow leftovers (CAP-3).
- medium, patched: README's `set -g` reset a dragged width on every reload; now `set -goq`; behaviour
  line names resize-pane, layout commands and "until tmux stops".
- maybe-false, patched (one format token): a sidebar alone in its tab could be adopted at full width
  before `reap`; not reproduced, `#{pane_at_right}` rules it out.
- low, rejected: holds-still counts frames, not time; costs extra resizes only, never fights a drag.
- low, rejected: no minimum width; only a deliberate `0` or a drag to the edge reaches it, a drag back
  fixes it.
- low, rejected: default 26 in two places (width.rs, tabrail-tmux), each commented.
- false: vmConfig path in a repo file; the repo is the owner's and already names vmConfig files.
- low, rejected: "tab 0" vs CAP-2's "tab 1"; tmux index 0 is the first tab.
- deferred: spec/stories.yaml drift (DW-13); no unit tests for `Width::keep` (DW-14).
