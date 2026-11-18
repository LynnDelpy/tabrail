---
id: SPEC-group-drag
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Drag groups to reorder them

## Why

Pain for the sidebar's user: group order is the line order of the groups file, so changing it means editing the file by hand, while tabs already move with the mouse. Groups should move the same way.

## Capabilities

- **CAP-1**
  - **intent:** The user can drag a group's row onto another group to change the order groups appear in.
  - **success:** With groups A, B, C shown, dragging C's row onto A's row shows C, A, B in every sidebar of every session; dragging A onto C then shows B, C, A; the order holds after `tabrail-tmux restart` and a new tmux server.
- **CAP-2**
  - **intent:** A click on a group's row still folds or unfolds it, and a drag never does.
  - **success:** Press and release on the same group row toggles its fold; a press on one group row released on another leaves both folds as they were.

## Constraints

- The order stays where it lives today, the groups file's line order (the file `@tabrail_groups` names, else `groups.conf`); a reorder keeps comments and every untouched line byte for byte.
- A failed or interrupted write never leaves the groups file broken or empty: it is the user's config.
- Tab dragging, tab drop targets and the right-click menus behave as before.
- `README.md` is the behaviour reference: its mouse and groups text change with this work.

## Non-goals

- A drag preview or drop marker while the button is held.
- Reordering by keyboard or menu.
- Merging or nesting groups by dropping one onto another.

## Success signal

On a throwaway tmux server with three groups holding tabs, an SGR drag from the last group's row released on the first group's row puts that group first in `capture-pane` of every sidebar and first among the group lines of the groups file, comments untouched; a plain click on a group row still folds it.

## Assumptions

- Default is not in the groups file: it stays last, cannot be dragged, and nothing lands below it.
- Dropping on any row of group H (its header or one of its tabs) counts as dropping on H; moving up lands above H, moving down below H, as tab drops do.

## Open Questions

- File order also decides which group a new tab joins when two groups' folders overlap: should reordering change that too, or should match priority stay separate from display order?
