---
id: SPEC-no-fold-arrow
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Group rows without the fold arrow

## Why

Lynn wants the symbol in front of a group's name gone: every group row starts with a fold arrow (`▾` open, `▸` folded) before its icon. The arrow repeats what the row already shows (a folded group lists `N tabs` and no tab rows, and its menu offers Unfold) and takes 2 columns from the group's name.

## Capabilities

- **CAP-1**
  - **intent:** A group's row, open or folded, shows its icon and name with no fold arrow, and the name gets the arrow's 2 columns.
  - **success:** At width 26: Website open with no timer reads `◆ Website` + 15 spaces + `▷ `; Infra folded with 3 tabs and no timer reads `◆ Infra  3 tabs` + 9 spaces + `▷ `; `Abcdefghijklmnopqrstuvwxyz0123` folded with `▶ 2h0m` reads `◆ Abcdefghijklmnopq ▶ 2h0m` (2 more name characters than today). No sidebar row contains `▾` or `▸`.

## Constraints

- Only the arrow and the space after it go: the icon is in column 1, the name starts in column 3. The tint, bold, timer, `▷`, folded count and cut order (count first, then name) stay as they are.
- Tab rows stay as they are: `❯` for the current tab in column 2, names from column 4.
- A press and release anywhere on a group's row still folds or unfolds it, and the menu's Fold/Unfold and `@sidebar_fold` stay the same. Clicks on the time and the `▷` follow their new columns: Website's `▶ 12m` is columns 11–15 (13–17 today).
- No other fold mark replaces the arrow. A folded group with no tabs, shown because it has a timer, looks the same as an open one.
- spec-nested-groups (ready-for-dev) draws headers as `{indent}{▾|▸} {icon} {name}` (story 1) with the name at column depth + 5 (story 4). Whichever of the two specs is built second drops the arrow from nested headers too, so the name starts at depth + 3.
- Tests that read the fold state from the arrow read it from `@sidebar_fold` or the folded count instead. README's example rows and Files table stay accurate.

## Non-goals

- Changing or removing the current tab's `❯`, or moving tab names.
- Removing group icons.
- A new fold indicator (glyph, colour, a count on open groups).
- Changing the fold click, the menu's Fold/Unfold, or `@sidebar_fold`.

## Success signal

On a throwaway tmux server with groups Website and Infra: `capture-pane` of a sidebar shows `◆ Website` and `▲ Infra` starting in column 1, with no `▾` or `▸` anywhere. A press and release on Infra's row folds it (`▲ Infra  1 tab`, Infra added to `@sidebar_fold`) and a second one unfolds it. A click on Website's running time stops it. `cargo test` and the timer test scripts pass.

## Assumptions

- The icon moves to column 1 with no leading space, as in the preview Lynn picked (`◆ Website               ▷`).
- A folded group with no tabs looking like an open one is acceptable: the option Lynn picked said the fold state would then show only through the count.
