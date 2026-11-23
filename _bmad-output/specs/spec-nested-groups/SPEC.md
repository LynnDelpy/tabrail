---
id: SPEC-nested-groups
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Groups inside groups

## Why

Pain for the sidebar's user: groups are flat, so projects that belong together (egov and its customers St.Gallen, Köniz, Kath-Zug, FMH, ville-fr) sit side by side as unrelated groups in one long list. A group should be able to hold groups, any depth deep, built and changed with the mouse the way groups already are.

## Capabilities

- **CAP-1**
  - **intent:** The user can make a group a subgroup of another by indenting its line under that group's line in the groups file, any depth; every sidebar shows the tree.
  - **success:** With `egov`, `  St.Gallen` and `    Archiv` each holding a tab, every sidebar shows egov's header, egov's own tabs, then St.Gallen's header and tabs one column further right, Archiv's two columns; one blank line after egov's whole tree and none inside it.
- **CAP-2**
  - **intent:** Folding a parent hides its whole tree.
  - **success:** Folding egov leaves only its header, counting every tab in its tree (`3 tabs`); unfolding shows St.Gallen folded again if it was folded before.
- **CAP-3**
  - **intent:** A new tab joins the group whose folder is the longest one holding the tab's folder, at any level.
  - **success:** With egov at `~/x/egov` and St.Gallen at `~/x/egov/stgallen`, a tab opened in `~/x/egov/stgallen/src` joins St.Gallen and one in `~/x/egov/core` joins egov, whatever their line order; equal lengths go to the first line.
- **CAP-4**
  - **intent:** The user can create a subgroup of a group from that group's menu (New subgroup…) or with a left click on the `+` at its header's right edge.
  - **success:** Either way a name field opens; Enter adds the name's line indented under the group as its last subgroup and opens a new tab in it, shown in every sidebar; Esc leaves the groups file identical.
- **CAP-5**
  - **intent:** The user can nest an existing group by dragging its header onto another group's header.
  - **success:** Dragging 4tw's header and releasing on St.Gallen's header at or right of the `S` makes 4tw, with its subgroups, St.Gallen's last subgroup in the file and in every sidebar.
- **CAP-6**
  - **intent:** The user can move a group into another group, or back to the top level, from the group's menu (Move into group…).
  - **success:** The picker lists Top level and every group but the group itself and its descendants; picking one moves the group's tree there as its last subgroup (or to the end of the top level, above Default); Esc changes nothing.
- **CAP-7**
  - **intent:** Dragging a group reorders it, with its whole tree, among its siblings, without changing its level.
  - **success:** 4tw released on any of egov's tab rows, or on a header of egov's tree left of that header's name, goes above egov (dragged up) or below egov's tree (dragged down); Köniz (in egov) released on 4tw's rows changes nothing; a press and release on a header still folds it, except on its `+`.

## Constraints

- Nesting lives only in the groups file (the file `@tabrail_groups` names, else `groups.conf`) as indentation: a group's parent is the nearest group line above it with less leading whitespace; written lines indent 2 spaces per level; the columns stay name, tint, icon, folders. A flat file shows and behaves as today, except for CAP-3's rule.
- CAP-3's longest folder replaces the line-order match rule for every group, flat ones included; line order still sets display order.
- Group names stay unique across the whole tree: a tab's `@group` and `@sidebar_fold` hold names only.
- Every groups-file write moves a group with its whole subtree, re-indents only the moved lines, and keeps comments and every other line byte for byte; it writes through a symlink, keeps the mode and renames into place, as `move-group` does, and never leaves the file broken or empty. It is the user's config.
- Every writer (rename, colour, move, new) keeps each line's leading whitespace.
- Default stays top-level and last: it cannot be nested, holds no subgroups, and gets no `+`, New subgroup… or Move into group….
- A drag changes a group's level only through a header drop at or right of the target's name; taking a group out to the top level is the menu or the file.
- Standard library only, files under 100 lines (AGENTS.md); `model.rs` and `draw.rs` are at 97 and 96.
- `README.md` is the behaviour reference: its groups, mouse, menu and match-order text change with this work; `groups.conf.example` explains indentation.

## Non-goals

- Taking a group out of its parent by drag.
- Deleting groups.
- Subgroups inheriting their parent's tint or icon.
- Nesting from a tab's menu: its New group… stays top-level.
- Subgroups of Default.
- A drag preview or drop marker while the button is held.
- Nesting or reordering by keyboard.

## Success signal

On a throwaway tmux server with groups `egov`, indented `St.Gallen`, deeper `Archiv` and `4tw`, each holding a tab: `capture-pane` of every sidebar shows the tree indented one column per level; an SGR drag of 4tw's header released on St.Gallen's header right of its name leaves the groups file with 4tw indented 4 spaces under St.Gallen and every other line unchanged (`diff`); the same drag released on one of egov's tab rows puts 4tw's line above egov's; folding egov hides all three subgroups; a new tab in `egov/stgallen` joins St.Gallen.

## Assumptions

- Leading spaces and tabs count one each; an indented line with no shallower group line above is top-level.
- A group shows when it or any descendant holds a tab; a parent without tabs of its own shows its header, then its subgroups.
- New subgroup… and `+` pick an unused tint and icon as New group… does; every subgroup keeps its own. A name that exists anywhere adds no line and opens a new tab in that group, as New group… does today.
- The `+` shows dim at the right edge of every header but Default's; a long header is cut for it; pressing on `+` and releasing elsewhere is a group drag. With spec-group-timer's time after the name, the name is still cut first and the `+` stays at the edge.
- A new tab of a group without folders opens in its nearest ancestor's first folder, else home.
- Moving or dropping a group into its own parent makes it the parent's last subgroup; into itself or a descendant does nothing.
- Folder lengths compare after `~` expansion, symlinks not resolved, as matching does today.
- Move to group… (tabs) lists groups in tree order, indented one column per level.
- Rename changes only the name; subgroups belong by position, so nothing else changes.
