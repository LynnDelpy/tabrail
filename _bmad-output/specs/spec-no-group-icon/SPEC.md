---
id: SPEC-no-group-icon
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Groups without their icon

## Why

Lynn is bothered by the symbol in front of each group's name: the icon from groups.conf (`✚`, `■`, `★`, `◆` …, Default's `•`). The icon goes from everywhere it is shown, and the name gets its 2 columns.

## Capabilities

- **CAP-1**
  - **intent:** A group's sidebar row, Default's included, shows its fold arrow and name with no icon, open or folded.
  - **success:** At width 26: Website open with no timer reads `▾ Website` + 15 spaces + `▷ `; Infra folded with 3 tabs and no timer reads `▸ Infra  3 tabs` + 9 spaces + `▷ `; `Abcdefghijklmnopqrstuvwxyz0123` folded with `▶ 2h0m` reads `▸ Abcdefghijklmnopq ▶ 2h0m` (2 more name characters than today); Default reads `▾ Default`; a subgroup two levels down reads `  ▸ Archiv`.
- **CAP-2**
  - **intent:** The Colour… picker and the Move to group list name groups without their icon.
  - **success:** Website's Colour… swatches read ` Website ` on each tint (not ` ◆ Website `); Move to group lists `egov  1 tab · current`, ` St.Gallen  1 tab` (one column in per level, as today) and `Default  0 tabs`, with no icon or `•`.

## Constraints

- Group rows: arrow in column depth + 1, name from column depth + 3 (top level: 1 and 3). The tint, bold, timer, `▷`, folded count and cut order (count first, then name) stay as they are. Clicks on the time and `▷` follow their new columns: Website's `▶ 12m` is columns 11–15 (13–17 today); a press and release elsewhere on the row still folds.
- Tab rows stay as they are: `❯` for the current tab in column 2, names from column 4.
- groups.conf keeps its format (name, tint, icon, folders): the icon column is still read and never shown, so existing files work unchanged. New group… still writes an unused icon.
- Nested groups (spec-nested-groups story 1, merged) lose the icon the same way: headers become `{indent}{▾|▸} {name}`. Its story 4 (ready-for-dev) places a header's name at depth + 5; built after this spec, it uses depth + 3.
- Tests that find a group's row by its icon find it by the arrow and name (`[▾▸] NAME`) instead. README and groups.conf.example say the icon column is not shown.

## Non-goals

- Removing the fold arrow `▾`/`▸`.
- Changing the current tab's `❯` or any tab row.
- Dropping the icon column from groups.conf or migrating existing files.
- Another mark per group in the icon's place (glyph or colour beyond the tint).

## Success signal

On a throwaway tmux server with groups Website (`◆`) and Infra (`✚`): `capture-pane` of a sidebar shows `▾ Website`, `▾ Infra` and `▾ Default` starting in column 1, with no `◆`, `✚` or `•` anywhere. The Colour… picker and Move to group list show the names without icons. A click on Website's running time stops it, and a press and release on Infra's row folds it. `cargo test` and the timer and colour test scripts pass.

## Assumptions

- groups.conf.example and README keep documenting the icon column, marked as not shown, so old and new files read alike.
