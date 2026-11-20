---
id: SPEC-group-colour
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Choose a group's colour

## Why

Pain for the sidebar's user: a group's colour (the faint tint behind its row) is set once, when New group… picks the first unused tint, and changing it means editing the groups file by hand, while groups are already renamed, folded and reordered with the mouse. Colour should be chosen the same way.

## Capabilities

- **CAP-1**
  - **intent:** The user can change an existing group's colour from that group's right-click menu, picking from a fixed palette.
  - **success:** Right-click group A's row, Colour…, pick a swatch: A's row shows that tint in every sidebar of every session without a restart, and still does after `tabrail-tmux restart` and a new tmux server. Esc in the picker changes nothing.
- **CAP-2**
  - **intent:** The user sees each palette colour as it looks behind a group row before picking, and which one the group has now.
  - **success:** Every picker row shows its colour as a swatch with the group's header text on it; the row of the group's current tint is marked current.
- **CAP-3**
  - **intent:** The user can narrow the long palette by typing part of a colour's name.
  - **success:** Typing `te` in the picker leaves only the teal rows; Enter on one picks it as in CAP-1.

## Constraints

- The colour stays where it lives today: column 2 of the group's line in the groups file (the file `@tabrail_groups` names, else `groups.conf`). A change rewrites only that field; comments and every other line stay byte for byte.
- A failed or interrupted write never leaves the groups file broken or empty: written through a symlink, mode kept, renamed into place, as group moves do. It is the user's config.
- Only palette colours are offered, each keeping the bold header text and icon readable.
- The picker is `tabrail-pick`'s searchable list in a popup (Claude Code option-list style, fzf); nothing new to install.
- Every sidebar of every session shows the new tint on its next redraw (`sidebar-refresh`), no restart.
- `README.md` is the behaviour reference: its groups and menu text change with this work.

## Non-goals

- A colour for Default: it stays plain and its menu gets no Colour… entry.
- Typing a custom hex in the UI (hand-editing the groups file still works).
- Picking a colour as part of New group… (its auto-pick stays) or from a tab's menu.
- Colouring more than today's header-row tint: tab rows, icon and text keep their colours.
- Changing a group's icon.

## Success signal

On a throwaway tmux server with two groups holding tabs, an SGR right-click on group A's row, Colour…, a colour's name and Enter turns A's row background to that colour in `capture-pane -e` of every sidebar, and A's line in the groups file holds the new hex in column 2 with every other line unchanged (`diff`); the same steps ending in Esc leave the file identical (`cmp`).

## Assumptions

- The palette has 25 named colours: the 7 tints New group… already cycles through (Blue, Brown, Green, Plum, Teal, Olive, Violet), so existing groups stay marked, plus Catppuccin Mocha's accent hues at a faint and a deep depth, near-duplicates dropped. One list serves both the picker and the auto-pick, which starts with the 7.
- Readable means the bold `#cdd6f4` header text has at least 4.5:1 contrast on each tint.
- Two groups may share a colour; the picker does not block it, and New group… still prefers an unused one.
- A hand-written tint outside the palette stays valid; the picker then marks no row current.
- The popup is no taller than the terminal; fzf scrolls the rest.
