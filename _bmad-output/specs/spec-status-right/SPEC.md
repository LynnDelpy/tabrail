---
id: SPEC-status-right
companions:
  - row-layout.md
  - ../spec-tab-rows/row-layout.md
  - ../../../README.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Status and timers at the right edge

## Why

Pain, for the sidebar's one user: each tab row's status (`⚙ 2m`, `⏱ 4m`, `✓ 3m12s`, `✗ 2`) follows its name, so timers sit at a different column on every row and cannot be scanned down. Right-aligned, they end in one column. Before and after rows: `row-layout.md`.

## Capabilities

- **CAP-1**
  - **intent:** A tab row's status ends at the sidebar's right edge, so the timers of all rows end in one column; name and program stay on the left.
  - **success:** At width 26, `deploy · vim` with `⏱ 4m` and `infra` with `✗ 2` both end in column 26, as in `row-layout.md`; at width 34 they both end in column 34.
- **CAP-2**
  - **intent:** A long name is still cut to leave room for program and status, with at least one space before the status; a row without status is unchanged.
  - **success:** `Fix login redirect` with `✳ ⚙ 2m` at width 26 reads ` ❯ Fix login redire ✳ ⚙ 2m`, as today; `notes` with no status reads as today.

## Constraints

- The right edge is the sidebar's current width (one shared width, spec-sidebar-width): no fixed column.
- The program keeps its place right after the name and its drop rule (left out when fewer than 8 of the name's characters would show), as spec-tab-program built it.
- Only changed lines are drawn (AGENTS.md menu pitfall): a timer tick still rewrites one row.
- README's example and status description show the right-aligned status.
- Verification only on a throwaway tmux server; never the user's tmux.

## Non-goals

- The clock line.
- Group rows, folded `N tabs` included.
- What the program or status shows (Claude's `✳`, nothing for shells, the icons).
- Icons in a column of their own.

## Success signal

On a throwaway tmux server with tabs running a command for 4 s, a failed command, a finished Claude and an idle shell, `capture-pane` of the sidebar shows every status ending in the last column at width 26, and again after the shared width is set to 34; names and programs read as before.

## Assumptions

- The whole status (icon and time) is right-aligned, so icons of rows with times of different length do not line up; the times' last characters do.
- The padding between name/program and status is plain space in the row's normal colour; the current tab's bold and `❯` are unchanged.
