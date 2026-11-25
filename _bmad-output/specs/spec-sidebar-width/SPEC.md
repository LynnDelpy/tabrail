---
id: SPEC-sidebar-width
companions:
  - brownfield.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Sidebar keeps one width through resizes

## Why

Pain, for the one user of the sidebar: resizing feels clunky and breaks the sidebar. tmux spreads every terminal resize evenly over sidebar and content, so the sidebar drifts (down to 1 or 2 columns, never back to 26). Each tab drifts on its own, so the sidebar jumps on a tab switch. Reflowed leftovers, old tab rows among them, stay on screen. Dragging the border fixes one tab only. Causes and repro: `brownfield.md`.

## Capabilities

- **CAP-1**
  - **intent:** Every tab's sidebar has one shared width; a terminal resize leaves it alone and the content pane takes the whole change.
  - **success:** Three tabs at 120 columns, viewer narrowed to 73 then widened to 200, every tab visited: each sidebar's `pane_width` is the shared width (26 by default).
- **CAP-2**
  - **intent:** Dragging a sidebar's border sets the shared width; every other tab shows it when visited, and new tabs open at it.
  - **success:** Tab 1's border dragged from 26 to 34: tab 1 is 34 at release, tabs 2 and 3 are 34 when visited, a new tab opens at 34, and a later terminal resize keeps 34.
- **CAP-3**
  - **intent:** After any change of its size, the sidebar shows exactly its current rows: no fragments, no stale or duplicated tab rows.
  - **success:** After a narrow-and-widen and after a drag, `capture-pane` of the sidebar equals a freshly started sidebar's at the same size (row 1 blank, no `2 tmp` leftover).

## Constraints

- One width per tmux server, held in tmux (no file), so every sidebar and `tabrail-tmux ensure` (`WIDTH=26` today) use the same value; default 26, settable in tmux.conf.
- Nothing spawned per resize or tab switch (README's speed promise): corrections go over the sidebar's control connection or through hooks and bindings without `run-shell`.
- Drawing only changed lines and pausing while a menu is open (AGENTS.md pitfalls) stay; a full repaint happens only when the size changes.
- `README.md` (setup, behaviour) and AGENTS.md's option list gain the width option and the drag rule.
- Verification only on throwaway tmux servers, the viewer started with `-f /dev/null`; never the user's tmux.

## Non-goals

- Per-tab or per-session widths.
- A hide/show toggle, or the sidebar on the right.
- Hiding the sidebar when the terminal is narrow (CAP-4, retired 2026-11-25).
- How rows are cut at narrow widths (tab-rows owns the row layout).
- Zoom (prefix z) behaviour.

## Success signal

On a throwaway tmux server running tabrail with three tabs: after narrowing the viewer to 73 columns, widening it to 200, visiting every tab and dragging tab 1's border to 34, every sidebar's `pane_width` is 34, `capture-pane` of each shows only current rows, and a new tab opens at 34.

## Assumptions

- "Old tabs or duplications" means stale or repeated tab rows inside the sidebar after a resize (reflow leftovers), not a second sidebar pane in a tab.
- The width is the global option `@sidebar_width`, default 26.
- Only a mouse drag of a sidebar's border sets the width; a keyboard `resize-pane` on a sidebar is undone like any other change.
- One width across all sessions of the server; a dragged width lasts until the server stops (resurrect does not save global options).
- One frame at the wrong width right after switching to a tab that tmux just resized is acceptable; the settled width is the contract.

## Open Questions

- tmux layout commands (prefix Space `next-layout`, `select-layout`) move or resize the sidebar: restore it to the left at the shared width, or out of scope?
